# pastebnn

a simple pastebin server built rust and svelte

## installation

host a local instance with docker:

```yaml
version: "3.7"
services:
  pastebnn:
    image: evanofslack/pastebnn:0.1.0
    ports:
      - "8080:8080"
    restart: unless-stopped

```

or persist pastes with a redis instance:
```yaml
version: "3.7"
services:
  redis:
    image: redis:latest
    restart: unless-stopped
    ports:
      - "6379:6379"
  pastebnn:
    image: evanofslack/pastebnn:0.1.0
    environment:
      PASTEBNN_STORAGE_BACKEND: "redis"
      PASTEBNN_REDIS_HOST: "redis"
      PASTEBNN_REDIS_PORT: 6379
    ports:
      - "8080:8080"
    restart: unless-stopped

```


## design

server: json rest api w/ tokio, axum and serde. 

ui: sveltekit and tailwind


## developement

`git clone https://github.com/evanofslack/pastebnn.git && cd pastebnn`

`docker compose -f docker-compose-dev.yaml up`


