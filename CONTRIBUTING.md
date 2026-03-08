# Contributing to Cryo Pay

## Getting Started

Clone the repo and start the full stack with Docker Compose:

```bash
git clone https://github.com/digitalscyther/cryo-pay && cd cryo-pay
docker compose build
NGINX_PORT=80 POSTGRES_PORT=6432 REDIS_PORT=6381 \
  docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
```

See [README](readme.md) for Firebase credentials setup and the full `api/.env` reference.

## Dev Setup

**Required environment variables** (in `api/.env`):

| Variable | Purpose |
|---|---|
| `POSTGRES_URL` | PostgreSQL connection string |
| `REDIS_URL` | Redis connection string |
| `APP_SECRET` | JWT signing secret |
| `INFURA_TOKEN` | Infura RPC access for EVM chains |
| `TGBOT_TOKEN` | Telegram bot token for notifications |

**Firebase credentials** (gitignored — create manually):
- `api/data/firebaseConfig.json` — service account private key
- `web/src/firebaseConfig.json` — web app config

**Build the Rust API without a live database:**

```bash
cd api
SQLX_OFFLINE=true cargo build
```

## Running Tests

```bash
# Rust unit tests (no DB required)
cd api && cargo test

# Integration tests require a live DATABASE_URL; they are marked #[ignore] by default
DATABASE_URL=postgres://... cargo test -- --ignored

# React frontend tests
cd web && npm test
```

## SQL Query Changes

After editing any `sqlx::query*!` macro, regenerate the `.sqlx/` query cache against a live database:

```bash
cd api
DATABASE_URL=postgres://cryo:example@localhost:6432/cryo cargo sqlx prepare
```

Always commit the updated `.sqlx/` directory. It is required for `SQLX_OFFLINE=true` builds (used in Docker and CI).

## Code Style

```bash
# Rust
cargo fmt
cargo clippy

# React — CRA default ESLint config, enforced by the build
cd web && npm run build
```

Fix all `clippy` warnings and ESLint errors before opening a PR.

## Submitting PRs

1. Describe what the change does and why. Link to a [roadmap item](docs/roadmap.md) if applicable.
2. Verify these pass locally before submitting:
   ```bash
   cd api  && SQLX_OFFLINE=true cargo build
   cd web  && npm run build
   ```
3. Keep commits focused. One logical change per PR is preferred.

## Architecture

See [docs/architecture.md](docs/architecture.md) for the module map, database schema, and auth/payment flows.

## Reporting Issues

Open a [GitHub Issue](https://github.com/digitalscyther/cryo-pay/issues). This is a solo project — response time is best-effort.
