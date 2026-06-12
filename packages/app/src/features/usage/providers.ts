export const PROVIDER_LABELS: Record<string, string> = {
  codex: 'Codex',
  claude: 'Claude Code',
}

export function providerLabel(id: string): string {
  return PROVIDER_LABELS[id] ?? id
}
