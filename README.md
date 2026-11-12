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

Sign in, then call a protected route:

```bash
curl -X POST localhost:8002/auth/dev-login \
     -H 'content-type: application/json' -d '{"login":"Sneaxiii"}'

curl localhost:8002/auth/session -H "authorization: Bearer $TOKEN"
```

`ENV` is required: `dev` or `prod`, any other value refuses to boot. `/auth/dev-login` exists only
with `ENV=dev`; an omitted login is generated server-side.

| Method | Route | Who |
| --- | --- | --- |
| POST | `/auth/dev-login` | anyone (dev builds only) — creates the account on first sight |
| GET | `/auth/session` | any signed-in user |
| POST | `/auth/refresh` | holder of a refresh token |
| GET | `/health` | anyone |
| GET | `/docs` | anyone — Swagger UI |

`make check` runs format, clippy, the `.sqlx` cache check and the tests. A changed query or
migration needs `make prepare` in the same commit: the Docker image compiles against `.sqlx/`.

## Front

Everything below runs from `front/`, with the API up on `localhost:8002`.

```bash
cp .env.example .env.local   # NEXT_PUBLIC_API_URL, defaults to http://localhost:8002
pnpm install
pnpm dev                     # http://localhost:3001 — sign in on /login: 3–30 ASCII letters or digits
```

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
