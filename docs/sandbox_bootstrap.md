# Sandbox bootstrap (no root, no apt)

For locked-down environments without sudo/apt/docker (agent sandboxes, CI
runners without privileges). Installs everything into `~/.local` + `~/.cargo`.
Proven 2026-08-26: `cargo test -p battle_core` 48/48 green.

## Why needed

`-sys` crate build scripts run even for `check`/`clippy`, so compiling
`battle_core` requires native deps even without GUI targets:
- `zmq-sys` → libzmq C++ library + `pkg-config`
- alsa/fontconfig/udev `-sys` crates are NOT needed for `battle_core`;
  only `battle_gui` builds need them (docker covers those).

## Steps

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

# 4. libzmq shared
curl -sSL -o ~/tmp/libzmq.tar.gz https://github.com/zeromq/libzmq/releases/download/v4.3.5/zeromq-4.3.5.tar.gz
tar -xzf ~/tmp/libzmq.tar.gz -C ~/tmp
# -DCMAKE_POLICY_VERSION_MINIMUM=3.5 is required: cmake 4.x rejects libzmq's old minimum otherwise
~/.local/bin/cmake -S ~/tmp/zeromq-4.3.5 -B ~/tmp/libzmq-build \
  -DCMAKE_POLICY_VERSION_MINIMUM=3.5 \
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

## Gotchas hit on the way

- `/tmp/opencode` can be root-owned/unwritable → use `~/tmp`.
- Static libzmq fails at link (`__gxx_personality_v0` etc.): rust's `cc`
  link line has no libstdc++. Shared `.so` carries its own C++ dep — use it,
  and delete the installed `.a` so the linker cannot pick it.
- `BUILD_STATIC` defaults ON in libzmq's cmake — hence the explicit `rm`.
- Inline `# comment` after `\` breaks bash line continuation — keep comments
  on their own line inside multi-line commands.
- zmq-sys honors `LIBZMQ_NO_PKG_CONFIG=1` + `LIBZMQ_LIB_DIR` /
  `LIBZMQ_INCLUDE_DIR` as a fallback if pkg-config itself is unavailable.
