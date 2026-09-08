# Etapa de compilación
FROM rust:1.80-slim-bullseye AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml ./
# Cache de dependencias
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs && cargo build --release

# Etapa final ligera
FROM debian:bullseye-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl1.1 curl && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/exposureiq-backend /app/exposureiq-backend
COPY migrations /app/migrations
RUN mkdir -p /app/uploads

EXPOSE 8080
CMD ["/app/exposureiq-backend"]
