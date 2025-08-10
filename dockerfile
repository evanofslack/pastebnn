FROM rust:1.83 AS builder-backend

RUN USER=root cargo new --bin pastebnn
WORKDIR /pastebnn
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && rm src/*.rs

COPY . ./

RUN rm ./target/release/deps/pastebnn* && cargo build --release


FROM node:18-alpine AS builder-frontend
ARG PUBLIC_API_BASE_URL=http://localhost:8080
ENV PUBLIC_API_BASE_URL=$PUBLIC_API_BASE_URL

WORKDIR /pastebnn
COPY ./web .
RUN npm ci && npm run build



FROM debian:bookworm-slim
ARG APP=/usr/src/app/pastebnn

RUN apt-get update \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP} \
    && mkdir -p ${APP}/web/build

COPY --from=builder-backend /pastebnn/target/release/pastebnn ${APP}
COPY --from=builder-frontend /pastebnn/build ${APP}/web/build

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./pastebnn"]
