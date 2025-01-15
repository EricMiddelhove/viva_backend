FROM rust:1.80-bookworm AS builder
LABEL authors="ericmiddelhove"

WORKDIR /app
COPY . .

RUN cargo clean
RUN cargo install --path .

FROM debian:bookworm-slim AS runner

COPY --from=builder /usr/local/cargo/bin/viva_app /usr/local/bin/viva_app

WORKDIR /home
CMD ["viva_app"]

EXPOSE 8000