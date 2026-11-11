# Scout Beam

A solo deduction game: a bot hides pieces on a grid, the player fires beams from the edges and
deduces the hidden layout. Vocabulary: [`CONTEXT.md`](CONTEXT.md).

- `api/` — Rust: axum + sqlx on PostgreSQL, OpenAPI with utoipa.

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
