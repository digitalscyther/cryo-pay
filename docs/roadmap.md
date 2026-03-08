# Roadmap

Improvement plan based on full codebase analysis, organized by priority.

---

## Phase 1 — Critical Security & Reliability

Must-fix before scaling. These are production blockers.

### Security
- [x] **SSL/TLS in Nginx** — TLS terminated at Traefik via docker-compose labels + external `proxy` network. CSP header deferred (needs frontend audit for Bootstrap/Web3.js/Firebase inline styles)
- [x] **Fix CORS config** — removed Nginx CORS headers; added proper `CorsLayer` with `AllowOrigin::mirror_request()` in Rust backend
- [x] **Add Nginx security headers** — added X-Frame-Options, X-Content-Type-Options, Referrer-Policy, X-XSS-Protection to Nginx; HSTS set at Traefik edge only
- [x] **Fix cookie security** — `SameSite=Lax`, `HttpOnly=true`, `Secure=true` on JWT cookies
- [x] **Remove hardcoded credentials** — replaced with `${VAR}` substitution in docker-compose.yml; dev defaults in docker-compose.dev.yml; added `.env.example`
- [x] **Webhook signature verification** — HMAC-SHA256 signing with per-webhook secret, `X-Signature-256` + `X-Webhook-Timestamp` headers; backwards compatible (empty secret = no signature)
- [x] **SSRF prevention** — blocks private IPs, loopback, link-local, Docker-internal hostnames before making HTTP requests; DNS rebinding not covered (requires custom resolver)

### Reliability
- [x] **Database backup strategy** — added `prodrigestivill/postgres-backup-local:17-alpine` sidecar with daily schedule, 7d/4w/6m retention
- [x] **Blockchain daemon resilience** — added `DaemonHealth` shared state (atomics) between daemon and API; consecutive error tracking with exponential backoff (5s–60s); daemon marked unhealthy after 10 consecutive failures; 5s sleep between polling cycles
- [x] **Add health checks** — `/health` endpoint checks PostgreSQL (SELECT 1), Redis (PING), and daemon liveness (heartbeat + staleness); returns 200/503 with JSON status; `/ping` kept for simple liveness

---

## Phase 2 — Testing & Error Handling

The codebase has essentially zero test coverage for business logic.

### Testing
- [ ] **Backend integration tests** — payment flow (create invoice → on-chain event → mark paid → notify), auth flow, API key auth. Deferred: requires Docker test infrastructure (PostgreSQL + Redis)
- [x] **Backend unit tests** — added `#[cfg(test)]` modules with `rstest` across 6 files: payable pricing, subscription validation, network JSON parsing, rate limiting, utils, daemon health. 41 tests total
- [ ] **Frontend component tests** — invoice creation, payment flow, settings CRUD. Deferred: needs axios/Firebase mocking infrastructure
- [x] **Frontend utility tests** — `utils.test.js` (13 tests) and `firebase.test.js` (8 tests); fixed `sortNetworkItems` bug (returned function instead of comparison)
- [x] **Add tests to CI** — added `test-api` and `test-web` jobs to `build.yml`; Docker builds gated on test success via `needs:`

### Error Handling
- [x] **Stop silencing errors in daemon** — replaced silent `unwrap_or_else` with explicit error handling: logs error with network name and consecutive count, sleeps with exponential backoff, marks daemon unhealthy after threshold
- [x] **Fix `unwrap_or_default()` on `paid_at`** — changed `InvoicePaidNotification.paid_at` to `Option<NaiveDateTime>`; no more fake epoch timestamp in webhook payloads
- [x] **Add retry logic** — generic `retry()` helper with exponential backoff (1s–8s) and `tracing::warn!` logging; applied to webhook delivery (3 attempts), Brevo email (3 attempts), Telegram notifications (2 attempts), and gas fee API (2 attempts). Infura RPC skipped (already has daemon-level backoff)
- [x] **Input validation** — positive amount checks on invoice creation and donations; network ID validation against configured networks; non-empty networks list; zero-day subscription rejection

---

## Phase 3 — Observability & Operations

### Monitoring
- [ ] **Structured logging** — `main.rs` sets ERROR level only; add request IDs, correlation IDs, structured fields
- [ ] **Metrics** — add Prometheus endpoint for request latency, payment success/failure rates, daemon block lag, Redis/DB health
- [ ] **Error tracking** — integrate Sentry (or similar) in both backend and frontend
- [ ] **Alerting** — daemon stopped processing blocks, external API failures, high error rate

### CI/CD
- [ ] **Add image security scanning** in CI (Trivy/Snyk)
- [ ] **Semantic versioning** — currently only tags `latest` + git SHA
- [ ] **Deployment automation** — no deploy step exists after build
- [ ] **Add Nginx rate limiting** — no request rate limits at proxy level

---

## Phase 4 — Code Quality & DX

### Backend
- [ ] **Split large modules** — `db/mod.rs` (641 lines) should be split by domain (invoices, users, api_keys, etc.)
- [ ] **Separate `api/state.rs`** — mixes DB, JWT, Firebase, Redis setup in one file
- [ ] **Custom error enum** — replace `Result<T, String>` with proper domain error types
- [ ] **Make limits configurable** — currently hardcoded: API key limit (5), webhook limit (2), callback URL limit (5), rate limits (3-10/day), subscription pricing, JWT expiry (7 days), gas fee cache TTL (10 min)

### Frontend
- [ ] **Extract custom hooks** — `useFetch`, `useCrudList` to eliminate duplicate state/fetch/error patterns across ApiKeys, Webhooks, CallbackUrls, DonationList
- [ ] **Add axios interceptor** — centralized error handling, token refresh, consistent error UI
- [ ] **Auth context** — replace cookie checks scattered across components with React Context
- [ ] **Split large components** — `CreateInvoice.js` (286 lines), `Controller.js` (166 lines), `Documentation.js`
- [ ] **Consider CRA migration** — Create React App is deprecated; evaluate Vite

---

## Phase 5 — Product Features

### UX Improvements
- [ ] **Mobile responsive design** — minimal responsive breakpoints, modals likely break on mobile
- [ ] **Consistent loading/error states** — some components use Spinner, some use disabled buttons, no skeleton loaders
- [ ] **Better MetaMask error messages** — distinguish "user rejected" vs "contract revert" vs "network error"
- [ ] **Toast notifications** instead of inline alerts for transient feedback

### Accessibility
- [ ] **ARIA labels** on icon buttons, form inputs
- [ ] **Keyboard navigation** support
- [ ] **Color contrast** verification (WCAG 2.1 AA)

### Internationalization
- [ ] **i18n framework** (react-i18next) — all strings currently hardcoded in English

### Business Features (from readme TODO)
- [ ] API documentation (OpenAPI/Swagger)
- [ ] Webhook documentation
- [ ] Instant blockchain verification (high-priority queue)
- [ ] Redirect on pay — if subscription, check blockchain instantly
- [ ] Basic seller analytics (transaction counts, totals by period)
- [ ] QR code generation for invoices
- [ ] Landing page with service description
- [ ] Reset Firebase first token after logout
- [ ] Email notifications subscription management
- [ ] Bulk invoice creation
- [ ] Embeddable payment widget (HTML button for seller websites)
- [ ] Payment history export (CSV/PDF)
- [ ] Customizable notification frequency (immediate vs daily digest)
- [ ] Privacy policy and terms of use
- [ ] Custom token/blockchain support (waitlist)
- [ ] White-label solution
- [ ] Advanced sales analytics

---

## Database Improvements

- [ ] **Add indexes** on commonly queried columns (`user_id`, `created_at`, `paid_at` in invoices)
- [ ] **Audit logging table** — track sensitive operations (payment status changes, API key creation/deletion)
- [ ] **Connection pool tuning** — no pool configuration visible, relies on sqlx defaults
