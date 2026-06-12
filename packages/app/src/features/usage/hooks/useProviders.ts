import { listen } from '@tauri-apps/api/event'
import { useCallback, useEffect, useState } from 'react'

import { getProviders, setProviderEnabled } from '../api'
import type { ProviderInfo } from '../types'

export function useProviders() {
  const [providers, setProviders] = useState<ProviderInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setError(null)
    try {
      const list = await getProviders()
      setProviders(list)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }, [])

  const toggle = useCallback(async (id: string, enabled: boolean) => {
    setError(null)
    try {
      const list = await setProviderEnabled(id, enabled)
      setProviders(list)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      throw e
    }
  }, [])

  useEffect(() => {
    void refresh()
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

  const enabledProviders = providers.filter((p) => p.enabled)

  return { providers, enabledProviders, loading, error, refresh, toggle }
}
