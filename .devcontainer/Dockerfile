# .devcontainer/Dockerfile
FROM docker.io/library/rust:1.88-slim

# Install system dependencies
RUN apt-get update && \
    apt-get install -y \
      git \
      gcc \
      make \
      pkg-config \
      libssl-dev \
      curl \
      luarocks \
      lua5.4 \
      lldb \
      ca-certificates \
      && rm -rf /var/lib/apt/lists/*

# Install Rust tools (just, cargo-watch)
RUN cargo install just cargo-watch

# Install LuaRocks packages if needed (example: busted for Lua testing)
RUN luarocks install busted

# Set up workspace
WORKDIR /workspace

ENV PATH="/root/.cargo/bin:${PATH}"