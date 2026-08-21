# Quirpy

*A QR code generator that builds its codes by hand.*

[![CI](https://github.com/graviaDaemon/Quirpy/actions/workflows/ci.yml/badge.svg)](https://github.com/graviaDaemon/Quirpy/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/graviaDaemon/Quirpy?include_prereleases)](https://github.com/graviaDaemon/Quirpy/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)

## What is this

Quirpy is a desktop application for generating QR codes — URLs, Wi-Fi credentials, contact cards,
calendar events, messages and MFA secrets — from a simple form, with a live preview and export to
image files.

It is also a Rust learning project, which explains its one unusual property: the QR encoder is
written from scratch rather than pulled in from a crate. Working out the encoding by hand is the
point, not a detail of the implementation.

Everything stays on your machine. There is no server, no account, and nothing is uploaded anywhere.

## Status

**v0.1.0 — the first release, and an early one.** What works today: the GUI, all seven data types,
project save/load, settings and logging, and the hand-rolled QR encoder — the preview shows a real,
scannable Model 2 QR code at every version (1–40) and error correction level (L/M/Q/H).

**Export is not wired up yet**, so the code can be scanned off the screen but not saved as a PNG,
SVG or JPG. Micro QR and rMQR are not supported. See [docs/design.md](docs/design.md) for where it
is going.

Every `0.x` release is published as a GitHub pre-release, which is an accurate description of it.
Bug reports are the most useful thing you can send right now — above all, a code that will not
scan, together with the payload and settings that produced it.

## Download

Builds for each release are on the [Releases page](https://github.com/graviaDaemon/Quirpy/releases).

| Platform | Asset | First launch |
| --- | --- | --- |
| macOS | universal `.app` in a `.zip` | Right-click the app → **Open** → **Open** |
| Windows | zipped `.exe` | SmartScreen → **More info** → **Run anyway** |
| Linux | `.tar.gz` or AppImage | `chmod +x` the AppImage before running it |

Those extra clicks exist because the builds are **unsigned** — an Apple Developer account and a
Windows code-signing certificate both cost money, and this is a hobby project. Your OS is telling
you it cannot verify who built the binary, which is accurate. If that bothers you, build from
source instead; it takes one command.

The Linux `.tar.gz` is a bare binary and needs system GTK/X11 libraries already present. The
AppImage is the safer choice if you are unsure.

## Build from source

You need [Rust 1.85 or newer](https://rustup.rs/).

```sh
git clone https://github.com/graviaDaemon/Quirpy.git
cd Quirpy
cargo run
cargo test
```

On Debian/Ubuntu, `eframe` and `rfd` need a few system packages first:

```sh
sudo apt install libgtk-3-dev libxkbcommon-dev libwayland-dev libxcb1-dev pkg-config
```

## Contributing

Contributions are welcome — start by opening an issue, then read
[CONTRIBUTING.md](.github/CONTRIBUTING.md) and the
[Code of Conduct](.github/CODE_OF_CONDUCT.md).

One rule up front, so nobody wastes an afternoon on it: **the QR encoder is hand-rolled on purpose.**
Pull requests that replace it with a QR crate will be declined, however much cleaner the result.

## Licence

MIT — see [LICENSE](LICENSE).
