import { invoke } from '@tauri-apps/api/core'

import type {
  ProviderInfo,
  SessionBreakdownItem,
  SessionDetailResponse,
  SetSyncSettingsResponse,
  SyncAllResponse,
  SyncSettings,
  SyncStatus,
  SyncWindowPreset,
  TimeBucket,
  TimeseriesPoint,
  UsageRange,
  UsageSource,
} from './types'

export function syncAll(force = false): Promise<SyncAllResponse> {
  return invoke('sync_all', { args: { force } })
}

export function getProviders(): Promise<ProviderInfo[]> {
  return invoke('get_providers')
}

export function setProviderEnabled(id: string, enabled: boolean): Promise<ProviderInfo[]> {
  return invoke('set_provider_enabled', { args: { id, enabled } })
}

export function getSyncStatus(): Promise<SyncStatus> {
  return invoke('get_sync_status')
}

export function getSyncSettings(): Promise<SyncSettings> {
  return invoke('get_sync_settings')
}

export function setSyncSettings(
  preset: SyncWindowPreset,
  customSinceMs?: number | null,
): Promise<SetSyncSettingsResponse> {
  return invoke('set_sync_settings', {
    args: { preset, custom_since_ms: customSinceMs ?? null },
  })
}

export function getUsageTimeseries(
  range: UsageRange,
  bucket: TimeBucket,
  source: UsageSource = 'all',
): Promise<TimeseriesPoint[]> {
  return invoke('get_usage_timeseries', { args: { range, bucket, source } })
}

export function getSessionBreakdown(
  range: UsageRange,
  limit = 20,
  source: UsageSource = 'all',
): Promise<SessionBreakdownItem[]> {
  return invoke('get_session_breakdown', { args: { range, limit, source } })
}

export function getSessionDetail(sessionId: string): Promise<SessionDetailResponse | null> {
  return invoke('get_session_detail', { args: { session_id: sessionId } })
}
