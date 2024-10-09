# Use an official Rust image as a parent image for building the application
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/app

# Copy the current directory contents into the container
COPY . .

# Build the core-etl application
RUN cargo build --release

# Use a smaller base image for the final container
FROM rust:latest

# Set the working directory
WORKDIR /usr/src/app

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/core-etl /usr/local/bin/core-etl

# Expose the SQLite database file as read-only if the "-s" flag is used
VOLUME ["/data"]

# Run the core-etl application
ENTRYPOINT ["/usr/local/bin/core-etl"]