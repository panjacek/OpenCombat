# CI pipeline

Workflow: [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) — runs on every push.

## Jobs

| Job | Command | Gate |
|-----|---------|------|
| fmt | `cargo fmt --all -- --check` | formatting drift |
| check | `cargo check --workspace --keep-going` | type errors |
| test | `cargo test --workspace` | unit/integration suites |
| clippy | `cargo clippy --workspace --keep-going -- -D warnings` | lint regressions |

## Environment

- Toolchain pinned to **1.98.0** via `dtolnay/rust-toolchain@1.98.0`, consistent with
  `rust-toolchain.toml`. No floating `stable` — new rustc releases cannot silently
  break the `-D warnings` gate.
- Native dependencies (alsa, fontconfig, udev, libzmq + cmake/pkg-config) installed per job;
  needed because `zmq-sys`/`alsa-sys` build scripts run even for check/clippy.
- `Swatinem/rust-cache@v2` caches the cargo registry and target dir. First run ≈ 10 min,
  subsequent pushes ≈ 2-3 min.

## Local reproduction

Every CI job has a make target with identical flags:

    make fmt          # == ci job "fmt"
    make check        # == ci job "check"
    make test         # == ci job "test"
    make lint         # == ci job "clippy"

With docker present these run in the `opencombat-dev` container; without it they fall back
to a local rustup toolchain honoring `rust-toolchain.toml`.

`--keep-going` surfaces every failing crate in one pass instead of stopping at the first,
matching how failures are reviewed locally.

## Legacy workflows

- `release.yml` — tag-triggered (`v*`) release automation inherited from upstream; untested
  since the fork diverged.
- `check-release.yml` — Windows/MSYS build check on `releases/**` branches; same status.
