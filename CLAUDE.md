# CLAUDE.md

Scout Beam — solo deduction game against a bot. Setup and commands: [`README.md`](README.md).

- **Backend** (`api/`): Rust, axum + sqlx on PostgreSQL — always via `make`
- **Frontend** (`front/`): Next.js App Router, React 19, Tailwind CSS 4, shadcn/ui, pnpm

**Domain vocabulary lives in `CONTEXT.md`** — read it before naming or modelling anything.

## Frontend (`front/app/`)

Same conventions as Mawster's front.

- `services/` — API wrappers over `lib/api.ts`, which attaches the JWT and refreshes it
- `components/ui/` — shadcn/ui — **never modify directly**, except `button.tsx` and `card.tsx`;
  no `h-*` on a `Button`
- Page-scoped components in `_components/`, hooks holding a page's state in `_viewmodels/`;
  files ≤150 lines
- No `contexts/` folder for now: state stays in the page and its viewmodels
- `@radix-ui/react-slot` ≥1.3 ships no `"use client"`: a Server Component importing `Button` fails
  prerender with `createContext is not a function`. Add `'use client'` to that file.
- Check a change with `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware`. Lint is oxlint
  alone, sonarjs rules included through `jsPlugins`: suppress with oxlint directives.

## Key Conventions

- **Language**: English (code, comments, variables)
- **i18n**: `useI18n()` always — never hardcode strings; keys in both
  `front/app/i18n/locales/en.ts` and `fr.ts`; `{{key}}` placeholders filled with `fill()`
- **Icons**: `lucide-react`
- **Styling**: Tailwind semantic tokens (`bg-card`, `text-muted-foreground`), dark mode first;
  game colours are data and live in `lib/colours.ts`
- **E2E hooks**: `data-cy` on anything a test would target

### Comments — keep them rare

- **2 lines maximum.** No block above a config line, env var, workflow step or list entry.
- One rationale lives in **one** place. Never narrate the incident that motivated the code.
- Worth a comment: why something is _absent_, why a workaround exists, why an order is load-bearing.
