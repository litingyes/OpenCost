import { format } from 'date-fns'
import { useEffect, useState } from 'react'

import {
  Drawer,
  DrawerContent,
  DrawerDescription,
  DrawerHeader,
  DrawerTitle,
} from '@/components/ui/drawer'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table'
import { getSessionDetail } from '@/features/usage/api'
import { providerLabel } from '@/features/usage/providers'
import type { SessionDetailResponse } from '@/features/usage/types'
import { formatShortPath, formatTokens, sessionLabel } from '@/lib/format'

interface SessionDetailDrawerProps {
  sessionId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SessionDetailDrawer({ sessionId, open, onOpenChange }: SessionDetailDrawerProps) {
  const [detail, setDetail] = useState<SessionDetailResponse | null>(null)
  const [loading, setLoading] = useState(false)

  useEffect(() => {
    if (!open || !sessionId) {
      setDetail(null)
      return
    }

    setLoading(true)
    void getSessionDetail(sessionId)
      .then(setDetail)
      .finally(() => setLoading(false))
  }, [open, sessionId])

  const session = detail?.session

  return (
    <Drawer open={open} onOpenChange={onOpenChange} direction="right">
      <DrawerContent className="data-[vaul-drawer-direction=right]:sm:max-w-lg">
        <DrawerHeader>
          <DrawerTitle>{session ? sessionLabel(session.title, session.id) : 'Session'}</DrawerTitle>
          <DrawerDescription className="truncate">
            {session ? providerLabel(session.source) : '—'} · {session?.model ?? '—'} ·{' '}
            {formatShortPath(session?.cwd ?? null)}
          </DrawerDescription>
        </DrawerHeader>

        {loading && <p className="px-4 text-sm text-muted-foreground">Loading session…</p>}

        {session && !loading && (
          <div className="flex flex-col gap-4 overflow-y-auto px-4 pb-6">
            <dl className="grid grid-cols-2 gap-3 text-sm">
              <div>
                <dt className="text-muted-foreground">Total</dt>
                <dd className="font-mono font-medium tabular-nums">
                  {formatTokens(session.total_tokens)}
                </dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Turns</dt>
                <dd className="font-mono tabular-nums">{session.turn_count}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Input</dt>
                <dd className="font-mono tabular-nums">{formatTokens(session.input_tokens)}</dd>
              </div>
              <div>
                <dt className="text-muted-foreground">Output</dt>
                <dd className="font-mono tabular-nums">{formatTokens(session.output_tokens)}</dd>
              </div>
            </dl>

            <div>
              <h3 className="mb-2 text-sm font-medium">Turn timeline</h3>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Time</TableHead>
                    <TableHead className="text-right">Input</TableHead>
                    <TableHead className="text-right">Output</TableHead>
                    <TableHead className="text-right">Total</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {detail.events.map((event) => (
                    <TableRow key={event.id}>
                      <TableCell className="text-xs text-muted-foreground">
                        {format(event.ts, 'MMM d HH:mm:ss')}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs tabular-nums">
                        {formatTokens(event.input_tokens)}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs tabular-nums">
                        {formatTokens(event.output_tokens)}
                      </TableCell>
                      <TableCell className="text-right font-mono text-xs tabular-nums">
                        {formatTokens(event.total_tokens)}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </div>
          </div>
        )}
      </DrawerContent>
    </Drawer>
  )
}
