# Windows Build Guide

How to build the Windows executable `dist/snake.exe` from a clean project
checkout. Docker is the only build environment; see [brief.md §3]
(../.agent/project-info/brief.md) for the build environment requirements.

## Prerequisites

- **Docker** — required. The build runs entirely inside Docker; nothing is
  compiled on the host.
- **Docker Compose** — required. The build is invoked through the compose
  service defined in [`compose.yaml`](../compose.yaml).
- **Rust / Cargo — NOT required on the host.** The pinned Docker image
  ([`Dockerfile`](../Dockerfile), `rust:1.98.1-slim-bookworm`) provides the
  Rust toolchain, the `x86_64-pc-windows-gnu` target, and the mingw-w64
  linker (configured in [`config.toml`](../.cargo/config.toml)).
- No specific IDE or development environment is assumed — any setup that can
  run the two commands below works.

The resulting executable is a standalone 64-bit Windows `.exe`
(`x86_64-pc-windows-gnu` target); see brief §17 for where it fits in the
project structure.

## Build Workflow

From a clean checkout of the repository, run the single documented command:

```text
docker compose run --rm build
```

What happens:

1. Compose builds the build image from the Dockerfile (implicit on first
   use — no separate image-build command is required).
2. The service runs `cargo build --release --target x86_64-pc-windows-gnu`
   inside the container.
3. The compiled binary is copied to the host-mounted output directory.

When the command exits cleanly, the artifact is on the host at:

    dist/snake.exe

Container-only details: the intermediate Cargo `target/` directory stays
inside the container (`CARGO_TARGET_DIR=/tmp/target`) so the host checkout
is not polluted; the `dist/` output directory is git-ignored so generated
binaries are never committed. `Cargo.lock` is written back to the project
root by the build and should be committed once generated — it pins
dependency versions for reproducible builds.

## What to Copy to Windows

For manual runtime validation on a Windows machine, copy the single file
`dist/snake.exe` (no other files, no DLLs are needed) and run it directly
from a Windows terminal:

```text
snake.exe
```

Runtime validation is a **separate manual step performed on Windows** —
it is NOT part of the Docker build process. Check the running game against
the Definition of Done checklist in [brief.md §18]
(../.agent/project-info/brief.md).
