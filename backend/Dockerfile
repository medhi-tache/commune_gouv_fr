# Compilation isolée ; le référentiel communal est intégré dans le binaire Rust.
FROM rust:1.99-bookworm AS build
WORKDIR /src
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
COPY backend/reference ./reference
# Annuaire commun : les comptes de démonstration reprennent les mêmes noms et fonctions.
COPY frontend/reference /frontend/reference
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 qomune
WORKDIR /app
COPY --from=build /src/target/release/qomune-api /usr/local/bin/qomune-api
USER qomune
ENV API_BIND=0.0.0.0:8080
EXPOSE 8080
CMD ["qomune-api"]
