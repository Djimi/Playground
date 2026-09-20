FROM rust:1.90-bookworm AS builder

WORKDIR /app
COPY backend ./backend
RUN cargo build --locked --release --manifest-path backend/Cargo.toml

FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app/backend
COPY --from=builder /app/backend/target/release/sofia-playgrounds-backend /usr/local/bin/playground-api
COPY --from=builder /app/backend/target/release/import-playgrounds /usr/local/bin/import-playgrounds
COPY public/data/sofia-neighborhoods.geojson /app/public/data/sofia-neighborhoods.geojson

CMD ["playground-api"]
