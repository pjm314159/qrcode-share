FROM rust:1.88-alpine AS builder

RUN apk add --no-cache musl-dev pkgconf openssl-dev openssl-libs-static

WORKDIR /app

COPY qrcode_share_backend/Cargo.toml qrcode_share_backend/Cargo.lock ./
COPY migrations /app/../migrations

RUN mkdir src && echo "fn main() {}" > src/main.rs

RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

COPY qrcode_share_backend/src ./src

RUN touch src/main.rs && cargo build --release

FROM alpine:3.21

RUN apk add --no-cache ca-certificates tzdata wget

WORKDIR /app

COPY --from=builder /app/target/release/qrcode_share_backend /app/

RUN addgroup -g 1000 appgroup && \
    adduser -u 1000 -G appgroup -D appuser

USER appuser

EXPOSE 3000

ENV HOST=0.0.0.0
ENV PORT=3000
ENV RUST_LOG=info

HEALTHCHECK --interval=10s --timeout=5s --start-period=15s --retries=3 \
  CMD wget -q --spider http://localhost:3000/health || exit 1

CMD ["./qrcode_share_backend"]
