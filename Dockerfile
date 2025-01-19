FROM rust:1.84-alpine3.21 AS builder

WORKDIR /app
COPY . .

RUN apk add musl-dev

RUN cargo clean
RUN cargo install --path .


FROM alpine:3.21

COPY --from=builder /usr/local/bin/viva_app /usr/local/bin/viva_app

WORKDIR /home

CMD ["viva_app"]

EXPOSE 8080