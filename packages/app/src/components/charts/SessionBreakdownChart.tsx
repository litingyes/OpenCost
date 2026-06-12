import { Bar, BarChart, CartesianGrid, Cell, XAxis, YAxis } from 'recharts'

import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from '@/components/ui/chart'
import { providerLabel } from '@/features/usage/providers'
import type { SessionBreakdownItem } from '@/features/usage/types'
import { formatTokens, sessionLabel } from '@/lib/format'

const chartConfig = {
  total_tokens: { label: 'Total tokens', color: 'var(--chart-1)' },
} satisfies ChartConfig

interface SessionBreakdownChartProps {
  data: SessionBreakdownItem[]
  showSource?: boolean
  onSelect?: (sessionId: string) => void
}

export function SessionBreakdownChart({
  data,
  showSource = false,
  onSelect,
}: SessionBreakdownChartProps) {
  const formatted = data.map((session) => {
    const base = sessionLabel(session.title, session.id)
    const label = showSource ? `${providerLabel(session.source)} · ${base}` : base
    return { ...session, label }
  })

  if (formatted.length === 0) {
    return (
      <div className="flex h-48 items-center justify-center rounded-lg border border-dashed text-sm text-muted-foreground">
        No sessions in this range
      </div>
    )
  }

  const height = Math.max(160, formatted.length * 28)

  return (
    <ChartContainer config={chartConfig} className="aspect-auto w-full" style={{ height }}>
      <BarChart
        data={formatted}
        layout="vertical"
        margin={{ left: 4, right: 16, top: 4, bottom: 4 }}
      >
        <CartesianGrid horizontal={false} strokeDasharray="3 3" />
        <XAxis
          type="number"
          tickLine={false}
          axisLine={false}
          tickFormatter={(v) => formatTokens(Number(v))}
        />
        <YAxis
          type="category"
          dataKey="label"
          width={showSource ? 160 : 120}
          tickLine={false}
          axisLine={false}
          tick={{ fontSize: 11 }}
        />
        <ChartTooltip
          content={
            <ChartTooltipContent
              formatter={(value) => (
                <span className="font-mono tabular-nums">{formatTokens(Number(value))}</span>
              )}
            />
          }
        />
        <Bar dataKey="total_tokens" fill="var(--color-total_tokens)" radius={4}>
          {formatted.map((entry) => (
            <Cell
              key={`${entry.source}:${entry.id}`}
              cursor={onSelect ? 'pointer' : 'default'}
              onClick={() => onSelect?.(entry.id)}
            />
          ))}
        </Bar>
      </BarChart>
    </ChartContainer>
  )
}
