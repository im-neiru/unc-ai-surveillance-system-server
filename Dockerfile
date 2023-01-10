FROM ubuntu:jammy

# Install dependencies
RUN apt-get update
RUN apt-get install -y \
    libpq-dev \
    libclang-dev \
    clang \
    cmake make \
    libavcodec-dev \
    libavformat-dev \
    libswscale-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer1.0-dev \
    libgtk-3-dev \
    libpng-dev \
    libjpeg-dev \
    libopenexr-dev \
    libtiff-dev \
    libwebp-dev \
    lldb

ENV TZ="Asia/Taipei"
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt-get install -y libopencv-dev

# Set environment variables
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.66.0

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y


# Install cargo
RUN apt-get install -y cargo

# Install diesel cli
RUN cargo install diesel_cli --no-default-features --features postgres

# Expose ports for lldb-server
# EXPOSE 31166
# EXPOSE 31200-31300

# Setup lldb-server-start script
# RUN chmod +x /usr/local/bin/lldb-server-start.sh

