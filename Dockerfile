# Stage 1: Build the Rust binary
FROM rust:slim-bookworm AS builder

# Install build dependencies required by songbird and symphonia
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    cmake \
    pkg-config \
    libopus-dev \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app
COPY . .

# Build for release
RUN cargo build --release --jobs 1

# Stage 2: Final runtime image
FROM debian:bookworm-slim

# Install runtime dependencies for songbird, ffmpeg, yt-dlp, and JS interpreter (Deno)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    ffmpeg \
    libopus0 \
    python3 \
    wget \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

# Install yt-dlp binary
RUN wget https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -O /usr/local/bin/yt-dlp && \
    chmod a+rx /usr/local/bin/yt-dlp

# Install Deno (required by yt-dlp for JS execution/bypassing YouTube signatures)
RUN curl -fsSL https://github.com/denoland/deno/releases/latest/download/deno-x86_64-unknown-linux-gnu.zip -o deno.zip && \
    unzip deno.zip && \
    mv deno /usr/local/bin/deno && \
    chmod +x /usr/local/bin/deno && \
    rm deno.zip

WORKDIR /app

# Copy the compiled binary from the builder
COPY --from=builder /usr/src/app/target/release/rizubot-discord /usr/local/bin/rizubot-discord

# Copy migrations and .env if needed (Prisma is removed since we use SQLx)
# Wait, SQLx migrations are usually baked into the binary if using `sqlx::migrate!()` 
# which is done in `src/main.rs`. Let's ensure the binary can run.

ENV RUST_LOG=info

# Run the bot
CMD ["rizubot-discord"]
