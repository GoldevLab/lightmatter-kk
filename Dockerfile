FROM rust:1.91-bookworm AS builder
WORKDIR /app
RUN rustup target add wasm32-unknown-unknown \
    && cargo install wasm-bindgen-cli --version 0.2.100 --locked
COPY . .
RUN cargo build --release --lib --target wasm32-unknown-unknown --no-default-features --features wasm \
    && wasm-bindgen --target web --out-dir public/images \
        target/wasm32-unknown-unknown/release/lightmatter.wasm
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --uid 65532 --create-home app
WORKDIR /app
COPY --from=builder /app/target/release/lightmatter /app/server
COPY --from=builder /app/public /app/public
RUN mkdir -p /data/reports && chown -R 65532:65532 /data /app
ENV RESUMA_ENV=production
ENV RESUMA_ADDR=0.0.0.0:8080
ENV RESUMA_TRUST_PROXY=1
ENV RESUMA_DATA_DIR=/data
ENV CARGO_MANIFEST_DIR=/app
ENV SITE_URL=https://lightmatter-kk.fly.dev
USER 65532
EXPOSE 8080
CMD ["/app/server"]
