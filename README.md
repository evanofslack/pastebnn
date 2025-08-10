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


## design

server: json rest api w/ tokio, axum and serde. 

ui: sveltekit and tailwind


## developement

`git clone https://github.com/evanofslack/pastebnn.git && cd pastebnn`

`cargo run`

or

`docker compose -f docker-compose-dev.yaml up`




