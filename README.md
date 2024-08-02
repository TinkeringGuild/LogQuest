# LogQuest

This project is very much a Work-in-Progress and still in an alpha state.

Initially I want to create a Linux-compatible game overlay and notification system, similar to [GINA](https://eq.gimasoft.com/gina/), that solves my personal, immediate needs.

The next step is to achieve GINA feature-parity, including support for GINA's XML trigger format.

The ultimate plan is to go beyond GINA and create a next-generation log-based overlay system, notification system, log analyzer, and gameplay-enhancing toolkit that is fully cross-platform.

LogQuest will always remain compatible with the rules of Project 1999.

## How to run

At the moment there are no release binaries published. LogQuest is not ready for that yet. You will have to build LogQuest yourself, for now.

### Building LogQuest

You will need to have Rust and NPM installed on your system to build LogQuest (unless you use Docker/Podman on Linux, as mentioned below). Follow the [Tauri Prerequisites Guide](https://tauri.app/v1/guides/getting-started/prerequisites) for instructions on how to set those up for your system. If you prefer a different JavaScript package manager than NPM, such as Yarn or PNPM, those should work as well.

See the documentation below for additional platform-specific dependencies.

Once you have all the dependencies installed, you start LogQuest in development mode with...

```bash
cd LogQuest
npm run tauri dev
```

When started this way, Tauri will watch for changes to TypeScript or Rust source files and automatically reload, recompile, and/or restart the application, as needed. This is a very convenient development environment, so feel free to tinker!

To build a release version of LogQuest, you run...

```bash
npm run tauri build -- --verbose
```

#### Building for Windows or macOS

For building on Windows or macOS, you *might* need some first-party developer tools installed to build LogQuest using the platform-specific TTS engine. This is unconfirmed at the moment; if you resolve build problems on these systems, please open an Issue with what you learned so this README can be updated with the correct information.

#### Building a release version for Linux with Docker/Podman

The easiest way to build on Linux, if you don't want a LogQuest development environment, is to use Docker or Podman. Podman is a little easier to work with because it does not need root-equivalent permissions to run containers. To get this setup quickly, refer to the [Podman Installation Instructions](https://podman.io/docs/installation#installing-on-linux). If you already have Docker installed, just use that.

The advantage of using this method is that all build dependencies are automatically installed into a self-contained image, rather than into your host system. This is a reliable, repeatable way to get everything setup without system-specific gotchas. When done, you can remove the image and 100% of what was just installed gets cleaned up. It is similar to (but faster than) building inside a virtual machine.

In your `git clone`d directory of LogQuest, run...

```bash
# This builds LogQuest inside a Docker/Podman image
podman build --file builder.debian.dockerfile --tag log-quest-builder

# Copy the file out of the image
podman run --rm log-quest-builder cat LogQuest.zip > LogQuest.zip

# Deletes the image from your system
podman rmi log-quest-builder

# NOTE! You will still have the base image installed. To remove it as
# well, get its "IMAGE ID" by running...
podman images

# ...and removing it specifically...
podman rmi <PUT IMAGE ID HERE>

# Alternatively, you can just remove all installed images:
podman rmi --all
```

If you use Docker, you'd run the same commands but replacing `podman` with `docker`.

There is also a `builder.archlinux.dockerfile` that does the same thing but with Arch Linux as the base distro.

#### Build dependencies for Linux without Docker/Podman

Below are the dependencies needed for Arch Linux and Debian...

``` bash
# On Arch Linux
pacman -S zip wget base-devel clang webkit2gtk npm speech-dispatcher

# On Debian (and Debian variants, e.g. Ubuntu or Linux Mint)
apt install file zip wget curl build-essential libssl-dev pkg-config libclang-dev libgtk-3-dev libwebkit2gtk-4.0-dev libasound2-dev libspeechd-dev speech-dispatcher libappimage-dev npm
```

With these installed, you should be able to run a development environment or build release binaries.

# The Tinkering Guild

The Tinkering Guild seeks to develop and share useful tools for EverQuest players.

LogQuest ("LQ") is developed under the Tinkering Guild umbrella organization, therefore it's an open-source project open to all other EverQuest-loving developers, with a particular emphasis on use with Project 1999. LQ should be compatible with modern EverQuest ("Live"), however support for that will have to come when there are tinkerers who want to develop features for that use-case.

# License

This code is published under the highly permissive open-source MIT License. See the [LICENSE](./LICENSE) file for more information.
