# OpenCombat

[![CI](https://github.com/panjacek/OpenCombat/actions/workflows/ci.yml/badge.svg)](https://github.com/panjacek/OpenCombat/actions/workflows/ci.yml)

[![Preview video](preview2.png)](https://youtu.be/_N6HLZEDKPk)

Open source close combat inspired game. Upstream presentation available [here](http://www.closecombatseries.net/CCS/modules.php?name=Forums&file=viewtopic&t=11696).

This is a revival fork of [buxx/OpenCombat](https://github.com/buxx/OpenCombat)
(abandoned since 2024, last upstream commit left the tree uncompilable).
Revived and maintained with AI-assisted development. Notable fixes so far:

- repaired broken `find_path` signature migration that prevented compilation,
- repaired stale deployment assets (missing soldier types / squad type maps),
- full workspace lint-clean gate (`clippy -D warnings`), rust-toolchain pinned,
- containerized dev environment (see below), first test suite + coverage baseline,
- GUI launch fixed in container (audio null sink, xauth passthrough, `--init-sync`).

## Development

### Requirements

Easiest path is Docker — no Rust installation needed on the host:

    docker

For a native toolchain instead: Rust 1.98 (via `rust-toolchain.toml`) plus Debian packages

    build-essential cmake pkg-config curl libasound2-dev libfontconfig-dev libudev-dev libzmq3-dev

### Run

All commands go through the Makefile. First build once:

    make image

#### Standalone server

    make smoke            # builds battle_server, boots headless for 15s as sanity check

or interactively:

    make shell
    cargo run --release --bin battle_server -- Demo1 \
        --rep-address tcp://0.0.0.0:4255 --bind-address tcp://0.0.0.0:4256

#### Gui with embedded server

    make run-gui          # X11 + xauth passthrough, placement phase window appears

Arguments are documented in `Makefile`; spawn zone controls (`--side-a-control`,
`--side-b-control`) select where each side may deploy. Note `--init-sync` is required —
without it the deployment is never applied and the game panics on start.

#### Quality gates

    make check            # cargo check --workspace
    make lint             # clippy --workspace --keep-going -- -D warnings
    make fmt              # cargo fmt --all
    make test             # cargo test --workspace
    make coverage-log     # llvm-cov summary -> .cache/coverage.log
    make audit            # cargo audit (RustSec advisories)

`check-log` / `lint-log` variants write `.cache/*.log`. Caches live in `.cache/`
(bind mounts; downloads happen once).

### Profile

Install [puffin_viewer](https://github.com/EmbarkStudios/puffin/tree/main/puffin_viewer):

    cargo install puffin_viewer

Start server or client with `--profile` flag:

    cargo run --bin battle_gui -- Demo1 assets/demo1_deployment.json --embedded-server --init-sync \
        --server-rep-address tcp://0.0.0.0:4255 --server-bind-address tcp://0.0.0.0:4256 \
        --side a --side-a-control W --side-a-control NW --side-a-control SW --side-b-control ALL --profile

Output will be like :

![Puffin viewer](puffin_viewer.png)

## CI

See [docs/ci.md](docs/ci.md) — four jobs (fmt, check, test, clippy) on every push.
