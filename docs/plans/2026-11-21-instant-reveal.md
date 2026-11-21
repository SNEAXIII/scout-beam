# Instant reveal

**Goal:** a bot game and a daily start with all 36 entries already fired; Race and Duel keep firing
one shot at a time.

## Global Constraints

- Vocabulary from `CONTEXT.md` (Bot game, Daily, Shot); already updated: "every entry is fired from
  the start".
- Backend through `make`; a changed or added query needs `make prepare`; check with `make check`.
- Front checked with `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware`; every string via
  `useI18n()`, keys in both `en.ts` and `fr.ts`.
- Comments rare, 2 lines max.
- No migration: dailies already in progress are completed when resumed; bot games already in
  progress finish one shot at a time.

---

### Task 1: The server fires every entry of a bot game and a daily

**Files:** `api/src/services/game_service.rs`, `api/src/services/daily_service.rs`,
`api/src/repositories/game_repository.rs`, `api/tests/game.rs`, `api/tests/daily.rs`

Add `game_service::fire_all(executor, game)`: traces every `Entry::all()` on the game's layout with
`engine::trace` and stores each with the existing `insert_shot_if_absent`, so it is idempotent and
the shot order is entries 1 to 36. Build the `Shot` exactly as `fire` does (reuse or extract that
conversion, do not duplicate it).

- `game_service::create` (bot game) inserts the game and calls `fire_all` in one transaction:
  `game_repository::insert` must accept an executor (`PgExecutor`, like `insert_shot_if_absent`).
- `daily_service::start` calls `fire_all` on the game it returns, whether created or found, when
  that game is in progress: an existing daily with a few shots gets the rest. A finished daily is
  left untouched.
- Room games (`room_service::join`, `hide`) are not changed.
- The `fire` message stays as it is: on a bot game or a daily it just replays a stored shot.

Tests: `a_new_game_is_in_progress_with_no_shot_and_no_layout` becomes "with all 36 shots and no
layout". Every `game.rs` test that fires a *new* entry on a bot game and checks a new row, a trace,
or the order (`a_shot_answers_what_the_engine_traces…`, `re_firing…`, `concurrent_re_fires…`,
`a_game_reads_back_its_shots_in_firing_order…`, `the_first_and_last_entries…`) moves to a Race
game (see the helpers in `api/tests/live.rs` / `room.rs`) or asserts the pre-fired state instead.
Validation and refusal tests (outside 1–36, solved, surrendered, other player, anonymous) keep a
bot game. In `daily.rs`: a started daily has 36 shots; a daily row inserted with a few shots
(`insert_daily_game` + shots) has 36 after `POST /daily/{date}/game`;
`a_daily_fires_every_entry_at_once_and_has_no_pace` drops its firing loop.

**Done when:** `make check` passes, and `GET /games/{id}` right after `POST /games` or
`POST /daily/{date}/game` returns 36 shots in entry order.

### Task 2: A "Start" button opens the daily

**Files:** `front/app/_components/daily-board.tsx`, `front/app/_viewmodels/use-daily.ts`,
`front/app/games/[id]/page.tsx`, `front/app/i18n/locales/en.ts`, `front/app/i18n/locales/fr.ts`

The daily is no longer started by clicking an entry on the empty grid. In `daily-board.tsx`,
replace the `daily-hint` overlay with a visible `Button` centred over the grid, same place, labelled
by the new key `daily.start` (`en`: `Start`, `fr`: `Commencer`), `data-cy='daily-start'`, disabled
while `starting`. The overlay wrapper loses `pointer-events-none` so the button is clickable; the
grid's entries are disabled while there is no game (`onFire` no longer starts anything).

`useDaily().start` takes no entry and pushes `/games/${game.id}` with no query. In
`games/[id]/page.tsx`, remove the `?fire=` first-shot effect and `useSearchParams`/`useRouter` if
they become unused. Remove the `daily.fireToStart` key from both locales.

**Done when:** tsc and oxlint pass; on `/`, the empty daily grid shows "Commencer", one click opens
the game page with every trace already drawn and the timer running; an in-progress daily still
shows `daily-resume`.
