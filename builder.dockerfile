FROM debian:bookworm

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

# Install Tauri CLI
RUN cargo install tauri-cli

RUN cat <<EOF > /bin/BUILD
#!/bin/sh
BUILDER_HOME=/home/builder
RELEASE_DIR=\$BUILDER_HOME/src-tauri/target/release

npm run tauri build -- --verbose  && \
echo "\nCreating LogQuest release ZIP file...\n"  && \
zip --junk-paths --verbose \
    \$BUILDER_HOME/LogQuest.zip \
    \$BUILDER_HOME/README.md \
    \$BUILDER_HOME/LICENSE \
    \$RELEASE_DIR/log-quest \
    \$RELEASE_DIR/bundle/appimage/log-quest_*_amd64.AppImage \
    \$RELEASE_DIR/bundle/deb/log-quest_*_amd64.deb  && \
echo '\nDone building LogQuest!'
EOF
RUN chmod +x /bin/BUILD

# Create a "builder" user to avoid building as root
RUN useradd --create-home builder  && \
    chown   --recursive   builder:builder  $CARGO_HOME  && \
    chmod   --recursive   u+rw,g+rw        $CARGO_HOME
USER builder
WORKDIR /home/builder

# Copy the LogQuest source files into the image
COPY --chown=builder:builder . .

RUN npm install
