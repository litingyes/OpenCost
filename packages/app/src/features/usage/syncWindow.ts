import type { SyncWindowPreset } from './types'

export const SYNC_WINDOW_PRESETS: { value: SyncWindowPreset; label: string }[] = [
  { value: '1d', label: '1d' },
  { value: '7d', label: '7d' },
  { value: '30d', label: '30d' },
  { value: '180d', label: '6mo' },
  { value: '365d', label: '1y' },
  { value: 'all', label: 'All' },
  { value: 'custom', label: 'Custom…' },
]

export function syncWindowLabel(preset: SyncWindowPreset, customSinceMs: number | null): string {
  if (preset === 'custom' && customSinceMs != null) {
    return new Date(customSinceMs).toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: 'numeric',
    })
  }
  return SYNC_WINDOW_PRESETS.find((p) => p.value === preset)?.label ?? preset
}
