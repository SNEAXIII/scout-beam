# Handoff — 2026-11-15

Four features were grilled together and are built one branch and one PR at a time, in this order:
auth → home page → daily → rooms (Race and Duel). Vocabulary: `CONTEXT.md`. Conventions:
`CLAUDE.md`. Guests: `docs/adr/0001-guests-are-user-rows.md`.

## State

| Feature | Plan | State |
| --- | --- | --- |
| Auth | `docs/plans/2026-11-14-auth.md` | merged, PR #10 |
| Home page | `docs/plans/2026-11-15-home-page.md` | merged, PR #11 |
| Daily | `docs/plans/2026-11-15-daily.md` | **in progress** on `feat/daily` |
| Rooms | `docs/plans/2026-11-15-duel-race.md` | planned, starts after the daily is merged |

On `feat/daily`, Tasks 1 and 2 (API) are committed; **Task 3 (playing the daily, front)** was
handed to an implementer subagent when this was last updated: its work may sit uncommitted in the
tree. Check `git status` and finish or review it before starting Task 4.

## How the work runs

The `flow:feature` loop, with these settings chosen by the user:

- a branch in the main checkout (no worktree), `<type>/<slug>` from `main`;
- **never push without asking**; small fixes on `main` are sometimes asked as `push def`;
- no `ponytail-review` at the end.

Per task: an implementer subagent (implement, run the checks, no commit) → a reviewer subagent on
the diff ("OK" or a list) → fixes back to the implementer → `git:commit`. One progress line per
task, then a summary with the deviations, then `git:make-pr` on the go-ahead.

Parallel work only when files do not overlap, in a worktree with `front/node_modules` linked by a
junction (`mklink /J`), removed with `rmdir` before the worktree. Agent worktrees land in
`.claude/worktrees/` (untracked).

## Decisions not written elsewhere

- **Daily**: everything is in its plan. Score = time + 60 s per wrong proposition, displayed as raw
  time with the penalty beside it ("12:34 + 2 min"); shots counted but not shown yet. Points
  `max(100, 1000 - floor(score/6))`, global ranking = sum of points, late dailies included; the day
  ranking only counts games played on their day. No surrender. A past day can be replayed only if
  someone played it on its day (the daily is drawn lazily).
- **Rooms**: everything is in their plan. Duel priority: after an action, the same player waits
  until the opponent acts or `10 s × fired / 36`, capped at 10 s by construction.
- **Auth, kept for later**: whether to store the email in clear is undecided (hash only for now);
  no account linking between password accounts and providers; logout does not revoke tokens
  server-side; v1 accounts made by dev-login are abandoned (no credentials, cannot sign in).

## Operations pending on the user's side

- Deploy needs the Docker secrets `scout_beam_email_pepper`, `scout_beam_discord_client_secret`
  (the real one) and `scout_beam_google_client_secret` (a placeholder until Google is set up).
- Discord redirect URIs: `https://scout-beam.mawster.app/login/callback/discord` and
  `http://localhost:3001/login/callback/discord`. `DISCORD_CLIENT_ID` is in the root `Makefile`;
  `GOOGLE_CLIENT_ID` is still empty.
- The local `api/.env` predates the provider variables: copy them from `api/.env.example`.
- Pepper rotation: `make down`, recreate the secret, `EMAIL_PEPPER_VERSION=2 make deploy`.

## Gotchas met

- CI uses the **latest stable rustc**, the local one may be older: new clippy lints break CI only
  (e.g. 1.100's `map_unwrap_or`). Prefer `map_or_else`.
- A running `make dev` locks `target\debug\scout-beam.exe`: `make check` then fails to relink.
- BoardGameGeek answers 403 to scripts (Cloudflare): a link there is checked in a browser.
- `stack.yaml` uses `update_config.failure_action: continue` because the API exits while
  PostgreSQL is not ready; making the API wait for the database would allow the safer default.
- Known small debts: the register response shows `last_login_date: null`; `tests/openapi.rs`
  does not list the auth routes; `en.ts`/`fr.ts` exceed 150 lines (accepted as data); the pieces
  animation on `/` plays once on load.
