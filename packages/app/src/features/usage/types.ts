export type UsageRange = '24h' | '7d' | '30d'
export type TimeBucket = 'hour' | 'day'
export type UsageSource = 'all' | string

export interface ProviderSyncReport {
  id: string
  files_scanned: number
  events_ingested: number
}

export interface SyncAllResponse {
  reports: ProviderSyncReport[]
}

export interface TimeseriesPoint {
  ts: number
  input: number
  cached: number
  output: number
  reasoning: number
  total: number
}

export interface SessionBreakdownItem {
  id: string
  source: string
  title: string | null
  total_tokens: number
  model: string | null
  cwd: string | null
  started_at: number | null
}

export interface SessionRow {
  id: string
  source: string
  title: string | null
  cwd: string | null
  model: string | null
  started_at: number | null
  ended_at: number | null
  input_tokens: number
  cached_input_tokens: number
  output_tokens: number
  reasoning_output_tokens: number
  total_tokens: number
  turn_count: number
  originator: string | null
  git_repo: string | null
}

export interface UsageEventRow {
  id: string
  source: string
  session_id: string
  turn_id: string | null
  ts: number
  model: string | null
  input_tokens: number
  cached_input_tokens: number
  output_tokens: number
  reasoning_output_tokens: number
  total_tokens: number
  rollout_path: string
}

export interface SessionDetailResponse {
  session: SessionRow
  events: UsageEventRow[]
}

export interface ProviderInfo {
  id: string
  display_name: string
  home: string
  enabled: boolean
  session_count: number
  total_tokens: number
}

export interface SyncStatus {
  last_sync: number | null
  providers: ProviderInfo[]
}
