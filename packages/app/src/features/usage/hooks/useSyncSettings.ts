import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'

import { getSyncSettings, setSyncSettings } from '../api'
import type { SyncSettings, SyncWindowPreset } from '../types'

export function useSyncSettings() {
  const [settings, setSettings] = useState<SyncSettings | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setError(null)
    try {
      const next = await getSyncSettings()
      setSettings(next)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }, [])

  const update = useCallback(async (preset: SyncWindowPreset, customSinceMs?: number | null) => {
    setSaving(true)
    setError(null)
    try {
      const response = await setSyncSettings(preset, customSinceMs)
      setSettings(response.settings)
      return response
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      throw e
    } finally {
      setSaving(false)
    }
  }, [])

  useEffect(() => {
    void refresh()
  }, [refresh])

  useEffect(() => {
    let unlisten: (() => void) | undefined
    void listen('usage:updated', () => {
      void refresh()
    }).then((fn) => {
      unlisten = fn
    })
    return () => {
      unlisten?.()
    }
  }, [refresh])

  return { settings, loading, saving, error, refresh, update }
}
