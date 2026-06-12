# OpenCost

> The Spend Instrument — a precise desktop tool for local AI agent token usage.

## Goals

OpenCost is a native desktop app that helps solo developers track and explain local AI agent token usage: which sessions, models, and workflows consume the most, and where waste or surprising patterns show up.

The target user runs Cursor, CLI agents, or IDE copilots alongside their IDE or terminal. They open OpenCost for quick checks at the desk—not long reporting sessions.

Success means turning opaque token burn into actionable clarity in seconds, not exporting spreadsheets. The product is the desktop app first; a marketing site may come later.

Visually and interactively, the north star is Linear-adjacent density and analysis: a precision dev tool, not a finance dashboard or a flashy consumer app.

For deeper context, see [PRODUCT.md](./PRODUCT.md) (strategy) and [DESIGN.md](./DESIGN.md) (visual system).

## Tech stack

- **Desktop shell:** Tauri 2
- **Frontend:** React 19, TypeScript, Vite
- **Styling:** Tailwind CSS 4, shadcn/ui (radix-nova)
- **Typography:** Geist Variable
- **Package management:** pnpm monorepo (`packages/app`)

## Development

```bash
pnpm install
cd packages/app && pnpm tauri dev   # dev server: http://localhost:1620
```

Build the frontend only:

```bash
pnpm --filter app build
```

## Short-term roadmap

### Done

- [x] Project scaffold (Tauri + React + shadcn/ui)
- [x] Product and visual specs (PRODUCT.md / DESIGN.md)

### Up next

- [x] Track usage for Codex (Claude Code next)
- [ ] Analyze which dimensions token spend goes to
- [ ] App Dashboard
- [ ] App Bar
- [ ] CLI
- [ ] …

## Related docs

- [PRODUCT.md](./PRODUCT.md) — product strategy
- [DESIGN.md](./DESIGN.md) — visual system
- [AGENTS.md](./AGENTS.md) — agent context
