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

### Sandbox bootstrap (no root, no apt — proven 2026-08-26)

For locked-down environments without sudo/apt/docker. Installs everything into
`~/.local` + `~/.cargo`. Verified: `cargo test -p battle_core` 48/48 green.

```bash
# 1. Rust toolchain (pinned by rust-toolchain.toml)
curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain 1.98.0

# 2. cmake + ninja + meson via pip (user space)
pip install cmake ninja meson

# 3. pkgconf (pkg-config replacement; 2.3.0 is meson-only)
curl -sSL -o ~/tmp/pkgconf.tar.xz https://distfiles.ariadne.space/pkgconf/pkgconf-2.3.0.tar.xz
tar -xJf ~/tmp/pkgconf.tar.xz -C ~/tmp && mkdir -p ~/tmp/pkgconf-build
~/.local/bin/meson setup ~/tmp/pkgconf-build ~/tmp/pkgconf-2.3.0 --prefix=$HOME/.local
~/.local/bin/meson install -C ~/tmp/pkgconf-build
ln -sf pkgconf ~/.local/bin/pkg-config

# 4. libzmq shared (static fails at link: C++ symbols, no libstdc++ in cc link line;
#    BUILD_STATIC also defaults ON — delete the .a after install so linker picks .so)
curl -sSL -o ~/tmp/libzmq.tar.gz https://github.com/zeromq/libzmq/releases/download/v4.3.5/zeromq-4.3.5.tar.gz
tar -xzf ~/tmp/libzmq.tar.gz -C ~/tmp
~/.local/bin/cmake -S ~/tmp/zeromq-4.3.5 -B ~/tmp/libzmq-build \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \   # cmake 4.x rejects old minimums otherwise
  -DCMAKE_INSTALL_PREFIX=$HOME/.local -DWITH_LIBSODIUM=OFF \
  -DBUILD_TESTS=OFF -DBUILD_SHARED=ON
~/.local/bin/cmake --build ~/tmp/libzmq-build -j$(nproc)
~/.local/bin/cmake --install ~/tmp/libzmq-build && rm -f ~/.local/lib/libzmq.a

# 5. Env for every shell running cargo here (put in ~/.bashrc or export per command):
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export PKG_CONFIG_PATH="$HOME/.local/lib/pkgconfig"
export LD_LIBRARY_PATH="$HOME/.local/lib"      # runtime lookup for libzmq.so.5
export CARGO_TARGET_DIR="$HOME/.cache/oc-target"  # repo target/ may be root-owned
```

Gotchas hit on the way:
- `/tmp/opencode` can be root-owned/unwritable → use `~/tmp`.
- zmq-sys honors `LIBZMQ_NO_PKG_CONFIG=1` + `LIBZMQ_LIB_DIR` / `LIBZMQ_INCLUDE_DIR`
  as a fallback if pkg-config itself is unavailable.
- alsa/fontconfig/udev `-sys` crates are NOT needed for `battle_core` targets;
  only `battle_gui` builds need them (docker covers those).

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
