# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Cryo Pay is a crypto payment gateway for USDT (ERC-20) invoices on EVM chains (Optimism, Arbitrum). Rust backend + React frontend, deployed via Docker Compose with Nginx, PostgreSQL, and Redis.

See [docs/architecture.md](docs/architecture.md) for detailed architecture, module map, database schema, and auth/payment flows.
See [docs/roadmap.md](docs/roadmap.md) for improvement plan and known issues.

### Quick orientation
- **`api/`** — Rust (Axum) backend. Single binary runs 3 concurrent tasks: HTTP API, blockchain monitor daemon, Telegram bot
- **`web/`** — React 18 frontend (CRA + Bootstrap + Web3.js + Firebase auth)
- **Nginx** reverse proxy: `/api/` → backend :8080, `/` → frontend :3000

## Common Commands

### Full stack (Docker Compose)
```bash
docker compose build && NGINX_PORT=80 POSTGRES_PORT=6432 REDIS_PORT=6381 docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
```

### Backend (Rust API)
```bash
cd api
cargo build                    # build
cargo build --release          # release build
cargo test                     # run tests
SQLX_OFFLINE=true cargo build  # build without live DB (uses .sqlx/ cache)
```

### Frontend (React)
```bash
cd web
npm install
npm start                      # dev server
npm run build                  # production build
npm test                       # run tests
```

Local frontend with custom API URL:
```bash
REACT_APP_BASE_API_URL=http://127.0.0.1:3001 REACT_APP_PROJECT_NAME=LOCALTest REACT_APP_CONTACTS='{"email":"foo@bar.baz","telegram":"foo","linkedin":"foo"}' npm start
```

### Database migrations
```bash
DATABASE_URL=postgres://cryo:example@localhost:6432/cryo sqlx migrate add -r <name>
```

## Important Configuration

- Backend config is via environment variables (see `api/.env`). Key vars: `POSTGRES_URL`, `REDIS_URL`, `APP_SECRET`, `NETWORKS` (JSON array of chain configs), `INFURA_TOKEN`, `TGBOT_TOKEN`, `BREVO_API_KEY`
- Firebase credentials: `api/data/firebaseConfig.json` (private key) and `web/src/firebaseConfig.json` — both gitignored
- Smart contract ABIs: `api/data/invoice_abi.json`, `api/data/erc20_abi.json`
- SQLx offline mode (`SQLX_OFFLINE=true`) is used in Docker builds; regenerate `.sqlx/` cache with `cargo sqlx prepare` when queries change
- Rust version: 1.92+ (set in `api/Dockerfile`)

## Deployment

### Pipeline
- **Build** (`cryo-pay/.github/workflows/build.yml`): push to master → GitHub Actions builds API + Web Docker images → pushes to `ghcr.io/digitalscyther/cryo-pay-{api,web}:latest`
- **Deploy** (`infra/.github/workflows/deploy-cryo-pay.yml`): triggers on compose file changes in infra repo → rsyncs compose files → writes `.env` from `CRYO_PAY_ENV` GitHub Secret → restarts services
- **Watchtower** on VPS auto-pulls new images from GHCR

### Fast local deploy (skip Actions for API)
Build and push the Rust API image locally to avoid slow CI builds:
```bash
docker build -t ghcr.io/digitalscyther/cryo-pay-api:latest --target final api/
docker push ghcr.io/digitalscyther/cryo-pay-api:latest
ssh root@<VPS> "cd /opt/services/cryo-pay && docker compose pull api && docker compose up -d --force-recreate api"
```
Requires GHCR login: `echo "<PAT>" | docker login ghcr.io -u digitalscyther --password-stdin`

### Releases
- `CHANGELOG.md` at repo root follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format — update it before tagging
- To publish a release:
  ```bash
  git tag vX.Y.Z && git push origin vX.Y.Z
  gh release create vX.Y.Z --title "vX.Y.Z — <title>" --notes-file CHANGELOG.md
  ```

### Credentials
- Production `.env` is managed via `CRYO_PAY_ENV` GitHub Secret in the infra repo
- To rotate credentials: update the GitHub Secret, then redeploy (or SSH in and update `.env` directly)
