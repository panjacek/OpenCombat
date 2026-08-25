# OpenCombat — agent notes

Fork of buxx/OpenCombat (abandoned upstream, AGPL-3). Revival driven by AI agents:
planning, implementation and review are agent-executed; the human owner directs scope,
owns all merges and verifies gameplay via make targets. Rust toolchain details are
abstracted behind the Makefile so agents operate reproducibly.

## Toolchain modes

Makefile auto-detects environment:

- **docker present** → everything runs in `opencombat-dev` container (`make image` first).
  Caches in repo-local `.cache/` (bind mounts, never docker volumes — owner preference).
- **no docker (e.g. this sandbox)** → targets fall back to plain `cargo`, which honors
  `rust-toolchain.toml` (pinned 1.98.0) via rustup. Install with:
  `curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.98.0`
  then `source ~/.cargo/env`. Native dev packages are REQUIRED for any compilation
  (alsa, fontconfig, udev, libzmq + cmake) because `-sys` crate build scripts run even for
  check/clippy. Without them only `cargo fmt` / metadata-level work succeeds.

`run-gui` and `shell` are docker-only by nature.

## Conventions

- ALWAYS run `make fmt` before handing off any Rust change — CI gates on rustfmt
  and hand-edited files will fail the fmt job.

- Logs: `make check-log` / `lint-log` write `.cache/*.log` so agents can read results.
- Clippy gate is `-D warnings`; use `--keep-going` to see all crates' errors at once.
- Plans live in `docs/plans/<date>_plan_<subject>.md` (gitignored).
- Never commit/push unless owner asks.
- Upstream defect history: last upstream commit left repo uncompilable (find_path signature);
  fixed 2026-08-25 across battle_server ×2 + battle_gui ×1.

## Known quirks

- ggez 0.9: no WASM; GUI in container needs X11 socket + xauth cookie (see run-gui target,
  pattern from panjacek/docker_gui_example) + EGL/mesa packages.
- puffin version skew (0.11 vs 0.16) pending unification.
