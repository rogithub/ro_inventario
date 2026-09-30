# Imagen de la aplicación privada (punto de venta). Solo el binario: las migraciones van dentro
# de él y el negocio.toml lo monta cada instancia (vive con sus manifests, no aquí).
# CI la publica en amd64; en la Raspberry, `podman build` la compila en arm64 para probarla.

# ── compilación ─────────────────────────────────────────────────────────────
# Misma versión que rust-toolchain.toml.
FROM docker.io/library/rust:1.98.0-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY apps ./apps
RUN cargo build --release --locked -p privada

# ── ejecución ───────────────────────────────────────────────────────────────
FROM docker.io/library/debian:bookworm-slim
COPY --from=build /src/target/release/privada /usr/local/bin/privada
# Usuario sin privilegios, numérico para que Kubernetes pueda exigir runAsNonRoot.
USER 10001:10001
ENV LOG_FORMAT=json \
    PORT=5100
EXPOSE 5100
CMD ["privada"]
