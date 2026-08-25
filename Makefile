IMAGE   := opencombat-dev
UID     := $(shell id -u)
GID     := $(shell id -g)
DISPLAY ?= :0
CACHE   := $(CURDIR)/.cache
XSOCK   := /tmp/.X11-unix
XAUTH   := /tmp/.docker.xauth

DOCKER_RUN := docker run --rm \
	--user $(UID):$(GID) \
	-v $(CURDIR):/work -w /work \
	-v $(CACHE)/cargo-registry:/usr/local/cargo/registry \
	-v $(CACHE)/cargo-git:/usr/local/cargo/git \
	-v $(CACHE)/target:/work/target

CARGO := $(DOCKER_RUN) $(IMAGE) cargo
RUNNER := $(DOCKER_RUN) $(IMAGE)

# docker present -> containerized toolchain; absent -> local rustup (see rust-toolchain.toml)
ifeq ($(shell command -v docker >/dev/null 2>&1 && echo yes),yes)
MODE := docker
else
CARGO := cargo
RUNNER :=
MODE := local rustup
export CARGO_TARGET_DIR := $(CACHE)/target
endif

.DEFAULT_GOAL := help
.PHONY: help image shell check check-log fmt lint lint-log test smoke audit coverage-log run-gui cache-size clean-cache

help:
	@echo "Toolchain mode : $(MODE)"
	@echo "make image        build dev container"
	@echo "make shell        interactive shell in container"
	@echo "make check        cargo check --workspace"
	@echo "make check-log    same, output saved to .cache/check.log"
	@echo "make fmt          cargo fmt --all"
	@echo "make lint         cargo clippy --workspace --keep-going -- -D warnings"
	@echo "make lint-log     same, output saved to .cache/lint.log"
	@echo "make test         cargo test --workspace"
	@echo "make smoke        build server, boot headless 15s"
	@echo "make audit        cargo audit (RustSec advisories)"
	@echo "make coverage-log test coverage summary -> .cache/coverage.log"
	@echo "make run-gui      launch GUI with X11 passthrough"
	@echo "make cache-size   show .cache usage"
	@echo "make clean-cache  delete .cache"

image:
	mkdir -p $(CACHE)/cargo-registry $(CACHE)/cargo-git $(CACHE)/target
	docker build --build-arg UID=$(UID) --build-arg GID=$(GID) -t $(IMAGE) .

shell:
	$(DOCKER_RUN) -it $(IMAGE) bash
check:
	$(CARGO) check --workspace

check-log:
	bash -o pipefail -c '$(CARGO) check --workspace 2>&1 | tee $(CACHE)/check.log' \
		&& echo "log saved: .cache/check.log"

lint-log:
	bash -o pipefail -c '$(CARGO) clippy --workspace --keep-going -- -D warnings 2>&1 | tee $(CACHE)/lint.log' \
		&& echo "log saved: .cache/lint.log"

fmt:
	$(CARGO) fmt --all

lint:
	$(CARGO) clippy --workspace --keep-going -- -D warnings

test:
	$(CARGO) test --workspace

smoke:
	$(CARGO) build --bin battle_server
	@$(RUNNER) bash -c 'timeout 15 ./target/debug/battle_server Demo1 --rep-address tcp://0.0.0.0:4255 --bind-address tcp://0.0.0.0:4256; ec=$$?; [ $$ec -eq 124 ] && echo "SMOKE OK (server ran until timeout)" || exit $$ec'

audit:
	$(RUNNER) cargo audit

coverage-log:
	bash -o pipefail -c '$(CARGO) llvm-cov --workspace --summary-only 2>&1 | tee $(CACHE)/coverage.log' \
		&& echo "log saved: .cache/coverage.log"

run-gui: xauth-prepare
ifndef RUNNER
	$(error run-gui needs docker; run the gui binary directly on a machine with graphics)
endif
	$(DOCKER_RUN) -e DISPLAY=$(DISPLAY) -e XAUTHORITY=$(XAUTH) \
		-v $(XSOCK):$(XSOCK):ro \
		-v $(XAUTH):$(XAUTH):ro \
		--ipc=host \
		$(IMAGE) cargo run --release --bin battle_gui -- Demo1 assets/demo1_deployment.json \
		--embedded-server --init-sync \
		--server-rep-address tcp://127.0.0.1:4255 --server-bind-address tcp://0.0.0.0:4256 \
		--side a --side-a-control N --side-a-control NW --side-a-control W --side-b-control ALL

xauth-prepare:
	xauth nlist $(DISPLAY) | sed -e 's/^..../ffff/' | xauth -f $(XAUTH) nmerge -

cache-size:
	du -sh $(CACHE)/*

clean-cache:
	rm -rf $(CACHE)
