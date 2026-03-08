# Cryo Pay

[![Rust](https://img.shields.io/badge/Rust-1.92+-orange?logo=rust)](https://www.rust-lang.org/)
[![React](https://img.shields.io/badge/React-18-blue?logo=react)](https://reactjs.org/)
[![Docker](https://img.shields.io/badge/Docker-Compose-2496ED?logo=docker)](https://docs.docker.com/compose/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Self-hosted crypto payment gateway for USDT (ERC-20) invoices on Optimism and Arbitrum.
Sellers create invoices, share a payment link, and the payer completes the transaction via
MetaMask. The blockchain daemon detects on-chain confirmation and fires webhooks,
email, and Telegram notifications automatically.

## Tech Stack

Rust (Axum) · React 18 · PostgreSQL · Redis · Nginx · Docker Compose · Solidity (EVM)

## Payment Flow

```mermaid
sequenceDiagram
    participant Buyer
    participant Frontend
    participant API
    participant Contract as Smart Contract (EVM)
    participant Daemon

    Buyer->>Frontend: Open invoice URL
    Frontend->>API: GET /payment/invoice/:id
    API-->>Frontend: Invoice (amount, networks, seller)
    Frontend->>Buyer: Payment UI (MetaMask + QR)
    Buyer->>Contract: Send USDT
    Contract-->>Daemon: Emit Transfer event (on-chain)
    loop Block scanning (per network)
        Daemon->>Contract: eth_getLogs (seller filter)
        Daemon->>API: Mark invoice paid (internal)
        API->>API: Verify amount >= invoice.amount
    end
    API->>Buyer: Webhook notification → status Paid
```

## Quickstart

1. **Clone the repo**
   ```bash
   git clone https://github.com/digitalscyther/cryo-pay && cd cryo-pay
   ```

2. **Add Firebase credentials**
   - `api/data/firebaseConfig.json` — service account private key
   - `web/src/firebaseConfig.json` — web app config

3. **Configure the backend**
   ```bash
   # Create api/.env — see the api/.env reference section below
   # Required: NETWORKS, INFURA_TOKEN, APP_SECRET, TGBOT_TOKEN, BREVO_API_KEY
   ```

4. **Start the stack**
   ```bash
   docker compose build
   NGINX_PORT=80 POSTGRES_PORT=6432 REDIS_PORT=6381 \
     docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
   ```

5. **Open the app** at `http://localhost`, sign in, and create your first invoice.

## Documentation

- [Architecture](docs/architecture.md) — module map, DB schema, auth/payment flows
- [Roadmap](docs/roadmap.md) — feature backlog and known gaps
- [Changelog](CHANGELOG.md) — what has been implemented

## Development

```bash
# Backend
cd api && cargo build
SQLX_OFFLINE=true cargo build   # without live DB

# Frontend
cd web && npm install && npm start

# Add a DB migration
DATABASE_URL=postgres://cryo:example@localhost:6432/cryo sqlx migrate add -r <name>
```

Local frontend with custom API URL:
```bash
REACT_APP_BASE_API_URL=http://127.0.0.1:3001 REACT_APP_PROJECT_NAME=LOCALTest REACT_APP_CONTACTS='{"email":"foo@bar.baz","telegram":"foo","linkedin":"foo"}' npm start
```

## api/.env reference

```dotenv
RUST_LOG=info,tower_http=trace

HOST=0.0.0.0
PORT=8080

POSTGRES_URL=postgres://cryo:example@postgres/cryo
REDIS_URL=redis://:redis123@redis

APP_SECRET=your_secret
GOOGLE_APPLICATION_CREDENTIALS=/opt/data/firebaseConfig.json
INFRA_RPM=1
ERC20_ABI_PATH=/opt/data/erc20_abi.json
CONTRACT_ABI_PATH=/opt/data/invoice_abi.json
EVENT_SIGNATURE=PayInvoiceEvent(string,address,address,uint128,uint128)
NETWORKS=[{"name":"optimism-sepolia","id":11155420,"link":"https://optimism-sepolia.infura.io/v3/foo","addresses":{"erc20":"0x9A211fD6C60BdC4Cc1dB22cBe2f882ae527B1D87","contract":"..."}},{"name":"optimism","id":10,"link":"https://optimism-mainnet.infura.io/v3/foo","addresses":{"erc20":"0x94b008aa00579c1307b0ef2c499ad98a8ce58e58","contract":"..."}},{"name":"arbitrum","id":42161,"link":"https://arbitrum-mainnet.infura.io/v3/foo","addresses":{"erc20":"0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9","contract":"..."}}]
TGBOT_TOKEN=foobarbaz
BREVO_API_KEY=foobarbaz
EMAIL_SENDER=noreply@example.com
INFURA_TOKEN=<infura_token>
WEB_BASE_URL=https://example.com:3000
API_GLOBAL_URL=http://127.0.0.1/api
CRYO_PAY_API_KEY=<self_api_key>
CRYO_PAY_SELF_ADDRESS=<wallet_address>
CRYO_PAY_RECIEVE_FROM_NETWORKS=["optimism-sepolia","optimism","arbitrum"]
```
