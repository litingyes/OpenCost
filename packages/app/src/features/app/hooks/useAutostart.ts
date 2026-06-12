import { disable, enable, isEnabled } from '@tauri-apps/plugin-autostart'
import { useCallback, useEffect, useState } from 'react'

export function useAutostart() {
  const [enabled, setEnabled] = useState(false)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setError(null)
    try {
      setEnabled(await isEnabled())
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }, [])

  const toggle = useCallback(async (next: boolean) => {
    setError(null)
    try {
      if (next) {
        await enable()
      } else {
        await disable()
      }
      setEnabled(next)
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e))
      throw e
    }
  }, [])

  useEffect(() => {
    void refresh()
  }, [refresh])

  return { enabled, loading, error, refresh, toggle }
}
