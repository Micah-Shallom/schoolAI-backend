# ---- Build Stage ----
    FROM rust:1.86-slim AS builder

    WORKDIR /app
    
    # Set compiler flags to target generic x86-64
    ENV CFLAGS="-march=x86-64 -mtune=generic"
    ENV CXXFLAGS="-march=x86-64 -mtune=generic"
    ENV RUSTFLAGS="-C target-cpu=x86-64"
        
       # Install build dependencies
    RUN apt-get update && \
        apt-get install -y cmake protobuf-compiler pkg-config libssl-dev g++ && \
        rm -rf /var/lib/apt/lists/*
    
    COPY Cargo.toml Cargo.lock ./
    COPY src ./src
    COPY migration ./migration
    COPY src/services/prompts /app/src/services/prompts
    
    # Build with release profile
    RUN cargo build --release
    
    # ---- Runtime Stage ----
    FROM debian:bookworm-slim
    
    WORKDIR /app

    RUN mkdir -p /app/src/services/prompts

    
    # Install runtime dependencies
    RUN apt-get update && \
        apt-get install -y libssl3 ca-certificates && \
        rm -rf /var/lib/apt/lists/*
    
    COPY --from=builder /app/target/release/schoolAI_backend /app/schoolAI_backend
    COPY .env .env
    COPY --from=builder /app/src/services/prompts /app/src/services/prompts
    
    EXPOSE 3000
    
    # Wait for PostgreSQL and run migrations before starting the app
    ENTRYPOINT ["sh", "-c", "sleep 5 && /app/schoolAI_backend"]