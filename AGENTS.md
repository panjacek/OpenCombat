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
- **locked-down sandbox (no root/apt/docker)** → user-space bootstrap procedure:
  see `docs/sandbox_bootstrap.md` (rustup + pip cmake/meson + pkgconf + shared
  libzmq; verified `cargo test -p battle_core` 48/48 green). Requires per-shell
  env: `PATH` (+`~/.local/bin`), `PKG_CONFIG_PATH=$HOME/.local/lib/pkgconfig`,
  `LD_LIBRARY_PATH=$HOME/.local/lib`, `CARGO_TARGET_DIR` off the repo.

`run-gui` and `shell` are docker-only by nature.

## Conventions

- ALWAYS run `make fmt` before handing off any Rust change — CI gates on rustfmt
  and hand-edited files will fail the fmt job.

- Commits follow Conventional Commits: `<type>(<scope>): <imperative summary>`.
  Types: `feat`, `fix`, `test`, `ci`, `docs`, `refactor`, `chore`. Scope optional
  (e.g. crate or area name). Keep behavior fixes separate from test/doc commits
  so bisect stays meaningful. Never mix unrelated changes into one commit.

- Logs: `make check-log` / `lint-log` write `.cache/*.log` so agents can read results.
- Clippy gate is `-D warnings`; use `--keep-going` to see all crates' errors at once.
- Plans live in `docs/plans/<date>_plan_<subject>.md` (gitignored).
- Never commit/push unless owner asks.

## Session lifecycle

- One branch per work batch, cut from `main` after the previous PR merged.
  Name it after the work (`feat/network-feature-gate`, `test/visibility-suite`), not the phase number.
- Session start: read latest `docs/plans/*` for state; `make check-log && make test` on host
  (or trust last log timestamps in `.cache/`) to confirm baseline before changing anything.
- Session end: update the plan doc status table + append an addendum with what shipped,
  what's pending verification, and the exact host commands the owner must run.
- Verification loop used so far: agent writes → `cargo fmt --all -- --check` locally
  (syntax gate) → owner runs `make test`/`lint-log`/`coverage-log` on host docker →
  agent reads `.cache/*.log` and fixes. Since 2026-08-26 the sandbox compiles+tests
  `battle_core` natively (see bootstrap above), so most round trips are gone; host
  verify still required for GUI/server targets.
- Upstream defect history: last upstream commit left repo uncompilable (find_path signature);
  fixed 2026-08-25 across battle_server ×2 + battle_gui ×1. Defect #2/#3: stale deployment
  assets (missing type_ / squad_types). Defect #4: Direction::from_angle negative-angle
  NW-sector bug — all found by the new test suites; keep that pattern going.

## Known quirks

- ggez 0.9: no WASM; GUI in container needs X11 socket + xauth cookie (see run-gui target,
  pattern from panjacek/docker_gui_example) + EGL/mesa packages.
- puffin version skew (0.11 vs 0.16) pending unification.
