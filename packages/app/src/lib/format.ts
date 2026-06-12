export function formatTokens(value: number): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(1)}M`
  }
  if (value >= 1_000) {
    return `${(value / 1_000).toFixed(1)}K`
  }
  return value.toLocaleString()
}

export function formatShortPath(path: string | null, max = 48): string {
  if (!path) return '—'
  if (path.length <= max) return path
  const parts = path.split('/')
  const file = parts.pop() ?? ''
  const tail = parts.slice(-2).join('/')
  const joined = tail ? `…/${tail}/${file}` : `…/${file}`
  return joined.length > max ? `…/${file}` : joined
}

export function sessionLabel(title: string | null, id: string): string {
  if (title && title.trim()) return title.trim()
  return id.slice(0, 8)
}
