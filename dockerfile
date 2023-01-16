FROM rust:1.63 as builder

RUN USER=root cargo new --bin pastebnn
WORKDIR /pastebnn
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release && rm src/*.rs

COPY . ./

RUN rm ./target/release/deps/pastebnn* && cargo build --release


FROM debian:buster-slim
ARG APP=/usr/src/app/pastebnn

RUN apt-get update \
    && rm -rf /var/lib/apt/lists/*

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP} \
    && mkdir -p ${APP}/web/build

COPY --from=builder /pastebnn/target/release/pastebnn ${APP}

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./pastebnn"]
