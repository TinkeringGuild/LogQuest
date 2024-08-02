FROM debian

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

# Install dependencies
RUN apt-get update && apt-get install --yes \
    file \
    zip \
    wget \
    curl \
    build-essential \
    libssl-dev \
    pkg-config \
    libclang-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    npm \
    libasound2-dev \
    libspeechd-dev \
    speech-dispatcher \
    libappimage-dev \
    # Clean up
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl https://sh.rustup.rs --silent --show-error --fail | \
    sh -s -- -y --default-toolchain stable

# Create a "builder" user to avoid building as root
RUN useradd --create-home builder  && \
    chown   --recursive   builder:builder  $CARGO_HOME  && \
    chmod   --recursive   u+rw,g+rw        $CARGO_HOME
USER builder
WORKDIR /home/builder

# Copy the LogQuest source files into the image
COPY --chown=builder:builder . .

# Install Tauri and other frontend deps
RUN npm install

# Build the Tauri app
RUN npm run tauri build -- --verbose

# Create the LogQuest.zip output file
RUN zip --junk-paths --verbose \
    ./LogQuest.zip \
    ./README.md \
    ./LICENSE \
    ./src-tauri/target/release/log-quest \
    ./src-tauri/target/release/bundle/appimage/log-quest*.AppImage \
    ./src-tauri/target/release/bundle/deb/log-quest*.deb  \
    ./src-tauri/target/release/bundle/rpm/log-quest*.rpm
