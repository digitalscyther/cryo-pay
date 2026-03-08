# Cryo Pay — Project Status

All planned improvements have been implemented. This document summarizes what was done and what remains optional.

## Completed

**Security**
- CORS fixed: explicit origin allowlist (`WEB_BASE_URL`) instead of `mirror_request()`
- Internal webhook/callback endpoints authenticated with derived token (`sha256("internal:"+APP_SECRET)`)
- `/buy/payment/:id/recheck` authenticated + ownership check added
- Paid amount validated against invoice amount before marking paid
- Nginx: rate limiting, `client_max_body_size 64k`, `proxy_read_timeout 30s`
- Input validation: positive amounts, valid network IDs, non-empty networks, no zero-day subscriptions

**Reliability & Ops**
- Daemon resilience: exponential backoff (5s–60s), unhealthy after 10 consecutive failures
- Docker healthchecks on all 6 services with `condition: service_healthy` deps
- Firebase JWK cached behind `RwLock`, refreshed on error
- Daemon off-by-one fixed (`last_block + 1`)
- PostgreSQL `max_connections` lowered to 50
- Log level fixed from `ERROR` to `INFO`
- Redis rate limit atomicity via Lua script; TTL-reset bug fixed
- Retry logic (exponential backoff) on webhook delivery, email, Telegram, gas fee API
- DB backup sidecar with 7d/4w/6m retention

**Testing**
- 41 Rust unit tests across 6 modules (rstest)
- Backend integration tests: invoice lifecycle, user idempotency, API key CRUD
- Frontend component tests: ApiKeys, Webhooks (jest + React Testing Library)
- Frontend utility tests: utils (13), firebase (8); fixed `sortNetworkItems` bug
- CI: `test-api` and `test-web` jobs gate Docker builds

**Code Quality**
- `AppError` enum (thiserror); all DB/Redis wrappers migrated
- `db/mod.rs` split into 6 domain sub-modules (invoice, blockchain, user, api_key, callback_url, webhook)
- OpenAPI spec (utoipa) with Swagger UI at `/swagger-ui/`
- `unwrap_or_default()` on `paid_at` fixed — now `Option<NaiveDateTime>`

**UX & Frontend**
- Fake testimonials and "100 Invoices Created!" removed
- "Analytics and Reporting" replaced with real "Webhook Notifications" card
- QR code on invoice page (encodes payment link, not wallet address)
- "Copy payment link" button on invoice and after creation
- YouTube iframe responsive layout fixed
- DonationPage copy rewritten (personal, honest)
- `alert()` in Controller.js removed
- "Anonymous" typo fixed

**Documentation**
- README rewritten as product pitch with Mermaid payment flow diagram
- Webhook payload and HMAC verification documented in frontend
- All API endpoints documented (including `/user/api_key`, `/user/webhook`, `/user/callback_url`)
- CONTRIBUTING.md added
- CHANGELOG.md added; v1.0.0 released

**OSS / Repo**
- MIT LICENSE added
- GitHub topics added
- Mermaid payment flow diagram in README

---

## Optional / Skipped

| Item | Reason |
|------|--------|
| Off-site backup sync (rclone/S3) | Local-only backups are fine for personal use |
| UptimeRobot on `/health` | Operator action — configure at uptimerobot.com |
| Structured logging / Prometheus metrics | Nice-to-have, no active need |
| Error tracking (Sentry) | No external users |
| CSP headers | Requires frontend audit (Firebase + Bootstrap + Web3.js inline styles) |
| WalletConnect / wagmi+viem | 20+ hours; QR fallback covers non-MetaMask users |
| i18n | Zero users outside owner |
| CRA → Vite migration | Functional; pure churn |
| Bulk invoice / CSV export / widget | No users at that volume |
| White-label / custom tokens | Waitlist features |
