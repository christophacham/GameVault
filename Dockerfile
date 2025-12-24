# =============================================================================
# GameVault - Single Multi-Stage Dockerfile
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build Frontend (Next.js static export)
# -----------------------------------------------------------------------------
FROM node:22-alpine AS frontend-builder

WORKDIR /app/frontend

# Copy package files
COPY frontend/package*.json ./

# Install dependencies
RUN npm install

# Copy source files
COPY frontend/ ./

# Build static export
RUN npm run build

# -----------------------------------------------------------------------------
# Stage 2: Build Backend (Rust)
# -----------------------------------------------------------------------------
FROM rust:1.85-alpine AS backend-builder

WORKDIR /app/backend

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# Copy cargo files first for dependency caching
COPY backend/Cargo.toml ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy actual source code and build.rs
COPY backend/src ./src
COPY backend/build.rs ./build.rs

# Build the release binary (touch to invalidate cargo cache from dummy build)
RUN touch src/main.rs && cargo build --release

# -----------------------------------------------------------------------------
# Stage 3: Runtime
# -----------------------------------------------------------------------------
FROM alpine:3.20

WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libgcc

# Copy the backend binary
COPY --from=backend-builder /app/backend/target/release/gamevault-backend /app/server

# Copy the frontend static files
COPY --from=frontend-builder /app/frontend/out /app/public

# Create data directory
RUN mkdir -p /data

# Environment variables
ENV DATABASE_URL=sqlite:///data/games.db?mode=rwc
ENV GAMES_PATH=/games
ENV PORT=3000
ENV HOST=0.0.0.0
ENV RUST_LOG=info

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/health || exit 1

# Run the server
CMD ["/app/server"]
