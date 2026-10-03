# Windows cross-compilation build image.
# Target: x86_64-pc-windows-gnu — the only practically cross-compilable
# Windows target from Linux (the msvc target needs Microsoft's link.exe
# and Windows SDK, unavailable here). mingw-w64 GCC links the binary;
# the result is a standard, standalone 64-bit Windows .exe.
# The pinned image tag is the Rust toolchain versioning mechanism;
# no rust-toolchain.toml is used.
FROM rust:1.98.1-slim-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends mingw-w64 \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-pc-windows-gnu

WORKDIR /project
