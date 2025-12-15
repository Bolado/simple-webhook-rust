FROM rust:1.92.0-slim AS build

WORKDIR /app

COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN cargo build --release

FROM cgr.dev/chainguard/glibc-dynamic

WORKDIR /app

COPY --from=build /app/target/release/simple-webhook-rust .

EXPOSE 3000

CMD ["./simple-webhook-rust"]
