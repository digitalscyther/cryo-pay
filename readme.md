### run

#### docker compose
add config (with private key) to api/data/firebaseConfig.json
add config to web/src/firebaseConfig.json

```text
docker compose build && NGINX_PORT=80 POSTGRES_PORT=6432 REDIS_PORT=6381 docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
```

#### local

##### front


### api .env
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
WEB_BASE_URL=https://example.com:3000
BREVO_API_KEY=foobarbaz
EMAIL_SENDER=noreply@example.com
INFURA_TOKEN=<infura_token>
```

## TODO

### Must-Have
- [x] Improve the smart contract to retain a commission of 0.3-1%, remaining on the contract
- [x] Add logic to recheck missed blocks due to network failures
- [x] Implement ~~OAuth2~~ authorization for sellers
  - [x] Add Firebase
  - [x] Add self JWT
  - [x] Refactor
  - [x] Integrate user_id for invoices
  - [x] Logout
- [x] Set up notifications about payment statuses for sellers via email and Telegram
  - [x] Add email
  - [x] Add telegram chat id
  - [x] Add flags where to notify
    - [x] DB
    - [x] Endpoints to get and set account details
    - [x] Frontend Account page
  - [x] Add email sending
    - [x] Read Brevo doc
    - [x] Integrate Brevo
  - [x] Add telegram message sending
    - [x] Telegram bot
      - [x] Webhook and LongPool
      - [x] Save telegram_chat_id into user
    - [x] User writing to bot
      - [x] Backend endpoint
        - [x] Endpoint
        - [x] Get telegram chat name
    - [x] Send notification by chat_id
- [x] Add support for Arbitrum and Base networks
  - [x] Arbitrum
    - [x] Find network id
    - [x] Find USDT ERC20 smart contract address
    - [x] Add smartcontract
      - [x] Deploy
      - [x] Add into config
  - [ ] ~~Base~~ (add when will be popular)
- [x] Delete own invoice
  - [x] Backend endpoint
  - [x] Frontend button
- [x] Set lower (Market) gas for transactions
  - [x] Backend
  - [x] Frontend
- [ ] Deploy
  - [ ] Check
  - [ ] Do

### Nice-to-Have
- [ ] (Bug) Failed pay step with correct popup (now show false successful payment)
- [ ] Add basic statistics for sellers (number of transactions, total amounts for a period)
- [ ] Automate QR code generation for invoices
- [ ] Set up storage for sellers' contact information for sending notifications
- [ ] Create a landing page with a service description and usage instructions
- [ ] Reset Firebase first token after logout

### Optional
- [ ] Add the ability to create invoices in bulk (e.g., for sellers with a large number of small orders)
- [ ] Integrate a simple widget for embedding on sellers' websites (e.g., HTML code for a payment button)
- [ ] Implement export of reports (CSV, PDF) for sellers
- [ ] Add the ability to customize notification frequency (e.g., immediately upon payment or once a day)
- [ ] Include a privacy policy and terms of use


### Tips
add migration
```bash
DATABASE_URL=postgres://cryo:example@localhost:6432/cryo sqlx migrate add -r <name>
```