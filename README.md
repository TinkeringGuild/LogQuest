# LogQuest

This project is very much a Work-in-Progress and still in an alpha state.

Initially I want to create a Linux-compatible game overlay and notification system, similar to [GINA](https://eq.gimasoft.com/gina/), that solves my personal, immediate needs.

The next step is to achieve GINA feature-parity, including support for GINA's XML trigger format.

The ultimate plan is to go beyond GINA and create a next-generation log-based overlay system, notification system, log analyzer, and gameplay-enhancing toolkit that is fully cross-platform.

LogQuest will always remain compatible with the rules of Project 1999.

## How to run

At the moment there are no release binaries published. LogQuest is not ready for that yet.

### Build dependencies

You will need to have Rust, Tauri and NPM installed on your system to build LogQuest. Follow the [Tauri Prerequisites Guide](https://tauri.app/v1/guides/getting-started/prerequisites) for instructions on how to set that up for your system.

LogQuest uses the cross-platform [tts](https://crates.io/crates/tts) crate for text-to-speech.

Building on Linux requires having `speech-dispatcher` v0.11 installed. This should be available in your package manager. To build on Arch Linux, you simply need to `pacman -S speech-dispatcher`. If you prefer using Nix packages and have Nix setup properly to integrate with your C/C++ compiler environment variables, you can probably install the [speechd](https://search.nixos.org/packages?from=0&size=50&sort=relevance&type=packages&query=speech-dispatcher) Nix package.

For building on Windows or macOS, you *might* need some first-party developer tools installed to build LogQuest using the platform-specific TTS engine. This is unconfirmed at the moment; if you resolve build problems on these systems, please open an Issue with what you learned so this README can be updated with the correct information.

### Running the desktop app

Once everything is installed, you can simply run:

```
cd LogQuest
npm run tauri dev
```

This builds the program in `debug` mode, launches the application, and automatically reloads/restarts the application when TypeScript or Rust files are changed. It is a very convenient development environment, so feel free to tinker!

To build a `release` binary of LogQuest, you run `npm run tauri build`.

# The Tinkering Guild

The Tinkering Guild seeks to develop and share useful tools for EverQuest players.

LogQuest ("LQ") is developed under the Tinkering Guild umbrella organization, therefore it's an open-source project open to all other EverQuest-loving developers, with a particular emphasis on use with Project 1999. LQ should be compatible with modern EverQuest ("Live"), however support for that will have to come when there are tinkerers who want to develop features for that use-case.

# License

This code is published under the highly permissive open-source MIT License. See the [LICENSE](./LICENSE) file for more information.
