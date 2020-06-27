FROM rust:latest as builder

RUN apt-get update && apt-get -y install ca-certificates cmake musl-tools libssl-dev && rm -rf /var/lib/apt/lists/*

COPY . .

RUN rustup target add x86_64-unknown-linux-musl

ENV PKG_CONFIG_ALLOW_CROSS=1

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest

RUN apk --no-cache add ca-certificates

COPY --from=builder /target/x86_64-unknown-linux-musl/release/chat .

EXPOSE 8080

CMD ["/chat"]