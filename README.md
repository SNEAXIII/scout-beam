# Scout Beam

A solo deduction game: a bot hides pieces on a grid, the player fires beams from the edges and
deduces the hidden layout. Vocabulary: [`CONTEXT.md`](CONTEXT.md).

- `api/` — Rust: axum + sqlx on PostgreSQL, OpenAPI with utoipa.
- `front/` — Next.js App Router, React 19, Tailwind CSS 4, shadcn/ui.

## API

Everything below runs from `api/`. `make` on its own lists the targets.

```bash
cp .env.example .env
make tools     # once: sqlx-cli, cargo-llvm-cov, cargo-nextest
make db        # PostgreSQL in Docker: dev on 5436, throwaway test instance on 5437
make migrate   # apply pending migrations (idempotent)
make seed-user LOGIN=Sneaxiii            # create an account (ROLE=admin|super_admin to promote)
make dev       # http://localhost:8002, Swagger UI on /docs
```

Sign in, then call a protected route. The tokens travel only as `HttpOnly` cookies
(`scout_beam_access`, `scout_beam_refresh`), so keep them in a cookie jar:

```bash
curl -c jar.txt -X POST localhost:8002/auth/dev-login \
     -H 'content-type: application/json' -d '{"login":"Sneaxiii"}'

curl -b jar.txt localhost:8002/auth/session
curl -b jar.txt -c jar.txt -X POST localhost:8002/auth/refresh   # a fresh pair, from the refresh cookie
curl -b jar.txt -c jar.txt -X POST localhost:8002/auth/logout    # both cookies cleared
```

A tool may still send the access token as `Authorization: Bearer <token>`; it wins over the cookie.

`ENV` is required: `dev` or `prod`, any other value refuses to boot. `/auth/dev-login` exists only
with `ENV=dev`; an omitted login is generated server-side.

| Method | Route | Who |
| --- | --- | --- |
| POST | `/auth/register` | anyone — creates a password account and signs it in; from a guest's session, turns that guest into the account; 5 sign-in attempts a minute per IP |
| POST | `/auth/login` | anyone with a password account; same rate limit as `/auth/register` |
| POST | `/auth/guest` | anyone — creates a guest under a generated login and signs it in; same rate limit; a guest gets 403 on `POST /games` |
| POST | `/auth/dev-login` | anyone (dev builds only) — creates the account on first sight |
| GET | `/auth/providers` | anyone — whether dev-login is mounted, and the authorize URL of each enabled provider |
| POST | `/auth/discord` | anyone with a Discord code — finds, links or creates the account; a guest's session converts that guest; same rate limit |
| POST | `/auth/google` | same as `/auth/discord`, with a Google code |
| GET | `/auth/session` | any signed-in user |
| POST | `/auth/refresh` | holder of the refresh cookie — sets a fresh pair of cookies |
| POST | `/auth/logout` | anyone — clears both session cookies |
| GET | `/daily/today` | anyone — today's date in Paris, where the day changes at midnight |
| POST | `/daily/{date}/game` | any signed-in user, guests included — the caller's game of that day: 201 on the first call, the same game with 200 afterwards; today's layout is drawn on first sight, a past day without a daily or a day to come is 404; a daily cannot be surrendered (409) |
| GET | `/daily/{date}/ranking` | anyone — that day's top 100 by score (seconds to solve plus 60 per wrong proposition), games started that same day by registered players; with a session, `me` is the caller's line even outside the top |
| GET | `/ranking` | anyone — top 100 by points summed over every solved daily, late ones included, registered players only; same `me` |
| POST | `/rooms` | any registered user (guests get 403 on every room route) — opens a waiting room for a `race` or a `duel` under a 6-character code, also its invitation link `/rooms/<code>` |
| GET | `/rooms/{code}` | any registered user — its two players see both logins, the winner and their own game id; anyone else only the mode, status and host |
| POST | `/rooms/{code}/join` | any registered user but the host — takes the free seat (409 once full); a Race starts both games on one layout, the first correct proposition wins and the other game is `lost`, a surrender hands the win to the opponent |
| GET | `/health` | anyone |
| GET | `/docs` | anyone — Swagger UI |

**Discord and Google apps.** Create a Discord application (Developers portal, OAuth2) and a Google
OAuth client (Cloud console, "Web application"). Register as redirect URI the front origin followed
by `/login/callback/discord` or `/login/callback/google`, e.g.
`http://localhost:3001/login/callback/discord`. Put the id and secret in `DISCORD_CLIENT_*` /
`GOOGLE_CLIENT_*`, plus an `EMAIL_PEPPER` (a long random string) and `EMAIL_PEPPER_VERSION=1`. A
provider without both variables stays off. The email is never stored, only a peppered hash used to
link Discord and Google to the same account. In Swarm, create the three secrets of `stack.yaml`; a
provider you do not use still needs a placeholder secret.

`make check` runs format, clippy, the `.sqlx` cache check and the tests. A changed query or
migration needs `make prepare` in the same commit: the Docker image compiles against `.sqlx/`.

## Front

Everything below runs from `front/`, with the API up on `localhost:8002`.

```bash
cp .env.example .env.local   # NEXT_PUBLIC_API_URL, defaults to http://localhost:8002
pnpm install
pnpm dev                     # http://localhost:3001 — sign in on /login, sign up on /register
```

`/login` signs in with a login and password, offers a button per provider the API has configured
(`GET /auth/providers`), and shows the dev-login form only when the API runs with `ENV=dev`. A
provider button stores a random `state` for the session and sends the browser to the provider,
which comes back on `/login/callback/discord` or `/login/callback/google`: a missing or different
`state` is refused there before anything is sent. To try the providers locally, create the apps as
described above with the redirect URIs on `http://localhost:3001`. Signing up or signing in with a
provider from a guest session turns that guest into the account.

`pnpm exec tsc --noEmit` and `pnpm exec oxlint --type-aware` check a change; `pnpm build` builds it.
The API's `CORS_ALLOWED_ORIGIN` must be `http://localhost:3001`.

## Deploy

`stack.yaml` is a Swarm stack that plugs into the Traefik of the Mawster stack (network
`traefik-public`, resolver `letsencrypt`). The front is served on `SCOUT_BEAM_HOST` and the API
under `/api` on the same host. Create the secrets once, on the manager:

```bash
openssl rand -hex 32 | docker secret create scout_beam_db_password -
openssl rand -hex 32 | docker secret create scout_beam_jwt_secret -
make deploy SCOUT_BEAM_HOST=scout-beam.example.com   # builds both images locally, then deploys
```
