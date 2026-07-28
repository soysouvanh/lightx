FROM rust:1.85 as builder

WORKDIR /usr/src/lightx
COPY . .

# Release build
RUN cargo build --release --bin lightx-teb

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /usr/src/lightx/target/release/lightx-teb .

CMD ["./lightx-teb"]
