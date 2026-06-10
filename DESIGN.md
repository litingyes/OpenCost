---
name: OpenCost
description: A precise desktop instrument for local agent token spend — insightful, dense, Linear-adjacent.
colors:
  background: 'oklch(1 0 0)'
  foreground: 'oklch(0.145 0 0)'
  primary: 'oklch(0.205 0 0)'
  primary-foreground: 'oklch(0.985 0 0)'
  muted: 'oklch(0.97 0 0)'
  muted-foreground: 'oklch(0.556 0 0)'
  border: 'oklch(0.922 0 0)'
  destructive: 'oklch(0.577 0.245 27.325)'
  sidebar: 'oklch(0.985 0 0)'
  sidebar-primary: 'oklch(0.205 0 0)'
typography:
  display:
    fontFamily: "'Geist Variable', sans-serif"
    fontSize: '1.5rem'
    fontWeight: 600
    lineHeight: 1.2
    letterSpacing: '-0.02em'
  title:
    fontFamily: "'Geist Variable', sans-serif"
    fontSize: '1rem'
    fontWeight: 500
    lineHeight: 1.375
    letterSpacing: 'normal'
  body:
    fontFamily: "'Geist Variable', sans-serif"
    fontSize: '0.875rem'
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: 'normal'
  label:
    fontFamily: "'Geist Variable', sans-serif"
    fontSize: '0.75rem'
    fontWeight: 500
    lineHeight: 1.33
    letterSpacing: '0.01em'
rounded:
  sm: 'calc(0.625rem * 0.6)'
  md: 'calc(0.625rem * 0.8)'
  lg: '0.625rem'
  xl: 'calc(0.625rem * 1.4)'
spacing:
  xs: '0.5rem'
  sm: '0.75rem'
  md: '1rem'
  lg: '1.5rem'
components:
  button-primary:
    backgroundColor: '{colors.primary}'
    textColor: '{colors.primary-foreground}'
    rounded: '{rounded.lg}'
    padding: '0 0.625rem'
    height: '2rem'
  button-primary-hover:
    backgroundColor: '{colors.primary}'
    textColor: '{colors.primary-foreground}'
    rounded: '{rounded.lg}'
  button-outline:
    backgroundColor: '{colors.background}'
    textColor: '{colors.foreground}'
    rounded: '{rounded.lg}'
    padding: '0 0.625rem'
    height: '2rem'
  button-ghost:
    backgroundColor: 'transparent'
    textColor: '{colors.foreground}'
    rounded: '{rounded.lg}'
    padding: '0 0.625rem'
    height: '2rem'
  input-default:
    backgroundColor: 'transparent'
    textColor: '{colors.foreground}'
    rounded: '{rounded.lg}'
    padding: '0.25rem 0.625rem'
    height: '2rem'
  card-default:
    backgroundColor: '{colors.background}'
    textColor: '{colors.foreground}'
    rounded: '{rounded.xl}'
    padding: '1rem'
---

# Design System: OpenCost

## Overview

**Creative North Star: "The Spend Instrument"**

OpenCost reads like a precision desktop tool, not a finance dashboard. The visual system is built for solo developers who glance at token attribution between IDE sessions: tight hierarchy, neutral surfaces, and motion that confirms state rather than decorates. Insight lives in typography, spacing, and data emphasis — not neon accents or hero metrics.

The current scaffold uses shadcn radix-nova on a neutral OKLCH grayscale. That is intentional restraint: the product's opinionated voice should come through in what gets highlighted (anomalies, waste, drift), not through decorative chrome. Linear is the north star for density, focus rings, and crisp transitions.

This system explicitly rejects crypto/fintech neon, generic SaaS dashboard slop, and enterprise bloat — per PRODUCT.md anti-references.

**Key Characteristics:**

- Single sans family (Geist Variable) at compact sizes — analytical, not editorial
- Tonal layering and hairline rings instead of drop shadows
- Compact controls (32px default button/input height) for desk-side density
- Dark mode as a first-class peer, not an afterthought
- Accent color reserved for meaning (destructive, future insight highlights) — not decoration

## Colors

A neutral instrument palette: near-black ink on clean white, with stepped grays for hierarchy. No brand chroma yet — when an accent arrives, it should signal insight or action, not wallpaper.

### Primary

- **Instrument Ink** (oklch(0.205 0 0)): Primary actions, key labels, sidebar active emphasis. The default "do something" color — confident, not loud.

### Neutral

- **Canvas** (oklch(1 0 0)): App background in light mode. True white, not warm cream.
- **Ink** (oklch(0.145 0 0)): Body text and primary content. High contrast against Canvas.
- **Whisper** (oklch(0.556 0 0)): Secondary labels, placeholders, metadata. Must still meet 4.5:1 on Canvas — bump toward Ink if borderline.
- **Surface Wash** (oklch(0.97 0 0)): Muted backgrounds, hover fills, secondary buttons.
- **Hairline** (oklch(0.922 0 0)): Borders and input strokes. Structural, not decorative.
- **Focus Ring** (oklch(0.708 0 0)): Keyboard focus halos at 50% opacity.

### Tertiary

- **Alert Ember** (oklch(0.577 0.245 27.325)): Destructive actions and error states only. Warm red-orange — never used as a brand accent.

### Named Rules

**The No-Neon Rule.** No glow, purple gradients, or trading-terminal drama. If a color feels like crypto/fintech, it is forbidden.

**The One Accent Rule.** When a brand accent is introduced, it appears on ≤10% of any screen and only where it carries meaning (insight callouts, primary CTA). Rarity is the point.

## Typography

**Display Font:** Geist Variable (with system sans fallback)
**Body Font:** Geist Variable (with system sans fallback)

**Character:** Technical humanist — clean, slightly tight, optimized for scanning numbers and labels at small sizes. One family in multiple weights; no competing sans pairings.

### Hierarchy

- **Display** (600, 1.5rem, 1.2): Page titles and primary section headers. Letter-spacing −0.02em max.
- **Title** (500, 1rem, 1.375): Card titles, panel headers, sidebar group labels.
- **Body** (400, 0.875rem / 14px, 1.5): Default UI copy and table content. Cap line length at 65–75ch in prose blocks.
- **Label** (500, 0.75rem / 12px, 0.01em tracking): Metadata, chart axis labels, badge text.

### Named Rules

**The Density Rule.** Default UI text is 14px (0.875rem). Reserve larger sizes for hierarchy jumps, not decoration.

## Elevation

Flat-by-default with tonal contrast. Depth is conveyed through background steps (background → card → muted), hairline rings (`ring-1 ring-foreground/10` on cards), and border shifts — not drop shadows.

### Shadow Vocabulary

No shadow vocabulary is defined yet. Shadows appear only as a response to state (popover lift, drag) if added later — never as default card decoration.

### Named Rules

**The Flat-By-Default Rule.** Surfaces are flat at rest. Rings and borders replace the ghost-card pattern (1px border + wide blur shadow). Pick one depth signal, not both.

## Components

### Buttons

- **Shape:** Gently rounded (10px / `--radius-lg`), compact 32px height default
- **Primary:** Instrument Ink fill, near-white text. Hover: 80% opacity fill. Active: 1px downward translate.
- **Outline:** Hairline border, Canvas background. Hover: Surface Wash fill.
- **Ghost:** Transparent at rest. Hover: Surface Wash fill. For toolbar and sidebar actions.
- **Destructive:** Alert Ember at 10% background tint — never solid neon red blocks.
- **Focus:** 3px ring at 50% Focus Ring opacity. Border shifts to ring color.

### Cards / Containers

- **Corner Style:** 14px (`rounded-xl`)
- **Background:** Card token (matches Canvas in light mode)
- **Shadow Strategy:** None — `ring-1 ring-foreground/10` hairline instead
- **Border:** Ring only; no paired drop shadow
- **Internal Padding:** 16px (`--spacing(4)`), 12px for `sm` variant

### Inputs / Fields

- **Style:** Hairline border, transparent background (light) / 30% input tint (dark), 10px radius, 32px height
- **Focus:** Ring treatment matching buttons — border-ring + 3px ring at 50%
- **Placeholder:** Whisper color — verify 4.5:1 contrast; darken if needed
- **Error:** Alert Ember border + ring at 20% opacity

### Navigation

- **Sidebar:** 16rem expanded width, Surface Wash / sidebar token background, compact item height
- **Active state:** Sidebar-primary (Instrument Ink) for emphasis — not a colored stripe
- **Mobile:** Sheet overlay; keyboard shortcut `b` toggles sidebar
- **Typography:** Title weight for group labels, Body for items

## Do's and Don'ts

### Do:

- **Do** use Geist at 14px for default UI density — this is a desk-side instrument, not a marketing page.
- **Do** convey depth with tonal steps and hairline rings, not drop shadows.
- **Do** reserve Alert Ember for errors and destructive confirmation — never as decoration.
- **Do** support dark mode with equal care; test contrast in both themes.
- **Do** use focus-visible rings on every interactive element (WCAG 2.1 AA keyboard nav).

### Don't:

- **Don't** use crypto / fintech neon aesthetics (glow, purple gradients, trading-terminal drama).
- **Don't** ship generic SaaS dashboard slop (cream heroes, gradient metric cards, eyebrow kickers on every section).
- **Don't** fall into enterprise bloat (corporate blue, alert fatigue, tables without narrative).
- **Don't** pair a 1px border with a wide soft drop shadow on the same element.
- **Don't** use border-left greater than 1px as a colored accent stripe on cards or list items.
- **Don't** rely on color alone in charts — pair hue with weight, pattern, or label.
