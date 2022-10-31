FROM rust:1.63 as builder-backend

RUN USER=root cargo new --bin pastebnn
WORKDIR /pastebnn
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/pastebnn*
RUN cargo build --release


FROM node:18 as builder-frontend
WORKDIR /pastebnn
COPY ./web .
RUN npm ci
RUN npm audit fix
RUN npm run build


FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP} \
    && mkdir -p ${APP}/pastebnn/web/build

COPY --from=builder-backend /pastebnn/target/release/pastebnn ${APP}/pastebnn
COPY --from=builder-frontend /pastebnn/build ${APP}/pastebnn/web/build

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./pastebnn"]