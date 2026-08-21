# Contributing to Quirpy

Quirpy is a small project with one maintainer, so a little coordination up front saves everyone
time. Thanks for being here.

## Before you start

**Open an issue first.** Every change — bug fix, feature, refactor — should have an issue behind it,
and you should comment on that issue to say you are working on it before you start writing code.
This is not bureaucracy; it is the only way to avoid two people solving the same problem twice, or
you finishing a feature that is out of scope.

## The one hard rule

**No QR encoding or decoding crate may be added as a dependency.**

Quirpy exists so that its author can work out QR encoding by hand. A pull request that swaps the
encoder for `qrcode`, `fast_qr`, `rxing` or anything similar deletes the reason the project exists,
and it will be declined no matter how good the code is. If you want to help with the encoder, help
with the hand-written one.

This covers dev-dependencies too: a reference implementation in the test suite would quietly become
the authority on whether our encoder is correct, which is the exact thing the project exists to
avoid. The encoder is validated against published ISO/IEC 18004 test vectors, invariants that
cross-check the transcribed tables, and a phone camera.

More generally: new dependencies of any kind need a justification in the pull request. Quirpy is a
desktop app that people download as a binary, so every crate added is code someone runs on their
machine on our recommendation.

## Development setup

You need [Rust 1.85 or newer](https://rustup.rs/) (edition 2024).

```sh
git clone https://github.com/<your-username>/Quirpy.git
cd Quirpy
cargo run
cargo test
```

On Debian/Ubuntu, `eframe` and `rfd` need system packages:

```sh
sudo apt install libgtk-3-dev libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev \
  libxcb1-dev libxcursor-dev libxrandr-dev libxi-dev pkg-config
```

## Before you push

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs exactly these three on Linux, macOS and Windows. Clippy warnings are errors there, so a
warning you ignored locally is a red build. Use default rustfmt settings — the repository
deliberately has no `rustfmt.toml`.

## Fork and branch workflow

Fork the repository, branch off `main`, and open a pull request back to `main`. Branch names:

- `fix/<issue>-short-slug`
- `feat/<issue>-short-slug`
- `docs/<slug>`

Keep branches short-lived. They are deleted automatically when the pull request merges.

## Pull requests

- **Link an issue.** The body must contain `Closes #N`. A check enforces this and the pull request
  fails without it. The `no-issue-needed` label is the escape hatch for typo and docs-only changes,
  and only a maintainer can apply it.
- **All pull requests are squash-merged**, so the pull request title becomes the commit message on
  `main`. Write it as one: imperative mood, no trailing period — "Add Wi-Fi hidden-network
  checkbox", not "added a checkbox for hidden networks".
- Say which operating system you actually ran the change on. See the pull request template.

## Labels and release notes

Release notes are generated from the labels on merged pull requests, so label yours (or leave it to
the maintainer):

| Label | Use for |
| --- | --- |
| `enhancement` | New features and improvements |
| `bug` | Bug fixes |
| `documentation` | Docs and README changes |
| `dependencies` | Dependency bumps |
| `internal` | Refactors, CI, tooling — anything users do not see |

These map one-to-one onto the categories in `.github/release.yml`; keep the two lists identical.

`no-issue-needed` and `needs-triage` also exist, but they are maintainer-applied and excluded from
the release notes.

## Releases

Releases are cut by the maintainer only. The version lives in `Cargo.toml`; a release is a `v*.*.*`
tag pushed to `main`, and the release workflow refuses to publish when the tag and `Cargo.toml`
disagree. Contributors never tag and never bump the version in a pull request — say so in the issue
if you think a release is due.

Pushing the tag builds and publishes the macOS, Windows and Linux artifacts, with notes generated
from the labels above.

## Project layout

- `src/quirpy_front/` — the egui interface. `app.rs` holds application state, `form.rs` is the left
  input panel, `preview.rs` the right preview panel, `menu.rs` / `menu_native.rs` the menu bar
  (macOS gets a real system menu bar via `muda`), `settings.rs` the preferences window, `history.rs`
  undo/redo.
- `src/quirpy_payload/` — pure payload string builders, one module per data type, with no egui
  imports. New data types start here and are unit-tested against their expected output string.
- `src/quirpy_project/` — `.qpy` save files (INI, plus a checksum and value obfuscation).
- `src/quirpy_config/` — user configuration.
- `src/quirpy_encoder/` — the hand-rolled QR encoder, split by stage: `galois.rs` (GF(256) and
  Reed–Solomon), `tables.rs` (the few transcribed spec tables plus derived capacities),
  `segment.rs` (mode selection and the data bit stream), `blocks.rs` (block split, error
  correction, interleaving), `matrix.rs` (function patterns and data placement), `mask.rs` (the
  eight masks and penalty scoring), `format.rs` (format and version information). This is where the
  interesting work is.
- `docs/design.md` — where the project is headed, and why.

## Scope

Some things are settled and will not be accepted:

- **Dynamic QR codes.** They need a redirect server and scan tracking, which contradicts Quirpy
  being entirely local.
- **SQRC and FrameQR.** Proprietary formats, out of scope.
- **QR Model 1.** Dropped from the current edition of ISO/IEC 18004, and no mainstream scanner
  reads it — which would leave the code with no way to verify it works.
- **Analytics or telemetry of any kind.** Quirpy makes no network calls except the (not yet built)
  update check against GitHub Releases.

Micro QR and rMQR are *not* in this list — they are stretch goals once the main encoder works.

## Testing expectations

- Payload builders and project save/load round-trips need unit tests.
- UI changes cannot be meaningfully unit-tested here, so instead say in the pull request which
  operating system you ran it on and what you clicked. "Tested on macOS 15.2, opened a saved
  project and undid twice" is worth more than a green checkbox.
