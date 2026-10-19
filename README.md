# pastebnn

a simple pastebin server built rust and svelte

## demo

https://bin.evanslack.dev

## ui

<img width="1258" height="886" alt="Screenshot 2025-08-10 at 6 54 25 PM" src="https://github.com/user-attachments/assets/1af498c8-c6bd-40ad-b5f0-2a01fd5e9bb1" />


## installation

host a local instance with docker:

```yaml
version: "3.7"
services:
  pastebnn:
    image: evanofslack/pastebnn:latest
    restart: unless-stopped
    environment:
      PASTEBNN_MAX_SIZE_BYTES: 1048576 # 1MB
      PASTEBNN_APP_URL: "bin.mysite.com" # visual effect only
    ports:
      - "8080:8080"

```

or persist pastes with redis:
```yaml
version: "3.7"
services:
  redis:
    image: redis:latest
    restart: unless-stopped
    ports:
      - "6379:6379"
  pastebnn:
    image: evanofslack/pastebnn:latest
    restart: unless-stopped
    environment:
      PASTEBNN_STORAGE_BACKEND: "redis"
      PASTEBNN_REDIS_HOST: "redis"
      PASTEBNN_REDIS_PORT: 6379
    ports:
      - "8080:8080"

```


## configuration

all settings are environment variables:

| variable | default | description |
| --- | --- | --- |
| `PASTEBNN_API_HOST` | `0.0.0.0` | listening host |
| `PASTEBNN_API_PORT` | `8080` | listening port |
| `PASTEBNN_APP_URL` | `localhost` | base url shown in the ui (visual only) |
| `PASTEBNN_LOG_LEVEL` | `info` | log level (`RUST_LOG` syntax) |
| `PASTEBNN_MAX_SIZE_BYTES` | `1024` | max size of a single paste |
| `PASTEBNN_MAX_TOTAL_BYTES` | `268435456` | max combined size of all pastes (in memory store only) |
| `PASTEBNN_MAX_EXPIRE_SECONDS` | `2592000` | max lifetime of an expiring paste (30 days) |
| `PASTEBNN_PURGE_PERIOD` | `60` | seconds between purging expired pastes |
| `PASTEBNN_RATE_LIMIT_PER_MINUTE` | `60` | paste api requests per minute per client ip, `0` disables |
| `PASTEBNN_TRUST_PROXY` | `false` | use the rightmost `X-Forwarded-For` entry as client ip. only enable behind a reverse proxy |
| `PASTEBNN_STORAGE_BACKEND` | `in-memory` | `in-memory` or `redis` |
| `PASTEBNN_REDIS_HOST` / `_PORT` / `_USERNAME` / `_PASSWORD` | | redis connection |

when using redis, cap its memory (e.g. `--maxmemory 256mb --maxmemory-policy noeviction`) since `PASTEBNN_MAX_TOTAL_BYTES` does not apply.

## api

- `POST /api/paste` with `{"text": "...", "key": null, "seconds_until_expire": 600, "burn_on_read": false}`
  - `key` is optional. when omitted a random 10 character key is generated. custom keys are 1-64 chars of `[A-Za-z0-9_-]`
  - returns `201` with the paste and a `delete_token`. the token is only returned here
  - errors: `400` invalid input, `409` key taken, `413` too large, `429` rate limited, `507` storage full
- `GET /api/paste/:key` returns the paste, `404` if missing, expired or already burned
- `DELETE /api/paste/:key` with header `X-Delete-Token: <token>`, `403` if the token is missing or wrong

## design

server: json rest api w/ tokio, axum and serde. 

ui: sveltekit and tailwind


## developement

`git clone https://github.com/evanofslack/pastebnn.git && cd pastebnn`

`cargo run`

or

`docker compose -f docker-compose-dev.yaml up`




