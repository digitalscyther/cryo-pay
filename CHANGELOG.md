# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] - 2026-03-09

### Added
- OpenAPI spec (utoipa) with Swagger UI at `/swagger-ui/` and JSON at `/api-docs/openapi.json`
- "API Docs" navbar link and interactive API Reference section in `/docs` linking to Swagger UI
- Seller analytics endpoint `GET /user/analytics?days=30` — daily breakdown + summary; React dashboard component
- Docker healthchecks on all 6 services with `condition: service_healthy` dependency ordering; `wget` added to API image
- `AppError` enum (thiserror) — all DB/Redis wrappers migrated; `From<AppError> for ResponseError`
- Redis rate limit atomicity via Lua script; fixed TTL-reset bug (EXPIRE was unconditional)

### Changed
- `db/mod.rs` (600+ lines) split into 6 domain sub-modules: invoice, blockchain, user, api_key, callback_url, webhook
- `payments/` split into `donation` and `subscription` sub-modules; `payments.paid_at` sync fixed
- Daemon block range corrected: `from_block(last_block_number + 1)` to avoid re-scanning last block
- Firebase JWK cached behind `RwLock`; refreshed on error instead of fetching on every login
- PostgreSQL `max_connections` lowered from 2000 to 50 (sqlx pool default is 10)
- Nginx: added `limit_req_zone`, `client_max_body_size 64k`, `proxy_read_timeout 30s`
- Static `ApiEndpoints` component replaced by Swagger UI link (auth scope note + networks table retained)

### Fixed
- Docker build: added `curl` to builder image so `utoipa-swagger-ui` can download Swagger UI assets at compile time

### Docs
- Roadmap cleaned up: completed items removed, future work kept; Project Status doc added
- CONTRIBUTING.md, CLAUDE.md, and README links updated

## [1.0.0] - 2026-03-08

### Security
- Protect internal webhook/callback endpoints with derived HMAC token (`sha256("internal:" + APP_SECRET)`)
- Fix CORS: replace wildcard `mirror_request()` with explicit `WEB_BASE_URL` allowlist
- Add ownership check on `POST /buy/payment/:id/recheck` to prevent arbitrary invoice activation
- Validate `paid_amount >= invoice.amount` in daemon before marking invoices paid
- SSRF prevention: block private, loopback, and Docker-internal hostnames on webhook URLs
- Webhook delivery includes HMAC-SHA256 `X-Signature` header for receiver verification
- TLS via Traefik reverse proxy; secure cookie flags (`SameSite=Lax`, `HttpOnly`, `Secure`)

### Added
- `/health` endpoint reporting daemon liveness and last-seen timestamp
- Daemon health monitoring (`monitoring::health::DaemonHealth`) shared across concurrent tasks
- QR code fallback on invoice page for mobile users without MetaMask (`qrcode.react`)
- One-click copy-to-clipboard button for invoice payment links
- Webhook documentation with full payload examples and HMAC verification code (Python + Node.js)
- Subscription and donation endpoints (backend + frontend)
- Rate limiting middleware on API routes
- CI pipeline: `test-api`, `test-web`, `test-integration` jobs via GitHub Actions
- 41 backend unit tests across 6 modules; integration tests for invoice lifecycle and API key CRUD
- Frontend component tests (ApiKeys, Webhooks) and utility tests (utils, firebase)

### Changed
- Homepage: replaced fake testimonials with "Built in the Open" section (Rust Backend, On-Chain Verification, Open Source)
- Homepage: new hero copy — "Create Crypto Invoices Instantly / No Fees, No Sign-Up, Just a Link"
- Homepage: added "For Developers" CTA section
- Donation page: rewrote from generic template to authentic solo-developer message
- README: added badges (Rust, React, Docker, License), 5-step quickstart, documentation links, and full `api/.env` reference
- Log level upgraded from `ERROR` to `INFO` in main tracing subscriber (daemon logs now visible)
- Invoice UI: removed `alert()` popups, replaced with inline feedback and error states

### Removed
- Fake "Over 100 Invoices Created!" counter from homepage
- Hardcoded internal credentials

### Fixed
- Mobile layout: YouTube embed now uses `ratio ratio-16x9` Bootstrap wrapper
- MetaMask detection: shows QR code and helpful prompt when MetaMask is absent

[1.1.0]: https://github.com/digitalscyther/cryo-pay/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/digitalscyther/cryo-pay/releases/tag/v1.0.0
