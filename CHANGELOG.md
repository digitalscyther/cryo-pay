# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[1.0.0]: https://github.com/digitalscyther/cryo-pay/releases/tag/v1.0.0
