# Security Policy

## Supported versions

Quirpy is pre-1.0 and maintained by one person in his spare time. Only the **latest release** is
supported; there are no backports to older versions. If you are on an older build, update first.

## Reporting a vulnerability

Please report privately, **not** as a public issue:

1. Preferred: [GitHub private vulnerability reporting](https://github.com/graviaDaemon/Quirpy/security/advisories/new)
   (Security → Report a vulnerability).
2. Fallback: email **graviaRotterdam@gmail.com**.

This is a hobby project, so there is no formal SLA. Realistically: an acknowledgement within about
7 days, and a fix as soon as one can be written and released. If you have heard nothing after two
weeks, feel free to nudge.

## Threat model and known limitations

Quirpy runs entirely on your machine. There is no server, no account, no telemetry, and no network
call at all other than the (not yet built) update check against GitHub Releases.

**`.qpy` project files obfuscate their contents; they do not encrypt them.** The key is a fixed
constant compiled into the source, so anyone with a copy of the source can reverse any stored value
— including Wi-Fi passwords and TOTP secrets. This is a deliberate design property (it keeps
saved projects from being casually readable in a text editor), not a vulnerability, and it does not
need to be reported. Two consequences worth stating plainly:

- Do not treat a `.qpy` file as a secure store for anything that matters.
- Never attach a `.qpy` file to a public issue. Strip secrets from logs and screenshots too.

Releases are unsigned (the macOS binary is ad-hoc signed only), which is why Gatekeeper and
SmartScreen warn on first launch. Verify downloads against the `SHA256SUMS` file published with
each release.

## In scope

- Memory-safety issues, including anything reachable through `unsafe` code.
- Crashes or undefined behaviour triggered by a malformed or hostile `.qpy` file.
- Path traversal or unintended file writes through the save and export paths.
- Anything that causes Quirpy to transmit user data off the machine.
