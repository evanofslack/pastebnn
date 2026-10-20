FROM rust:1.97-slim-trixie AS builder-backend
WORKDIR /pastebnn

# build dependencies on their own layer so source changes reuse the cache
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo build --release --locked \
    && rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release --locked


FROM node:26-alpine AS builder-frontend
WORKDIR /web

COPY web/package.json web/package-lock.json ./
RUN npm ci

COPY web ./
RUN npm run build


# glibc + libgcc only, no shell or package manager, runs as uid 65532
FROM gcr.io/distroless/cc-debian13:nonroot
WORKDIR /app

COPY --from=builder-backend /pastebnn/target/release/pastebnn ./pastebnn
COPY --from=builder-frontend /web/build ./web/build

EXPOSE 8080
ENTRYPOINT ["/app/pastebnn"]
