import { format } from 'date-fns'
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from 'recharts'

import {
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from '@/components/ui/chart'
import type { TimeseriesPoint } from '@/features/usage/types'
import { formatTokens } from '@/lib/format'

const chartConfig = {
  input: { label: 'Input', color: 'var(--chart-1)' },
  cached: { label: 'Cached', color: 'var(--chart-2)' },
  output: { label: 'Output', color: 'var(--chart-3)' },
  reasoning: { label: 'Reasoning', color: 'var(--chart-4)' },
} satisfies ChartConfig

interface UsageTimeSeriesChartProps {
  data: TimeseriesPoint[]
  bucket: 'hour' | 'day'
}

export function UsageTimeSeriesChart({ data, bucket }: UsageTimeSeriesChartProps) {
  const formatted = data.map((point) => ({
    ...point,
    label: format(point.ts, bucket === 'hour' ? 'MMM d, HH:mm' : 'MMM d'),
  }))

  if (formatted.length === 0) {
    return (
      <div className="flex h-48 items-center justify-center rounded-lg border border-dashed text-sm text-muted-foreground">
        No token usage in this range
      </div>
    )
  }

  return (
    <ChartContainer config={chartConfig} className="aspect-auto h-56 w-full">
      <AreaChart data={formatted} margin={{ left: 0, right: 8, top: 8, bottom: 0 }}>
        <CartesianGrid vertical={false} strokeDasharray="3 3" />
        <XAxis dataKey="label" tickLine={false} axisLine={false} tickMargin={8} minTickGap={24} />
        <YAxis
          tickLine={false}
          axisLine={false}
          tickMargin={8}
          width={48}
          tickFormatter={(v) => formatTokens(Number(v))}
        />
        <ChartTooltip
          content={
            <ChartTooltipContent
              labelFormatter={(_, payload) => {
                const ts = payload?.[0]?.payload?.ts as number | undefined
                return ts ? format(ts, bucket === 'hour' ? 'MMM d, yyyy HH:mm' : 'MMM d, yyyy') : ''
              }}
              formatter={(value, name) => (
                <span className="font-mono tabular-nums">
                  {formatTokens(Number(value))}{' '}
                  {chartConfig[name as keyof typeof chartConfig]?.label}
                </span>
              )}
            />
          }
        />
        <ChartLegend content={<ChartLegendContent />} />
        <Area
          type="monotone"
          dataKey="input"
          stackId="tokens"
          stroke="var(--color-input)"
          fill="var(--color-input)"
          fillOpacity={0.6}
        />
        <Area
          type="monotone"
          dataKey="cached"
          stackId="tokens"
          stroke="var(--color-cached)"
          fill="var(--color-cached)"
          fillOpacity={0.6}
        />
        <Area
          type="monotone"
          dataKey="output"
          stackId="tokens"
          stroke="var(--color-output)"
          fill="var(--color-output)"
          fillOpacity={0.6}
        />
        <Area
          type="monotone"
          dataKey="reasoning"
          stackId="tokens"
          stroke="var(--color-reasoning)"
          fill="var(--color-reasoning)"
          fillOpacity={0.6}
        />
      </AreaChart>
    </ChartContainer>
  )
}
