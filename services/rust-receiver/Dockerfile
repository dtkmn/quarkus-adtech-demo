# --- Stage 1: Build the Rust application ---
FROM rust:1.73 as builder

# Create a new empty shell project
WORKDIR /usr/src/app
COPY . .

# Build the application in release mode
RUN cargo build --release

# --- Stage 2: Create the final, smaller image ---
FROM debian:buster-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/rust-receiver /usr/local/bin/rust-receiver

# Set the entrypoint for the container
CMD ["rust-receiver"]
