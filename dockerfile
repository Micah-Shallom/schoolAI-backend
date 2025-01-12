# ---- Build Stage ----
    FROM rust:1.85-bullseye as builder

    WORKDIR /app
    
    COPY Cargo.toml Cargo.lock ./
    # RUN cargo fetch --locked
    
    COPY src ./src
    COPY migration ./migration
    
    RUN cargo build --release
    
    # ---- Runtime Stage ----
    FROM debian:bullseye-slim
    
    WORKDIR /app
    
    COPY --from=builder /app/target/release/schoolAI_backend /app/schoolAI_backend
    
    COPY .env .env
    
    EXPOSE 3000
    
    # Wait for PostgreSQL and run migrations before starting the app
    ENTRYPOINT ["sh", "-c", "sleep 5 && /app/schoolAI_backend"]
    