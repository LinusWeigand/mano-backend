# Use Rust official image
FROM rust:latest AS builder

# Set working directory
WORKDIR /usr/src/app

# Install dependencies
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch

# Copy source code
COPY . .

# Build the release version of the Rust backend
RUN cargo build --release

# Runtime image
FROM debian:buster-slim
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/mano ./backend

# Set up environment variable (Use ARG, then ENV to allow runtime override)
ARG DATABASE_URL
ENV DATABASE_URL=${DATABASE_URL}

# Expose the port
EXPOSE 8000

# Run the backend
CMD ["./backend"]
