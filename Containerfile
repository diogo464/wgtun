FROM docker.io/rustlang/rust:nightly AS builder

WORKDIR /usr/src
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
	echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
	cargo build --release && \
	rm -f target/release/deps/wgtun*

COPY src ./src
RUN cargo build --release

FROM docker.io/debian:buster-slim
COPY --from=builder /usr/src/target/release/wgtun /usr/local/bin/wgtun
ENTRYPOINT ["/usr/local/bin/wgtun"]

LABEL org.opencontainers.image.source "https://github.com/diogo464/wgtun"
