# Scout Beam v1

**Goal:** a player logs in, starts a game against the bot, fires shots from the 36 entries, places
a draft and submits it until a proposition is correct.

## Global Constraints

- Vocabulary is `CONTEXT.md`: Grid, Entry, Piece, Straight side, Slanted side, Layout, Bot, Game,
  Shot, Beam colour, Draft, Proposition. Code, comments and identifiers in English.
- Back in `api/`, a Rust crate modelled on `SNEAXIII/mawster-rs` (clone at `/home/user/mawster-rs`):
  same layering `domains/<x>/{model, dto, error, repository, service, routes}`, same `[lints]`
  block, utoipa on every route, sqlx forward-only migrations in `api/migrations/`, `#[sqlx::test]`
  integration tests with the `TestApp` harness, same `Makefile` target names.
- No WebSocket: every action is one HTTP request answered with its result.
- The hidden layout never leaves the server while the game is not solved — not in a response, not
  in a log line.
- A game belongs to its player: another player's game answers 404, never 403.
- No points, no score, no shot counter shown.
- Front in `front/`, modelled on Mawster's front (`/home/user/Mawster/front`): Next.js App Router,
  React 19, Tailwind CSS 4 semantic tokens, shadcn/ui, pnpm, oxlint, files ≤150 lines,
  page-scoped components in `_components/`. French UI only, no i18n layer in v1.
- Pieces are drawn as plain SVG polygons in flat colours — no images, no gradients.
- Ports, so it runs next to Mawster: API `8002`, front `3001`, PostgreSQL dev `5436`, test `5437`.

---

### Task 1: API skeleton and login

**Files:** `api/Cargo.toml`, `api/src/{main,lib,app,state,config}.rs`, `api/src/shared/`,
`api/src/auth/`, `api/src/domains/user/`, `api/migrations/0001_user.sql`, `api/tests/`,
`api/Makefile`, `api/compose.yaml`, `api/.env.example`, `api/Dockerfile`, `.github/workflows/ci.yml`,
`README.md`

Bootstrap the crate (package name `scout-beam`) by copying what the POC already proves and
trimming it to one need: an account that can log in. Keep `auth/` (JWT codec, extractor), the user
domain's `POST /auth/dev-login`, `GET /auth/session`, `POST /auth/refresh`, the `seed_user` binary,
`shared/` (error, db, openapi, validation), `/health` and `/docs`. Drop alliance, game account,
admin routes, fixtures, load scripts, the audit middleware, `email_hash`, `discord_id`,
`google_id` and the trigram index. The `user` table keeps id, login, role, timestamps.
CI runs fmt, clippy and tests against a PostgreSQL service, without the Docker push job. The
README says how to start the database, migrate, seed an account and run the API.

**Done when:** `make check` passes, and `/docs` lists only `/health` and the three `/auth/*` routes.

### Task 2: Beam engine

**Files:** `api/src/engine/{mod,grid,piece,beam,colour}.rs`

A pure module — no axum, sqlx or serde-on-HTTP types — that, given a layout and an entry, returns
the exit entry and the beam colour.

- Grid: 10 columns × 8 rows, origin top-left, `x` to the right, `y` downward.
- Entry numbering, imposed: top edge `1–10` left to right, right edge `11–18` top to bottom,
  bottom edge `19–28` right to left, left edge `29–36` bottom to top. A beam enters perpendicular
  to its edge.
- Each cell a piece covers is either full, or half (a right triangle filling one corner: NE, NW,
  SE or SW). A full cell and the straight sides of a half cell turn the beam back; the slanted
  side of a half cell turns it 90°.
- The five shapes at rotation 0, cell offsets `(x, y)`: Square `(0,0) full`; Bar `(0,0) full,
  (1,0) full`; Spike `(0,0) half SW`; Corner `(0,0) half SW, (0,1) full, (1,1) half SW`;
  Diamond `(0,0) half SE, (1,0) half SW, (0,1) half NE, (1,1) half NW`. Rotation is in quarter
  turns clockwise and rotates the half-cell corners too.
- Piece colours: red, blue, yellow, white. Beam colour is the mix of the colours the beam bounced
  off: none if no bounce, white if only white, otherwise red, blue, yellow, violet (red+blue),
  orange (red+yellow), green (blue+yellow), black (all three).
- A layout is valid when it has the five shapes once each, every cell inside the grid, no two
  pieces overlapping and no two pieces sharing a cell side.
- Guard the trace with a step cap and return an error past it rather than looping.

**Done when:** unit tests cover every entry edge, a bounce off each kind of side from each
direction, a beam that comes back to its own entry, each row of the colour table, and each
invalid-layout rule.

### Task 3: Bot

**Files:** `api/src/engine/bot.rs`

`engine::bot::draw_layout(rng)` returns a valid layout: the five shapes at random positions and
rotations, colours drawn from red, blue, yellow, white with at least one red, one blue and one
yellow. It retries until valid. It takes the RNG as a parameter so tests can seed it.

**Done when:** a test draws 1,000 seeded layouts and every one is valid and holds the three
primaries; the same seed gives the same layout.

### Task 4: Games and shots

**Files:** `api/migrations/0002_game.sql`, `api/src/domains/game/`, `api/tests/game.rs`

Tables `game` (id, user_id, layout JSONB, status `in_progress | solved`, created_at, solved_at)
and `shot` (id, game_id, entry 1–36, exit_entry, colour, created_at). Routes, all behind
`CurrentUser`:

- `POST /games` — the bot draws a layout, the game is stored, answers the game view.
- `GET /games` — the caller's games, newest first, without layout.
- `GET /games/{id}` — status, shots in firing order, `layout` only when solved.
- `POST /games/{id}/shots` with `{ "entry": n }` — traces with the engine, stores and answers
  `{ entry, exit_entry, colour }`. Re-firing an entry already fired answers the stored shot
  without inserting a second one. Firing on a solved game answers 409.

**Done when:** integration tests prove each route, the 404 on another player's game, the 409, the
absence of `layout` before solving, the entry bounds (0 and 37 rejected), and `/docs` lists the
four routes.

### Task 5: Propositions

**Files:** `api/migrations/0003_proposition.sql`, `api/src/domains/game/`, `api/tests/proposition.rs`

Table `proposition` (id, game_id, layout JSONB, correct, created_at). `POST /games/{id}/propositions`
takes a layout (five pieces: shape, x, y, rotation, colour). An invalid layout answers 422 with
the broken rule. A valid one is correct when the engine gives the same exit entry and beam colour
as the hidden layout on all 36 entries. The answer is `{ correct }`, plus `layout` — the hidden
one — when correct. A correct proposition marks the game solved in the same transaction. A
proposition on a solved game answers 409.

**Done when:** integration tests cover a correct proposition equal to the hidden layout, a correct
one placed differently but indistinguishable, a wrong one, an invalid one, and the 409.

### Task 6: Front skeleton, login and game list

**Files:** `front/` (Next.js project), `front/app/login/`, `front/app/page.tsx`,
`front/app/_components/`, `front/app/lib/api.ts`, `front/app/services/games.ts`

Create the Next.js app on Mawster's stack and dark-first theme. `lib/api.ts` reads
`NEXT_PUBLIC_API_URL`, attaches the JWT from `localStorage` as `Authorization: Bearer` and sends
the user to `/login` on a 401. `/login`: one login field calling `POST /auth/dev-login`. `/`: the
player's games (date, in progress or solved) and a "Nouvelle partie" button calling `POST /games`
then opening `/games/{id}`. Every interactive element carries a `data-cy`.

**Done when:** with the API running, logging in, starting a game and coming back to `/` shows it in
the list; `pnpm exec oxlint --type-aware` and `tsc` are clean.

### Task 7: Game page — shots

**Files:** `front/app/games/[id]/page.tsx`, `front/app/games/[id]/_components/`,
`front/app/services/games.ts`

Draw the grid with its 36 entries numbered as in Task 2. Clicking an entry calls
`POST /games/{id}/shots`, then marks the entry and its exit with the beam colour, and adds a line
to the shot list. Fired entries stay marked on reload, read from `GET /games/{id}`. A solved game
shows the hidden layout and disables shooting.

**Done when:** firing every entry in the browser marks all 36, reload keeps them, and the colours
match the API answers.

### Task 8: Draft and proposition

**Files:** `front/app/games/[id]/_components/`, `front/app/services/games.ts`

The draft is drawn on the game grid itself, inside the ring of entries: pick one of the five
shapes, rotate it a quarter turn, pick its colour, click a cell to place it, click a placed piece
to remove it. Each shape can be placed once.
"Proposer" is enabled when the five are placed and calls `POST /games/{id}/propositions`: a 422
shows the broken rule, a wrong one says "Incorrect", a correct one switches the game to solved
(Task 7). The draft is kept in `localStorage` per game so a reload keeps it.

**Done when:** a game can be solved end to end in the browser by copying the layout read from the
database, and a wrong draft is refused with "Incorrect".
