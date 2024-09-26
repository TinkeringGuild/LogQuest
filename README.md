# LogQuest

This project is very much a Work-in-Progress and still in an alpha state.

I was motivated to create this program because a program like [GINA](https://eq.gimasoft.com/gina/) is very useful for EverQuest raiding and general gameplay, however GINA does not run on Linux. I have also heard many people express frustratons with GINA's features (or lack thereof), so I thought it would be a fun and worthwhile project to create my own application that could possibly supercede GINA. LogQuest works on Windows, Linux, and (probably) macOS.

It is an important goal of LogQuest to have full GINA compatibility. You should be able to export your triggers from GINA, import them into LogQuest easily, and have basically everything work the same (within the limits of what GINA allows exporting). GINA compatibility is mostly implemented now, with just two unimplemented features remaining (coming soon!): counters and stopwatches.

LogQuest doesn't seek to be just another GINA, though. The plan is to offer a sizable superset of GINA's features.

## Differences between LogQuest and GINA

There are two main differences between how GINA and LogQuest function from a user-perspective: Effects and Tags.

### Effects and Pattern differences

An Effect is a something that happens in response to a Trigger's patterns matching a new line in a log file.

With GINA, you configure a Trigger with a single plain-text or Regular Expression pattern, then choose from a few options available to do something in when that pattern matches a new line in a log file. In LogQuest, a Trigger can have multiple patterns; if *any* of them match, the Trigger will fire. Patterns can be a GINA-style Regular Expression, a verbatim whole-line text match, or a partial-line match.

LogQuest offers a much more open-ended customization of what happens in response to a Trigger matching a line. Effects can run in a particular order, concurrently or serially, can respond to future lines matching after the Trigger, can pause a certain duration or until another line matches a new pattern, start multiple Timers that can each have their own states and effects, and much more.

When LogQuest imports GINA triggers, it re-interprets the settings of each Trigger to use the LogQuest effects system.

More documentation on the Effect system will come soon.

### Trigger Tags

In LogQuest, there is no such thing as a "Profile" like in GINA. Instead, the user can create as many "Trigger Tags" as they want, and then they can assign individual Triggers to particular Trigger Tags. When you play EverQuest, you enable specific Trigger Tags to activate all Triggers associated with those Tags.

You could have one Trigger Tag per Character that you play, or you could have Trigger Tags associated with different use-cases. For example, you might have a "Raiding" Tag that you use across several toons, or a Tag associated with a particular camp that you farm. This level of modularity of allows you to share setups very easily between toons.

Because LogQuest is written in Rust, the log-watching and effect-execution system is very efficient. You shouldn't feel limited to how many Triggers you can have enabled at one time.

## How to run

The first alpha release v0.1.0 will be published soon.

On Windows, the only dependency that you must first install to run LogQuest is [Microsoft Edge WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2). You can install it by downloading the "Evergreen Bootstrapper" from that page and running it.

If you desire to do so, you can find below instructions for how to build LogQuest on your own machine. LogQuest is fully open-source, so these same steps can be used to establish a development environment allowing you to modify it any way you wish.

### Building LogQuest

You will need to have Rust and NodeJS/NPM installed on your system to build LogQuest (unless you use Docker/Podman on Linux, as mentioned below). If you prefer a different JavaScript package manager than NPM, such as Yarn or PNPM, those should work as well.

**See the documentation sections further down for your platform-specific dependencies.**

You will also need a copy of the LogQuest source code. You can download a ZIP of the code from GitHub or you can use `git` to clone the repository if you know how to do so.

Once you have all the dependencies installed, you start LogQuest in development mode with...

```bash
cd LogQuest
npm install
npm run tauri dev
```

When started this way, Tauri will watch for changes to TypeScript or Rust source files and automatically reload, recompile, and/or restart the application, as needed. This is a very convenient development environment, so feel free to tinker!

To build an optimized *release* version of LogQuest, you run...

```bash
npm run tauri build -- --verbose
```

#### Installing Windows development dependencies

Installing Rust and NodeJS/NPM is relatively easy on Windows. There are installers you can download for each that make the process very streamlined:

- NodeJS/NPM: [Nodist](https://github.com/nodists/nodist/releases)
- Rust: [Rustup](https://rustup.rs)

When installing Rustup,  if you do not have the Microsoft Visual Studio SDK installed, it will automatically install it for you.

Once you've installed these, just follow the Building LogQuest instructions above.

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

If you use Docker, you'd run the same commands as above, simply replacing `podman` with `docker`.

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

# Project 1999

LogQuest will always remain compatible with the rules of Project 1999.

It is primarily designed for use *with* Project 1999, but it should work just the same on other servers. I do not play Live, Quarm, etc., but I welcome other users play-testing it with those clients. If you find a bug, please open an Issue on GitHub describing the problem.

# License

This code is published under the highly permissive open-source MIT License. See the [LICENSE](./LICENSE) file for more information.
