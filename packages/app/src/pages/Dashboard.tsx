import { format, startOfDay } from 'date-fns'
import { CalendarIcon, RefreshCw } from 'lucide-react'
import { useEffect, useState } from 'react'

import { SessionBreakdownChart } from '@/components/charts/SessionBreakdownChart'
import { UsageTimeSeriesChart } from '@/components/charts/UsageTimeSeriesChart'
import { SessionDetailDrawer } from '@/components/SessionDetailDrawer'
import { Button } from '@/components/ui/button'
import { Calendar } from '@/components/ui/calendar'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card'
import { Popover, PopoverAnchor, PopoverContent } from '@/components/ui/popover'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { Switch } from '@/components/ui/switch'
import { useProviders } from '@/features/usage/hooks/useProviders'
import { useSyncSettings } from '@/features/usage/hooks/useSyncSettings'
import { useUsage } from '@/features/usage/hooks/useUsage'
import { SYNC_WINDOW_PRESETS, syncWindowLabel } from '@/features/usage/syncWindow'
import type { SyncWindowPreset, TimeBucket, UsageRange, UsageSource } from '@/features/usage/types'
import { formatTokens } from '@/lib/format'

const CHART_RANGE_OPTIONS: { value: UsageRange; label: string }[] = [
  { value: '1d', label: '1d' },
  { value: '7d', label: '7d' },
  { value: '30d', label: '30d' },
  { value: '180d', label: '6mo' },
  { value: '365d', label: '1y' },
]

export function Dashboard() {
  const [range, setRange] = useState<UsageRange>('7d')
  const [bucket, setBucket] = useState<TimeBucket>('day')
  const [source, setSource] = useState<UsageSource>('all')
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [customCalendarOpen, setCustomCalendarOpen] = useState(false)

  const { providers, enabledProviders, toggle: toggleProvider } = useProviders()
  const {
    settings: syncSettings,
    saving: savingSyncSettings,
    update: updateSyncSettings,
  } = useSyncSettings()
  const { timeseries, sessions, status, loading, syncing, error, sync } = useUsage(
    range,
    bucket,
    source,
  )

  useEffect(() => {
    if (source !== 'all' && !enabledProviders.some((p) => p.id === source)) {
      setSource('all')
    }
  }, [source, enabledProviders])

  const rangeTotal = timeseries.reduce((sum, p) => sum + p.total, 0)

  function handleRangeChange(value: UsageRange) {
    setRange(value)
    setBucket(value === '1d' ? 'hour' : 'day')
  }

  async function handleSyncWindowChange(value: SyncWindowPreset) {
    if (value === 'custom') {
      setCustomCalendarOpen(true)
      return
    }
    await updateSyncSettings(value)
  }

  async function handleCustomDateSelect(date: Date | undefined) {
    if (!date) return
    const sinceMs = startOfDay(date).getTime()
    await updateSyncSettings('custom', sinceMs)
    setCustomCalendarOpen(false)
  }

  function handleSessionSelect(sessionId: string) {
    setSelectedSessionId(sessionId)
    setDrawerOpen(true)
  }

  const syncWindowSelectValue =
    syncSettings?.preset === 'custom' ? 'custom' : (syncSettings?.preset ?? '7d')

  const customSelectedDate =
    syncSettings?.custom_since_ms != null ? new Date(syncSettings.custom_since_ms) : undefined

  return (
    <div className="min-h-screen bg-background">
      <header className="border-b px-6 py-4">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h1 className="font-heading text-lg font-semibold tracking-tight">OpenCost</h1>
            <p className="text-sm text-muted-foreground">Agent token usage</p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Select value={source} onValueChange={(v) => setSource(v as UsageSource)}>
              <SelectTrigger className="w-[130px]" size="sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All</SelectItem>
                {enabledProviders.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    {p.display_name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={range} onValueChange={(v) => handleRangeChange(v as UsageRange)}>
              <SelectTrigger className="w-[88px]" size="sm">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CHART_RANGE_OPTIONS.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Popover open={customCalendarOpen} onOpenChange={setCustomCalendarOpen}>
              <PopoverAnchor asChild>
                <div>
                  <Select
                    value={syncWindowSelectValue}
                    onValueChange={(v) => void handleSyncWindowChange(v as SyncWindowPreset)}
                    disabled={savingSyncSettings || syncing}
                  >
                    <SelectTrigger className="w-[120px]" size="sm">
                      <SelectValue>
                        {syncSettings
                          ? syncWindowLabel(syncSettings.preset, syncSettings.custom_since_ms)
                          : 'History'}
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      {SYNC_WINDOW_PRESETS.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </PopoverAnchor>
              <PopoverContent className="w-auto p-0" align="end">
                <Calendar
                  mode="single"
                  selected={customSelectedDate}
                  onSelect={(date) => void handleCustomDateSelect(date)}
                  disabled={(date) => date > new Date()}
                />
              </PopoverContent>
            </Popover>
            {syncSettings?.preset === 'custom' && (
              <Button
                variant="outline"
                size="sm"
                className="px-2"
                onClick={() => setCustomCalendarOpen(true)}
                disabled={savingSyncSettings || syncing}
                aria-label="Change custom sync start date"
              >
                <CalendarIcon className="size-4" />
              </Button>
            )}
            <Button
              variant="outline"
              size="sm"
              onClick={() => void sync(false)}
              disabled={syncing || savingSyncSettings}
            >
              <RefreshCw className={syncing ? 'animate-spin' : ''} />
              Sync
            </Button>
            <div className="rounded-lg border px-3 py-1.5 text-sm">
              <span className="text-muted-foreground">Range </span>
              <span className="font-mono font-medium tabular-nums">{formatTokens(rangeTotal)}</span>
            </div>
          </div>
        </div>

        <div className="mt-3 flex flex-col gap-2">
          {providers.map((provider) => (
            <div
              key={provider.id}
              className={`flex flex-wrap items-center gap-3 text-sm ${provider.enabled ? '' : 'opacity-60'}`}
            >
              <Switch
                checked={provider.enabled}
                onCheckedChange={(checked) => void toggleProvider(provider.id, checked)}
                aria-label={`Enable ${provider.display_name}`}
              />
              <span className="min-w-[6rem] font-medium">{provider.display_name}</span>
              <span className="font-mono text-xs tabular-nums text-muted-foreground">
                {formatTokens(provider.total_tokens)} · {provider.session_count} sessions
              </span>
              <span className="truncate text-xs text-muted-foreground">{provider.home}</span>
            </div>
          ))}
        </div>

        <p className="mt-2 text-xs text-muted-foreground">
          {status?.last_sync && <>Last sync {format(status.last_sync, 'MMM d HH:mm')}</>}
          {syncSettings && (
            <>
              {status?.last_sync && ' · '}
              {syncSettings.effective_since_ms != null ? (
                <>History from {format(syncSettings.effective_since_ms, 'MMM d, yyyy')}</>
              ) : (
                'All history'
              )}
            </>
          )}
        </p>
        {error && <p className="mt-2 text-sm text-destructive">{error}</p>}
      </header>

      <main className="grid gap-4 p-6 lg:grid-cols-2">
        <Card className="lg:col-span-2">
          <CardHeader className="pb-2">
            <CardTitle className="text-base">Usage over time</CardTitle>
            <CardDescription>
              Stacked by input, cached input, output, and reasoning tokens
            </CardDescription>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="flex h-56 items-center justify-center text-sm text-muted-foreground">
                Loading…
              </div>
            ) : (
              <UsageTimeSeriesChart data={timeseries} bucket={bucket} />
            )}
          </CardContent>
        </Card>

        <Card className="lg:col-span-2">
          <CardHeader className="pb-2">
            <CardTitle className="text-base">Top sessions</CardTitle>
            <CardDescription>Click a bar to inspect turn-level usage</CardDescription>
          </CardHeader>
          <CardContent>
            {loading ? (
              <div className="flex h-48 items-center justify-center text-sm text-muted-foreground">
                Loading…
              </div>
            ) : (
              <SessionBreakdownChart
                data={sessions}
                showSource={source === 'all'}
                onSelect={handleSessionSelect}
              />
            )}
          </CardContent>
        </Card>
      </main>

      <SessionDetailDrawer
        sessionId={selectedSessionId}
        open={drawerOpen}
        onOpenChange={setDrawerOpen}
      />
    </div>
  )
}
