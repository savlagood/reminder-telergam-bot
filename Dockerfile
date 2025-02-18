FROM rust:latest as builder

WORKDIR /app
COPY . .

RUN apt-get update && \
    apt-get install -y --no-install-recommends libsqlite3-dev libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir -p /app/log

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/reminder_bot /app/
COPY --from=builder /app/config/log4rs.yaml /app/config/

RUN apt-get update && \
    apt-get install -y --no-install-recommends libsqlite3-0 libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    mkdir -p /app/log

ENV TELOXIDE_TOKEN=""
ENV RUST_LOG=info

CMD ["./reminder_bot"]
