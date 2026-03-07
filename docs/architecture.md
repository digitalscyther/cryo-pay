# Architecture

## System Overview

Cryo Pay is a crypto payment gateway for USDT (ERC-20) invoices on EVM chains (Optimism, Arbitrum). It consists of a Rust backend, a React frontend, and is deployed via Docker Compose with Nginx, PostgreSQL, and Redis.

## Components

### Backend — `api/` (Rust / Axum)

The single binary (`main.rs`) spawns three concurrent tokio tasks:

1. **HTTP API server** — Axum routes organized under:
   - `/auth` — Firebase token verification, JWT issuance
   - `/user` — account settings, API keys, callback URLs, webhooks
   - `/payment` — invoice CRUD
   - `/blockchain` — gas fee suggestions
   - `/external` — third-party integrations (CryoPay self-callbacks)
   - `/buy` — donations, subscriptions, payment checkout

2. **Blockchain monitor daemon** (`monitoring/daemon.rs`) — continuously polls EVM chains via Infura JSON-RPC for `PayInvoiceEvent` logs. Uses sliding window rate limiters to stay within Infura credit budgets (per-second and per-day). Tracks the last processed block per network in the database to handle restarts and missed blocks.

3. **Telegram bot** (`telegram/bot.rs`) — listens for user messages to link Telegram chat IDs for payment notifications.

### Frontend — `web/` (React 18)

- Create React App with Bootstrap (react-bootstrap) for UI
- Web3.js for MetaMask wallet interaction (invoice payment)
- Firebase SDK for authentication
- Key pages: Dashboard (create/list invoices), Invoice payment page, Settings (API keys, callbacks, webhooks, subscriptions), Documentation

### Infrastructure

- **Nginx** — reverse proxy: `/api/` routes to backend (port 8080), `/` routes to frontend (port 3000)
- **PostgreSQL 17** — primary data store, accessed via sqlx with compile-time query checking
- **Redis** — rate limiting counters, gas fee caching (via ConnectionManager for auto-reconnection)
- **Docker Compose** — orchestrates all services; `docker-compose.dev.yml` overlay exposes ports for local development

## Backend Module Map (`api/src/`)

| Module | Purpose |
|---|---|
| `api/state.rs` | `AppState` — holds DB pool, Redis, Firebase creds, JWT config, network list, Infura token |
| `api/middleware/auth.rs` | JWT + API key authentication extraction |
| `api/middleware/rate_limiting/` | Redis-backed per-user rate limiting |
| `monitoring/daemon.rs` | Blockchain event polling with Infura rate limit management |
| `events/notifications/` | Email (Brevo API) and Telegram notification dispatch on payment events |
| `payments/` | CryoPay self-payment handling (subscriptions, donations via own invoices) |
| `db/` | sqlx query functions (compile-time checked via `.sqlx/` offline cache) |
| `db/billing.rs` | Payments and subscriptions queries |
| `network/` | EVM network configuration (chain ID, RPC URL, contract addresses) |
| `telegram/client.rs` | Telegram Bot API client for sending notifications |
| `mailer/` | Brevo email integration |

## Database Schema

Managed via sqlx migrations in `api/migrations/`. Key tables:

- **invoices** — payment invoices with amount, seller address, network IDs, paid status, optional external_id
- **users** — linked to Firebase auth, stores notification preferences (email/telegram flags, telegram_chat_id)
- **api_keys** — hashed API keys per user for programmatic access
- **callback_urls** — whitelisted redirect URLs after payment
- **webhooks** — URLs to POST payment success events to
- **payments** — billing records for donations/subscriptions
- **subscriptions** — active subscription state per user with expiry
- **last_block** — per-network checkpoint for the blockchain monitor daemon

## Auth Flow

1. User authenticates via Firebase (Google) in the frontend
2. Firebase ID token is sent to `/auth` endpoint
3. Backend verifies token with Firebase Admin SDK, creates/fetches user, issues a JWT
4. Subsequent requests use JWT (cookie) or API key (header) — extracted by auth middleware

## Payment Flow

1. Seller creates an invoice (amount + wallet address + networks)
2. Buyer opens invoice page, connects MetaMask, pays via smart contract (`PayInvoiceEvent`)
3. Monitor daemon detects the on-chain event, marks invoice as paid
4. Notifications sent (email/Telegram) and webhooks fired to seller
