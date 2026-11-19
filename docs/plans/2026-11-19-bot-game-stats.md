# Bot game timer, score and stats

**Goal:** time every Bot game, score it, give it points on a logarithmic curve shared with the
daily, and show the player's Bot game stats on the profile.

## Global Constraints

- Vocabulary from `CONTEXT.md`: **Bot game** (a game with no `daily_date` and no `room_id`),
  **Score**, **Points**, **Surrender**.
- Score of a Bot game = whole seconds from `created_at` to its end + 60 per wrong proposition
  + 300 if surrendered. A daily's score is unchanged (seconds + 60 per wrong proposition).
- Points = `GREATEST(0, round(1000 - 200 * ln(1 + score / 60.0)))::int8`, defined once as the SQL
  function `points(score int8)` and used by every query; never recomputed in Rust or TypeScript.
- Games in progress count nowhere in the stats; Race and Duel games are never part of them.
- Backend: layering of `CLAUDE.md`, a changed query needs `make prepare`, check with `make check`
  in `api/`. Frontend: `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware` in `front/`,
  every string through `useI18n()` in `en.ts` and `fr.ts`, `data-cy` on what a test targets.

---

### Task 1: Surrender time and the points function

**Files:** `api/migrations/0011_bot_game_score.sql`, `api/src/repositories/daily_repository.rs`,
`api/src/repositories/game_repository.rs`, `api/src/models/game_model.rs`,
`api/src/dto/game_dto.rs`, `api/tests/daily.rs`, `api/tests/game.rs`

Migration: add `game.surrendered_at TIMESTAMPTZ`, backfill it with `updated_at` for the games
already surrendered, and a check that it is set exactly when the status is `surrendered` (same
shape as `game_solved_at_matches_status`). Create the immutable SQL function `points(score int8)
RETURNS int8` with the formula of the Global Constraints.

`mark_surrendered` sets `surrendered_at = now()`. Add `surrendered_at` to `Game` (every query
returning a `Game` selects it) and to `GameView`.

Global ranking: replace `GREATEST(100, 1000 - score / 6)` with `points(score)` and drop the
`ponytail:` comment about the old weights.

**Done when:** a test shows a daily solved with score 0 is worth 1000 points and one of score 300
(e.g. two wrong propositions and 3 min, or adjusted timestamps) is worth 642; a test shows a
surrendered game has `surrendered_at` in its `GET /games/{id}` response; `make check` passes.

### Task 2: Bot game score in the list, and the stats endpoint

**Files:** `api/src/repositories/game_repository.rs`, `api/src/models/game_model.rs`,
`api/src/dto/game_dto.rs`, `api/src/services/game_service.rs`,
`api/src/controllers/game_controller.rs`, `api/tests/game.rs`

`GET /games`: each summary gains `score` and `points`, both `Option<i64>`, set only for a finished
(solved or surrendered) Bot game, computed in the listing query with the Score rule and `points()`.

New `GET /games/stats` (authenticated, registered in the router and in OpenAPI) answering
`BotGameStats` for the caller's Bot games:

- `played`, `solved`, `surrendered`: counts (`played` includes games in progress);
- `best_seconds`: fewest seconds of a solved Bot game without penalties, `null` if none;
- `average_points`: average points of the 5 latest finished Bot games, rounded, `null` if none;
- `recent`: the 20 latest finished Bot games, oldest first, each `{ ended_at, score, points,
  surrendered }`.

Declare the route before `/games/{id}` so `stats` is not read as an id. The score expression lives
in one place in the repository (a CTE or a view reused by both queries), not copied.

**Done when:** integration tests cover a solved Bot game with one wrong proposition (score =
seconds + 60), a surrendered one (+300), a daily and a room game left out of the stats, and
`average_points` over only the 5 latest; `make check` passes.

### Task 3: Timer on Bot games, score in the game list

**Files:** `front/app/games/[id]/_components/daily-timer.tsx` → `game-timer.tsx`,
`front/app/games/[id]/_components/game-header.tsx`, `front/app/services/games.ts`,
`front/app/play/_components/game-list.tsx`, `front/app/i18n/locales/en.ts`, `fr.ts`

Rename `DailyTimer` to `GameTimer` (`data-cy='game-timer'`, `game-penalty`; update any Cypress
spec using the old hooks) with an `endedAt` prop instead of `solvedAt`; the header shows it on a
daily and on a Bot game (`room_code` null), frozen at `solved_at ?? surrendered_at`. Its title
string moves from `t.daily.timer` to a key under `t.game`.

`GameSummary` and `GameView` types gain `score`, `points` and `surrendered_at` as the API sends
them. In `/play`, a finished Bot game row shows its score with `clock()` and its points
(`data-cy='game-score'`, `game-points`).

**Done when:** a Bot game in progress shows a ticking timer, a surrendered one shows it stopped;
the `/play` list shows score and points of finished Bot games; tsc and oxlint pass.

### Task 4: Bot game stats on the profile

**Files:** `front/app/services/games.ts`, `front/app/profile/page.tsx`,
`front/app/profile/_viewmodels/use-bot-stats.ts`, `front/app/profile/_components/bot-stats.tsx`,
`front/app/profile/_components/score-curve.tsx`, `front/app/i18n/locales/en.ts`, `fr.ts`

`getBotStats()` in `services/games.ts` calls `GET /games/stats`. The profile page, for a
registered player only, shows a card with played / solved / surrendered, best time (`clock()`),
average points of the last 5 (`data-cy='stats-average'`), and the score curve.

`score-curve.tsx`: hand-drawn inline SVG, no chart library: one polyline of `recent` scores in
order, y growing with the score (lower is better, say it in the axis label), each game a dot,
surrendered ones in a distinct colour taken from `lib/colours.ts` or a semantic token; a dot's
`<title>` gives its score and points. Fewer than two finished games: a short empty message instead.

**Done when:** a registered player with finished Bot games sees the card and the curve on
`/profile`, a guest does not; tsc and oxlint pass.
