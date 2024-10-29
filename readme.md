### run

```text
docker compose build && NGINX_PORT=80 POSTGRES_PORT=6432 docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
```

### api .env
```dotenv
RUST_LOG=info,tower_http=trace

HOST=0.0.0.0
PORT=8080

POSTGRES_URL=postgres://cryo:example@postgres/cryo

DELAY_BETWEEN_CHECKS=300
ERC20_ABI_PATH=/opt/data/erc20_abi.json
CONTRACT_ABI_PATH=/opt/data/invoice_abi.json
EVENT_SIGNATURE=PayInvoiceEvent(string,address,address,uint128,uint128)
NETWORKS=[{"name":"optimism-sepolia","id":11155420,"link":"https://optimism-sepolia.infura.io/v3/foo","addresses":{"erc20":"0x9A211fD6C60BdC4Cc1dB22cBe2f882ae527B1D87","contract":"..."}},{"name":"optimism","id":10,"link":"https://optimism-mainnet.infura.io/v3/foo","addresses":{"erc20":"0x94b008aa00579c1307b0ef2c499ad98a8ce58e58","contract":"..."}}]
```