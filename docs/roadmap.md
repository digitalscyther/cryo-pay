# Roadmap

Future work, organized by priority. See [CHANGELOG](../CHANGELOG.md) for what is already implemented.

---

## Observability

- **Structured logging** — add request IDs and correlation IDs to log output
- **Metrics** — Prometheus endpoint for request latency, payment success/failure rates, daemon block lag
- **Error tracking** — integrate Sentry in backend and frontend
- **Alerting** — notify on daemon failure, external API errors, high error rate
- **UptimeRobot** — configure free-tier monitor on `/health`

## CI/CD

- **Image security scanning** — Trivy or Snyk in GitHub Actions
- **Deployment automation** — add deploy step to build CI (currently handled via infra repo + Watchtower)

## Code Quality

### Backend
- **Separate `api/state.rs`** — mixes DB, JWT, Firebase, Redis setup; split by concern
- **Make limits configurable** — currently hardcoded: API key limit (5), webhook limit (2), callback URL limit (5), rate limits (3–10/day), subscription pricing, JWT expiry (7 days), gas fee cache TTL (10 min)

### Frontend
- **Extract custom hooks** — `useFetch`, `useCrudList` to eliminate duplicate state/fetch/error patterns
- **Axios interceptor** — centralized error handling, token refresh, consistent error UI
- **Auth context** — replace scattered cookie checks with React Context
- **Split large components** — `CreateInvoice.js`, `Controller.js`, `Documentation.js`
- **CRA → Vite migration** — Create React App is deprecated

## UX

- **Mobile responsive design** — minimal responsive breakpoints; modals likely break on small screens
- **Consistent loading/error states** — mix of Spinner, disabled buttons, no skeleton loaders
- **Better MetaMask error messages** — distinguish "user rejected" vs "contract revert" vs "network error"
- **Toast notifications** — replace inline alerts for transient feedback

## Accessibility

- **ARIA labels** on icon buttons and form inputs
- **Keyboard navigation** support
- **Color contrast** verification (WCAG 2.1 AA)

## Product Features

- Instant blockchain verification (high-priority queue instead of waiting for next daemon cycle)
- Redirect on pay — for subscriptions, check blockchain immediately after payment
- Reset Firebase token after logout
- Email notifications subscription management (opt-in/out per event type)
- Bulk invoice creation
- Embeddable payment widget (HTML button for seller sites)
- Payment history export (CSV/PDF)
- Customizable notification frequency (immediate vs daily digest)
- Privacy policy and terms of use page
- Custom token/blockchain support
- Advanced sales analytics

## Database

- **Add indexes** — `user_id`, `created_at`, `paid_at` in invoices table
- **Audit logging** — track sensitive operations (payment status changes, API key CRUD)

## Skip List

| Item | Reason |
|------|--------|
| WalletConnect / wagmi+viem | 20+ hours; QR fallback covers non-MetaMask users |
| Enable smart contract commission | Owner is the only user; charging yourself has no benefit |
| i18n (react-i18next) | Zero users outside owner |
| Off-site backup sync | Local-only backups sufficient for personal use |
| White-label solution | No external users |
| CSP headers | Requires frontend audit (Firebase + Bootstrap + Web3.js inline styles) |
| Bulk invoice / CSV export / widget | No users at that volume |
| Subscription `until` at payment time bug | Real bug, zero practical impact at zero users |
| CRA → Vite migration | CRA is deprecated but functional; pure churn |
