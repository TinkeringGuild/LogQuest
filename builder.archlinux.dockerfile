FROM archlinux

# This is needed to fix a problem in the linuxdeploy plugin
# that Tauri uses to build an AppImage on Arch Linux.
# More info: https://github.com/tauri-apps/tauri/issues/8929
ENV NO_STRIP=true

ENV RUSTUP_HOME=/usr/local/rustup
ENV CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH

# Install system dependencies for Arch Linux
RUN pacman -Sy --noconfirm  \
      zip  \
      wget  \
      base-devel  \
      clang  \
      webkit2gtk \
      npm  \
      speech-dispatcher  \
        &&  \
    pacman -Scc --noconfirm
      # libsoup  \

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
