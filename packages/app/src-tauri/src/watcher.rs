use crate::commands::AppState;
use crate::providers::{enabled_providers, sync_enabled_providers};
use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub struct WatcherHandle {
    restart_tx: mpsc::Sender<()>,
}

pub fn start_watcher(app: AppHandle) -> WatcherHandle {
    let (restart_tx, restart_rx) = mpsc::channel();
    std::thread::spawn(move || {
        if let Err(e) = run_watcher_loop(app, restart_rx) {
            eprintln!("OpenCost watcher error: {e}");
        }
    });
    WatcherHandle { restart_tx }
}

pub fn request_restart(app: &AppHandle) {
    if let Ok(watcher) = app.state::<AppState>().watcher.lock() {
        if let Some(handle) = watcher.as_ref() {
            let _ = handle.restart_tx.send(());
        }
    }
}

fn run_watcher_loop(app: AppHandle, restart_rx: mpsc::Receiver<()>) -> Result<(), String> {
    loop {
        let watch_paths = collect_watch_paths(&app)?;
        if watch_paths.is_empty() {
            match restart_rx.recv_timeout(Duration::from_secs(5)) {
                Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => continue,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
            }
        }

        let (event_tx, event_rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    let _ = event_tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )
        .map_err(|e| e.to_string())?;

        for path in &watch_paths {
            if path.exists() {
                let mode = if path.ends_with("archived_sessions") {
                    RecursiveMode::NonRecursive
                } else {
                    RecursiveMode::Recursive
                };
                let _ = watcher.watch(path, mode);
            }
        }

        let mut pending = false;
        loop {
            match restart_rx.try_recv() {
                Ok(()) => break,
                Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
                Err(mpsc::TryRecvError::Empty) => {}
            }

            match event_rx.recv_timeout(Duration::from_secs(2)) {
                Ok(event) => match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any => {
                        pending = true;
                    }
                    _ => {}
                },
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if pending {
                        pending = false;
                        trigger_sync(&app);
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
    }
}

fn collect_watch_paths(app: &AppHandle) -> Result<Vec<PathBuf>, String> {
    let state = app.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let providers = enabled_providers(&db)?;
    let mut paths = Vec::new();
    for provider in providers {
        for path in provider.watch_paths() {
            if !paths.contains(&path) {
                paths.push(path);
            }
        }
    }
    Ok(paths)
}

fn trigger_sync(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(db) = state.db.lock() {
            let _ = sync_enabled_providers(&db, false);
        }
    }
    let _ = app.emit("usage:updated", ());
}
