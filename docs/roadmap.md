# Roadmap

Improvement plan based on full codebase analysis, organized by priority.

---

## Phase 1 — Critical Security & Reliability

Must-fix before scaling. These are production blockers.

### Security
- [ ] **SSL/TLS in Nginx** — currently HTTP only, no encryption in transit
- [ ] **Fix CORS config** — `Access-Control-Allow-Origin: *` combined with `Access-Control-Allow-Credentials: true` violates the CORS spec and is insecure. Restrict to known origins
- [ ] **Add Nginx security headers** — `X-Frame-Options`, `X-Content-Type-Options`, `Strict-Transport-Security`, `Content-Security-Policy`
- [ ] **Fix cookie security** — `SameSite=None` in `api/src/api/auth/mod.rs:64` disables CSRF protection. Use `SameSite=Lax` or `Strict` with proper domain config
- [ ] **Remove hardcoded credentials** from `docker-compose.yml` — Postgres password `example`, Redis password `redis123` should use GitHub Secrets (like `FIREBASE_CLIENT_CONFIG`, `PROJECT_NAME` in `build.yml`) injected via the infra deploy workflow into `.env` on the VPS
- [ ] **Webhook signature verification** — outgoing webhooks (`monitoring/app_state.rs`) have no HMAC signature, so receivers can't verify authenticity
- [ ] **SSRF prevention** — webhook URL validation (`api/user/webhook.rs`) doesn't block internal IPs

### Reliability
- [ ] **Database backup strategy** — no backup mechanism exists for the `postgres_data` Docker volume
- [ ] **Blockchain daemon resilience** — `monitoring/daemon.rs:202-238` runs an infinite loop with no supervisor restart, no health signal to the main process, and silently swallows errors (line 220-224). If it stops, payments go undetected
- [ ] **Add health checks** — no `/health` endpoint checking DB/Redis connectivity; no Docker healthcheck directives

---

## Phase 2 — Testing & Error Handling

The codebase has essentially zero test coverage for business logic.

### Testing
- [ ] **Backend integration tests** — payment flow (create invoice → on-chain event → mark paid → notify), auth flow, API key auth
- [ ] **Backend unit tests** — subscription pricing (`payments/payable.rs`), rate limiting logic, JWT claims, network config parsing
- [ ] **Frontend component tests** — invoice creation, payment flow, settings CRUD
- [ ] **Add tests to CI** — `build.yml` currently only builds Docker images, no test step

### Error Handling
- [ ] **Stop silencing errors in daemon** — `monitoring/daemon.rs:220` converts RPC errors to empty vec; should retry with backoff and alert on repeated failure
- [ ] **Fix `unwrap_or_default()` on `paid_at`** — `events/notifications/mod.rs:131` sends epoch time (1970-01-01) instead of actual payment time when field is None
- [ ] **Add retry logic** for external API calls — Infura RPC, Brevo email, Telegram, webhook delivery. Currently all fire-and-forget
- [ ] **Input validation** — no checks for negative amounts (`api/buy/donation.rs:36`), invalid network IDs (`api/payments/mod.rs:119`), or negative subscription days

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
