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
    lldb \
    curl

ENV TZ="Asia/Taipei"
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone
RUN apt-get install -y libopencv-dev

# Optional terminal
RUN apt-get install -y fish git


# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Set environment variables
ENV RUSTUP_HOME=/root/.rustup/ \
    CARGO_HOME=/root/.cargo/ \
    PATH=/root/.cargo/bin/:${PATH} \
    RUST_VERSION=1.66.0

# Install cargo
# RUN apt-get install -y cargo

# Switch to stable toolchain 
RUN /root/.cargo/bin/rustup default stable 

# Install diesel cli
RUN /root/.cargo/bin/cargo install diesel_cli --no-default-features --features postgres

# Test rustc
RUN rustc --version

# Expose ports for lldb-server
# EXPOSE 31166
# EXPOSE 31200-31300

# Setup lldb-server-start script
# RUN chmod +x /usr/local/bin/lldb-server-start.sh
