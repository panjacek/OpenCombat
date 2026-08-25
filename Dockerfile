FROM rust:1.98-slim-trixie

ENV RUSTUP_TOOLCHAIN=1.98.0

ARG UID=1000
ARG GID=1000

RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential cmake pkg-config curl \
      libasound2-dev libfontconfig-dev libudev-dev libzmq3-dev \
      libx11-6 libxext6 libxrandr2 libxcursor1 libxi6 \
      libxkbcommon-x11-0 libgl1 libegl1 libgl1-mesa-dri mesa-vulkan-drivers \
    && rm -rf /var/lib/apt/lists/*

# no sound card in container -> discard audio instead of crashing
RUN printf 'pcm.!default {\n  type null\n}\n' > /etc/asound.conf

RUN curl -LsSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash \
    && cargo binstall --no-confirm cargo-audit cargo-llvm-cov \
    && rustup component add clippy rustfmt llvm-tools-preview

RUN groupadd -o -g ${GID} dev \
    && useradd -m -u ${UID} -g ${GID} -s /bin/bash dev

USER dev
WORKDIR /work
