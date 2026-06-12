import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'

import { getSessionBreakdown, getSyncStatus, getUsageTimeseries, syncAll } from '../api'
import type {
  SessionBreakdownItem,
  SyncStatus,
  TimeBucket,
  TimeseriesPoint,
  UsageRange,
  UsageSource,
} from '../types'

export function useUsage(range: UsageRange, bucket: TimeBucket, source: UsageSource) {
  const [timeseries, setTimeseries] = useState<TimeseriesPoint[]>([])
  const [sessions, setSessions] = useState<SessionBreakdownItem[]>([])
  const [status, setStatus] = useState<SyncStatus | null>(null)
  const [loading, setLoading] = useState(true)
  const [syncing, setSyncing] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setError(null)
    try {
      const [ts, breakdown, syncStatus] = await Promise.all([
        getUsageTimeseries(range, bucket, source),
        getSessionBreakdown(range, 20, source),
        getSyncStatus(),
      ])
      setTimeseries(ts)
      setSessions(breakdown)
      setStatus(syncStatus)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }, [range, bucket, source])

  const sync = useCallback(
    async (force = false) => {
      setSyncing(true)
      setError(null)
      try {
        await syncAll(force)
        await refresh()
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e))
      } finally {
        setSyncing(false)
      }
    },
    [refresh],
  )

  useEffect(() => {
    setLoading(true)
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

  useEffect(() => {
    let unlisten: (() => void) | undefined
    void listen('providers:changed', () => {
      void refresh()
    }).then((fn) => {
      unlisten = fn
    })
    return () => {
      unlisten?.()
    }
  }, [refresh])

  return { timeseries, sessions, status, loading, syncing, error, refresh, sync }
}
