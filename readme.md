### run

#### docker compose
add config (with private key) to api/data/firebaseConfig.json
add config to web/src/firebaseConfig.json

```text
docker compose build && NGINX_PORT=80 POSTGRES_PORT=6432 docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
```

#### local

##### front


### api .env
```dotenv
RUST_LOG=info,tower_http=trace

HOST=0.0.0.0
PORT=8080

POSTGRES_URL=postgres://cryo:example@postgres/cryo

APP_SECRET=your_secret
GOOGLE_APPLICATION_CREDENTIALS=/opt/data/firebaseConfig.json
INFRA_RPM=1
ERC20_ABI_PATH=/opt/data/erc20_abi.json
CONTRACT_ABI_PATH=/opt/data/invoice_abi.json
EVENT_SIGNATURE=PayInvoiceEvent(string,address,address,uint128,uint128)
NETWORKS=[{"name":"optimism-sepolia","id":11155420,"link":"https://optimism-sepolia.infura.io/v3/foo","addresses":{"erc20":"0x9A211fD6C60BdC4Cc1dB22cBe2f882ae527B1D87","contract":"..."}},{"name":"optimism","id":10,"link":"https://optimism-mainnet.infura.io/v3/foo","addresses":{"erc20":"0x94b008aa00579c1307b0ef2c499ad98a8ce58e58","contract":"..."}}]
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
- [ ] Set up notifications about payment statuses for sellers via email and Telegram
  - [x] Add email
  - [ ] Add telegram chat id
  - [ ] Add flags where to notify
    - [ ] DB
    - [ ] Frontend Account page
  - [ ] Add email sending
    - [ ] Read Brevo doc
    - [ ] Integrate Brevo
  - [ ] Add telegram message sending
    - [ ] Telegram bot
    - [ ] User writing to bot
    - [ ] Send notification by chat_id
- [ ] Add support for Arbitrum and Base networks
  - [ ] Arbitrum
    - [ ] Find network id
    - [ ] Find USDT ERC20 smart contract address
    - [ ] Add smartcontract
      - [ ] Deploy
      - [ ] Add into config
  - [ ] Base
    - [ ] Find network id
    - [ ] Find USDT ERC20 smart contract address
    - [ ] Add smartcontract
      - [ ] Deploy
      - [ ] Add into config
- [ ] Delete own invoice
  - [ ] Backend endpoint
  - [ ] Frontend button

### Nice-to-Have
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