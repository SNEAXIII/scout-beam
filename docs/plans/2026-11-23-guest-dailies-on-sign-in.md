# Guest dailies on sign-in

**Goal:** a guest who signs in to an existing account hands that account its dailies of days the
account has not played, so a daily seen as a guest can no longer be replayed clean on the account.

## Global Constraints

- Backend through `make` only; a new or changed query needs `make prepare` in the same commit;
  `make check` passes.
- Layering from `CLAUDE.md`: SQL in `repositories/`, rules in `services/`, controllers call services.
- A moved game keeps its `created_at`, `solved_at`, shots and propositions: only `user_id` changes.
- On a day the account already has a game, the guest's game stays on the guest, untouched.
- The emptied guest user is left as is, never deleted.
- Glossary already updated (`CONTEXT.md`, **Guest**): no other doc change.

---

### Task 1: Move the guest's dailies when signing in to an existing account

**Files:** `api/src/repositories/game_repository.rs`, `api/src/services/user_service.rs`,
`api/src/controllers/user_controller.rs`, `api/tests/user.rs` (or `api/tests/daily.rs`, whichever
already tests guests), `api/.sqlx/`

Repository: `game_repository::move_guest_dailies(pool, guest_id, account_id)`, a single `UPDATE`
that sets `user_id` to the account on every game of `guest_id` with a `daily_date` the account has
no game for, and only when `guest_id` is a guest user (`"user".guest` in the same statement, so a
real account's token can never be drained). Returns the number of rows moved.

Service: `user_service::take_guest_dailies(pool, caller: Option<Uuid>, account: &User)`. Does
nothing when `caller` is `None` or equals `account.id` (register and a converting OAuth sign-in
turn the guest into the account itself). A unique violation on `game_user_id_daily_date_unique`
(the account started that day's daily in the same instant) is swallowed: the sign-in must not
fail for it; reuse the `is_unique_violation` helper already used by `oauth_service`.

Controllers: `post_login` takes `caller: Option<Identity>` like `post_register`, and adds
`security((), ("bearer" = []))` to its `#[utoipa::path]`; after `login_with_password`, call
`take_guest_dailies`. `provider_sign_in` calls it after `oauth_service::sign_in` returns. Update
the doc comment of `provider_sign_in` and `post_login` to say a guest's dailies follow.

**Done when:** `#[sqlx::test]` integration tests prove, through the HTTP routes:
a guest solves a daily, then `POST /auth/login` with the guest's cookie → the account's day
ranking lists the account with the guest's time, and `GET` of that daily for the account returns
the moved game (not a new one); a guest whose day the account already played → the account's game
is unchanged and the guest keeps its own; an in-progress guest daily moves too; signing in with a
non-guest account's cookie moves nothing. `make check` is green.

### Task 2: Send the guest session along with a password login

**Files:** `front/app/services/auth.ts`

`login` goes through `convertingSignIn` instead of `signIn`, like `register` and `oauthLogin`, so
an expired guest access cookie is renewed first and `POST /auth/login` sees the guest
(the API endpoint now reads it, see Task 1's `take_guest_dailies`). Rename nothing else; adjust
the comment above `convertingSignIn` so it covers "a guest's dailies follow" as well as conversion
(2 lines max).

**Done when:** `pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware` pass, and in the app:
play the daily as a guest, sign in with an existing password account that has not played it → the
home page shows that day's game as the account's, and the day ranking lists the account.
