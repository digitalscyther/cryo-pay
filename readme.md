### run

```text
docker compose build && NGINX_PORT=80 POSTGRES_PORT=6432 docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d
```