# Redesign features and the v1 style

**Goal:** build every behaviour the UI redesign specs ask for (data, routes, flows, calls to
action) and give each screen the v1 style of the designer's mockups.

## Global Constraints

- Source of truth: the redesign sheets in `~/scout-beam-design/` (`README.md`, `docs/accueil.md`,
  `docs/quotidiennes.md`, `docs/1v1.md`, `docs/navigation.md`, `docs/charte.md`,
  `docs/references-mawster.md`, `docs/style-v1.md`, `docs/retours-maquettes-v1.md`). When this plan and a sheet disagree on behaviour, the sheet wins.
- **Visual: `docs/style-v1.md`** (tokens and building blocks read off the designer's v1 mockups;
  the designer is unavailable, so we extend that style ourselves). Task 1 lays the tokens and the
  shell; each later task styles its own screen with those blocks. Name things after behaviour
  (`DailyCalendar`, `RematchButton`), not after a look.
- Out of scope, kept as `ponytail:` in the sheets: matchmaking, public rooms, Elo, a challenge
  table. Do not stub them.
- Vocabulary is `CONTEXT.md`. New terms to add there with their task: **Streak**, **Rematch**,
  **Challenge**, **Record**.
- API layering and checks are `CLAUDE.md`'s: migrations from `0011_…`, `make prepare` with any
  changed query, integration tests in `api/tests/`, `make check`. The README route table gains a
  line per new route.
- Front: files ≤150 lines, page state in `_viewmodels/`, i18n keys in both `en.ts` and `fr.ts`
  (French copy from the sheets), `data-cy` on every new control, `pnpm exec tsc --noEmit` and
  `pnpm exec oxlint --type-aware`.
- One branch and one PR per task; Task 1 first, the rest in any order unless a task names another.

---

### Task 1: Shell, v1 tokens and settings

**Files:** `front/app/layout.tsx`, `front/app/ui/global.css`, `front/components/nav/*` (new),
`front/app/lib/theme.ts` (new), every `page.tsx` that renders its own header

Shell of `docs/navigation.md`: an icon rail on the left from `md`, a bottom tab bar below it,
**hidden on `/games/[id]`** where a one-line 44px game header (back, timer, `⋯`) takes over.

- Tokens: the dark set of `docs/style-v1.md` as `.dark`, Mawster's `new-a` light set with the v1
  primary as `:root`. Add `--board` and `--board-line`, the same dark values in both themes, as
  the grid's backdrop and lines so `lib/colours.ts` stays readable in light. No `primary` on the
  grid.
- Theme: dark by default, light on request, stored in `localStorage` under `theme` with the
  try/catch pattern of Mawster's `look.ts`; no flash: an inline script in `<head>` applies the
  stored class before paint.
- Nav entries and icons exactly as `docs/navigation.md`: Accueil `/`, Jouer `/play`,
  Quotidiennes `/daily`, Classement `/ranking`, Profil `/profile` (signed out: `/login`).
- Settings live on the profile (`docs/ecrans.md` #10): menu Paramètres (pseudo, language — move
  `LanguageSwitcher` here —, theme, keyboard AZERTY/QWERTY reusing `useKeyboardLayout`), Règles,
  Se déconnecter. No settings modal.
- Remove the per-page headers that only held back arrows and the language switcher; keep a page
  title.

**Done when:** every route renders inside the shell at 390px and 1440px with no horizontal
scroll, the active entry is marked, the theme survives a reload without a flash, and the grid
looks the same in both themes.

### Task 2: Day figures — players, rank out of total, next daily

**Files:** `api/src/repositories/daily_repository.rs`, `api/src/services/daily_service.rs`,
`api/src/dto/daily_dto.rs`, `api/src/controllers/daily_controller.rs`, `api/tests/daily.rs`,
`front/app/services/daily.ts`, `front/app/_components/daily-board.tsx`

- `GET /daily/{date}/ranking` gains `total`: the number of ranked lines that day (on time and
  late), so the card reads « 54ᵉ / 312 ».
- `GET /daily/today` gains `next_in_seconds`: seconds until the next Paris midnight, computed by
  PostgreSQL like `today`.
- Front: the solved daily card shows rank / total, points (same formula as the API; read it from
  the ranking line, do not recompute) and « Prochaine dans 9 h 38 », counting down every minute
  from `next_in_seconds`; it refetches `/daily/today` when it reaches 0.

**Done when:** a test proves `total` counts late lines and excludes guests, `next_in_seconds` is
in `(0, 86400]`; the card counts down and flips to the new day at 0.

### Task 3: My dailies and the Streak

**Files:** `api/src/repositories/daily_repository.rs`, `api/src/services/daily_service.rs`,
`api/src/dto/daily_dto.rs`, `api/src/controllers/daily_controller.rs`, `api/tests/daily.rs`,
`CONTEXT.md`

New `GET /daily/mine` (any signed-in caller, guests included): every daily game of the caller,
newest first — `date`, `status`, `late`, `seconds`, `wrong_propositions`, `points` (null unless
solved) — plus `streak`.

- **Streak**: consecutive days, ending today or yesterday, whose daily was **solved on its own
  day**; a late solve or a gap breaks it. Compute it in the service from the list, as a pure
  function with a unit test (today unsolved but yesterday solved still counts the run).
- One row per day already holds (unique index), so the list is small: no pagination.

**Done when:** `tests/daily.rs` covers an empty history, a run of three, a run broken by a late
solve, and a run still alive while today is unplayed.

### Task 4: Dailies page — calendar and day detail

Adds the Quotidiennes entry (`CalendarDays`, `/daily`) to `components/nav/app-nav.tsx`; Task 1
left it out while `/daily` still redirected home.

**Files:** `front/app/daily/page.tsx`, `front/app/daily/_components/*` (new),
`front/app/daily/_viewmodels/use-dailies.ts` (new), `front/app/services/daily.ts`,
`front/app/_components/recent-days.tsx` (new)

Needs Task 3. Replace the redirect of `/daily` with the page of `docs/quotidiennes.md`.

- Month calendar with `‹ ›` navigation, bounded by `FIRST_DAY` and today; each day in one of the
  six states of the sheet, read from `GET /daily/mine`. `?date=YYYY-MM-DD` selects a day and its
  month.
- Day detail: not played (past) → `Rattraper ce jour` (reuses `playDaily`), with « Classée après
  les joueurs du jour »; in progress → `Reprendre`; solved → time, rank, points, `Voir ma partie`;
  plus that day's ranking through the existing `DayRanking` component.
- Header: dailies solved, Streak, total points.
- `RecentDays`: the last seven days as the same day states, each linking to `/daily?date=…`;
  rendered on `/` under the rankings. Remove the date input from `DailyBoard`: the calendar
  replaces it.

**Done when:** a signed-in player sees their solved, late, in-progress and unplayed days on the
calendar, catches up a past day from it, and the home page shows the seven-day strip.

### Task 5: Rankings — tabs, total, centred on me

**Files:** `front/app/ranking/page.tsx`, `front/app/ranking/_components/*`,
`front/app/_components/day-ranking.tsx`, `front/app/_components/home-rankings.tsx` (new)

- `/ranking` gets tabs `[Jour | Général]`; the Jour tab has its day picker and reads `?date=`;
  `?tab=day|global` selects the tab. Opening it scrolls my line into view.
- `home-rankings.tsx` on `/`: the day top 10 and the global top 10 side by side from `md`; below
  it, tabs with a top 5, each with the pinned « Toi » line and `Voir tout` to `/ranking?tab=…`.
- Ranking line variants of `docs/accueil.md`: podium marks for 1–3, highlighted me, late in the
  `late` colour; signed out or guest, a ghost line « Inscrivez-vous pour apparaître ici » linking
  to `/register`.

**Done when:** both rankings show on `/` at 390px and 1440px, `Voir mon rang` (Task 6) lands on
my line, and a guest sees the ghost line.

### Task 6: End of the daily — calls to action and Share

**Files:** `front/app/games/[id]/_components/game-over-panel.tsx`,
`front/app/games/[id]/_components/daily-actions.tsx` (new), `front/app/lib/share.ts` (new),
`front/app/_components/daily-board.tsx`

The solved daily panel, on the game page and on the home card:

- Primary `Voir mon rang` → `/ranking?tab=day&date=<date>`.
- Group « Continuer à jouer »: `Défier un ami` (opens a Race, same flow as Task 9's tile),
  `Lancer un défi` (Task 8), `Contre le bot` (`POST /games`, then the game page).
- `Partager` as an icon button; `Rattraper hier` as a link when yesterday's daily is unplayed
  (from `GET /daily/mine`, Task 3).
- A guest who solved sees `S'inscrire` instead of the rank; the other actions stay.

`share.ts` builds the text and hands it to `navigator.share` when it exists, the clipboard
otherwise, with the toast « Copié ». Exact text (lines joined by `\n`):

`Scout Beam {dd/mm} — {m:ss}{ (+N min) if wrong propositions}`
`{one emoji per shot, in firing order}`
`{n} tirs · {origin}`

Emoji per beam colour: none `⚪`, white `⬜`, red `🟥`, blue `🟦`, yellow `🟨`, violet `🟪`,
orange `🟧`, green `🟩`, black `⬛`. Unit-test the builder.

**Done when:** solving a daily shows the actions above, `Partager` produces the exact text, and
the builder's test passes.

### Task 7: Day ranking line of a named player

**Files:** `api/src/repositories/daily_repository.rs`, `api/src/services/daily_service.rs`,
`api/src/dto/daily_dto.rs`, `api/src/controllers/daily_controller.rs`, `api/tests/daily.rs`,
`front/app/services/daily.ts`

`GET /daily/{date}/ranking?player=<login>` adds `player`: that login's line that day, inside the
top or not, null when they are unranked. Same shape as `me`. Used by the Challenge (Task 8).

**Done when:** a test proves `player` is filled outside the top 100, null for an unknown login or
a guest, and absent without the parameter.

### Task 8: Challenge — « Bats mon temps »

**Files:** `front/app/daily/[date]/page.tsx` (new), `front/app/daily/[date]/_components/*` (new),
`front/app/lib/share.ts`, `front/app/games/[id]/_components/game-over-panel.tsx`, `CONTEXT.md`

Needs Tasks 6 and 7. A Challenge is a link, no table:
`{origin}/daily/{date}?defi={login}`.

- `Lancer un défi` (Task 6) shares the Task 6 text with the first line replaced by
  « Bats mon temps sur Scout Beam : {m:ss} » and the challenge link as the last line.
- Arrival page: reads the challenger's line with `?player=` and shows « {login} a fait {m:ss}
  sur la quotidienne du {dd/mm}. Bats son temps ! » with `Relever le défi` (reuses `playDaily`,
  guest login included). A past day says it will be ranked late. Already played: skip straight to
  the comparison. Unknown or unranked challenger: plain « Joue la quotidienne du {dd/mm} ».
- The game keeps the challenger in its URL (`/games/{id}?defi={login}`); the end panel adds
  « Tu as battu {login} de {m:ss} » or « {login} garde l'avantage (+{m:ss}) », comparing scores.

**Done when:** a second account opens a challenge link signed out, plays as a guest, and sees the
comparison on the end panel.

### Task 9: Jouer — the modes

**Files:** `front/app/page.tsx`, `front/app/play/page.tsx`, `front/app/play/_components/*`,
`front/app/services/rooms.ts`

`/play` (`docs/ecrans.md` #4, `docs/retours-maquettes-v1.md`): choice cards Quotidienne (with
its state), Contre le bot, Multijoueur. The home keeps the order of `docs/accueil.md` with a single
card linking to `/play` in place of the modes.

- Multijoueur opens Race and Duel tiles with `Défier un ami` — one click
  opens the room, copies the invitation link, toasts « Lien copié » and goes to `/rooms/{code}`;
  a Bot tile with `Jouer`; the 6-character code field with `Rejoindre` (move it from
  `room-actions.tsx`, reuse its validation).
- A guest sees the Race and Duel tiles with `Créer un compte pour défier` → `/register?next=/`.

**Done when:** from `/`, a registered player opens a Race in one click with the link in the
clipboard, joins by code, and starts a bot game; a guest is sent to sign up.

### Task 10: Mes parties

**Files:** `front/app/play/page.tsx`, `front/app/play/_components/*`

Needs Task 9: the list sits under the mode cards of `/play`.

- Section « Mes parties »; filters `[Toutes | En cours | Terminées]` kept in `?filter=`; waiting
  rooms first; a line shows mode, opponent, status, date.
- Empty list: `Jouer contre le bot` and a link to today's daily.

**Done when:** the three filters and the empty state work, and waiting rooms come first.

### Task 11: Rematch

**Files:** `api/migrations/0011_rematch.sql`, `api/src/models/room_model.rs`,
`api/src/repositories/room_repository.rs`, `api/src/services/room_service.rs`,
`api/src/errors/room_error.rs`, `api/src/dto/room_dto.rs`, `api/src/controllers/room_controller.rs`,
`api/tests/room.rs`, `front/app/services/rooms.ts`,
`front/app/games/[id]/_components/rematch.tsx` (new),
`front/app/games/[id]/_components/game-over-panel.tsx`, `CONTEXT.md`

Migration: on `room`, `invited_id UUID REFERENCES "user"` (only this user may join, null for an
open room) and `rematch_id UUID REFERENCES room ON DELETE SET NULL` (the room that replays this one; Task 12
deletes a cancelled rematch room).

- `POST /rooms/{code}/rematch` (either player of a `finished` room): opens a waiting room of the
  same mode hosted by the caller with the other player as `invited_id`, and links it as
  `rematch_id`. A second call by either player answers the existing rematch (200, not 201), so two
  crossed clicks meet in one room. 409 when the room is not finished.
- `POST /rooms/{code}/join` refuses anyone but `invited_id` when it is set (409, new
  `RoomError` variant).
- `GET /rooms/{code}` exposes `rematch_code` and who asked for it.
- Front: the end panel of a room game polls its room every 2 s (reuse the `use-room` polling
  pattern) while it is over. Asker: `Revanche` → « En attente de {login}… » and `Annuler`.
  Other player: « {login} veut une revanche » with `Accepter` (joins, then goes to the new room)
  and `Non merci` (hides the offer locally). The rematch room shows up in Mes parties as a waiting
  room for both.
- Secondary actions on that panel: `Voir les tracés`, `Contre le bot`, `Accueil`.

**Done when:** `tests/room.rs` proves a rematch reuses the mode, only the invited player can join,
crossed requests share one room, and a non-finished room refuses; in the browser, two accounts
chain two Races through `Revanche`.

### Task 12: Cancel a waiting room, and the bot while waiting

**Files:** `api/src/repositories/room_repository.rs`, `api/src/services/room_service.rs`,
`api/src/controllers/room_controller.rs`, `api/tests/room.rs`, `front/app/services/rooms.ts`,
`front/app/rooms/[code]/_components/*`

- `DELETE /rooms/{code}`: the host of a `waiting` room deletes it (204); anyone else 403, a room
  past waiting 409. Cancelling a Rematch room (Task 11) clears the old room's `rematch_id`
  (`ON DELETE SET NULL` in Task 11's migration).
- Waiting room screen: `Copier le lien` primary, `Partager` through `navigator.share` when it
  exists, `Annuler` (confirm dialog, then `/`), and the link « En attendant, jouer contre le bot »
  opening a bot game in a new tab.
- The invited player's view of a room: « {host} te défie en {mode} », the mode's rule in one line,
  `Rejoindre`.

**Done when:** the tests cover the three answers of `DELETE`, and a host cancels a waiting room
from its page.

### Task 13: Record against each opponent

**Files:** `api/src/repositories/game_repository.rs`, `api/src/services/game_service.rs`,
`api/src/dto/user_dto.rs`, `api/src/controllers/user_controller.rs`, `api/tests/user.rs`,
`front/app/services/users.ts`, `front/app/profile/_components/record.tsx` (new),
`front/app/play/_components/*`, `CONTEXT.md`

`GET /users/me/record`: per opponent login, `wins` and `losses` over finished rooms (a win is
`winner_id = me`), most played first, plus the totals. Mark it
`ponytail: wins and losses only; an Elo rating when 1v1 games are frequent enough to rate`.

- Profile: totals, the list « contre Bob : 3 V – 2 D », and the Streak (Task 3).
- Mes parties: the totals line above the list.

**Done when:** a test proves wins and losses per opponent across Race and Duel, a surrender
counted as a loss, and an empty record for a player without rooms.
