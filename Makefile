SHELL = /bin/bash
.SHELLFLAGS = -euo pipefail -c

CARGO          = cargo
MEMBERS        = monumentum_handler monumentum_core
PUBLISH_ORDER  = monumentum_handler monumentum_core

PKG           ?= monumentum_handler

CLIPPY_FLAGS  ?= -- -D warnings

.DEFAULT_GOAL := help

.PHONY: help
help:
	@echo "Usage: make <target> [PKG=<package>] [CLIPPY_FLAGS=<flags>]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*?## "}; {printf "  %-20s %s\n", $$1, $$2}'
	@echo ""
	@echo "Contoh:"
	@echo "  make ci"
	@echo "  make clippy CLIPPY_FLAGS=''"
	@echo "  make test-pkg PKG=monumentum_core"

.PHONY: all
all: build

.PHONY: build
build:
	$(CARGO) build --workspace

.PHONY: release
release:
	$(CARGO) build --release --workspace

.PHONY: check
check:
	$(CARGO) check --workspace

.PHONY: check-all
check-all:
	$(CARGO) check --workspace --all-targets --all-features

.PHONY: test
test:
	$(CARGO) test --workspace

.PHONY: test-all
test-all: test

.PHONY: test-verbose
test-verbose:
	RUST_BACKTRACE=1 $(CARGO) test --workspace -- --nocapture

.PHONY: watch-test
watch-test:
	$(CARGO) watch -x 'test --workspace'

.PHONY: watch-build
watch-build:
	$(CARGO) watch -x 'check --workspace'

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: clippy
clippy:
	$(CARGO) clippy --workspace --all-targets --all-features $(CLIPPY_FLAGS)

.PHONY: clippy-all
clippy-all: clippy

.PHONY: clippy-strict
clippy-strict:
	$(CARGO) clippy --workspace --all-targets --all-features -- -D warnings

.PHONY: lint
lint: fmt clippy

.PHONY: ci
ci: fmt-check clippy test-verbose

.PHONY: ci-fast
ci-fast: fmt-check clippy test

.PHONY: clean
clean:
	$(CARGO) clean

.PHONY: doc
doc:
	$(CARGO) doc --workspace --no-deps

.PHONY: doc-open
doc-open: doc
	$(CARGO) doc --workspace --no-deps --open

.PHONY: bench
bench:
	$(CARGO) bench --workspace

.PHONY: coverage
coverage:
	$(CARGO) llvm-cov --workspace --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

.PHONY: update
update:
	$(CARGO) update

.PHONY: audit
audit:
	@if command -v cargo-audit >/dev/null 2>&1; then \
		$(CARGO) audit; \
	else \
		echo "cargo-audit not installed. Run: cargo install cargo-audit"; \
	fi

.PHONY: publish-check
publish-check:
	@for crate in $(MEMBERS); do \
		echo "🔍 Memeriksa packaging $$crate"; \
		$(CARGO) package -p "$$crate" || exit 1; \
	done
	@echo "✅ Semua crate siap publish."

.PHONY: publish-all
publish-all:
	@for crate in $(PUBLISH_ORDER); do \
		echo "📦 Publishing $$crate ..."; \
		$(CARGO) publish -p $$crate || exit 1; \
		sleep 5; \
	done
	@echo "✅ Semua crate berhasil dipublish."

.PHONY: version
version:
	@if [ -z "$(V)" ]; then \
		echo "Usage: make version V=<major|minor|patch|X.Y.Z>"; \
		exit 1; \
	fi
	@if [ ! -x "dev/version_bump.sh" ]; then \
		echo "ERROR: dev/version_bump.sh not found or not executable"; \
		exit 1; \
	fi
	dev/version_bump.sh $(V)

.PHONY: snap
snap:
	mkdir -p dev
	@for crate in $(MEMBERS); do \
		if [ -d "$$crate" ]; then \
			echo "Snapping $$crate/src"; \
			snapcat "$$crate/src" -f markdown -o "dev/$$crate.src.snapcat.md" || true; \
		fi; \
		if [ -d "$$crate/tests" ]; then \
			echo "Snapping $$crate/tests"; \
			snapcat "$$crate/tests" -f markdown -o "dev/$$crate.tests.snapcat.md" || true; \
		fi; \
	done
	@echo "Merging all snapshots into dev/root.md"
	cat dev/*.snapcat.md > dev/root.md 2>/dev/null || true
	@echo "Done. See dev/root.md"

.PHONY: run
run:
	$(CARGO) run -p $(PKG)

.PHONY: install
install:
	$(CARGO) install --path .

.PHONY: uninstall
uninstall:
	$(CARGO) uninstall monumentum || true

.PHONY: rebuild
rebuild: release install

.PHONY: build-pkg
build-pkg:
	$(CARGO) build -p $(PKG)

.PHONY: release-pkg
release-pkg:
	$(CARGO) build --release -p $(PKG)

.PHONY: check-pkg
check-pkg:
	$(CARGO) check -p $(PKG)

.PHONY: test-pkg
test-pkg:
	$(CARGO) test -p $(PKG)

.PHONY: test-verbose-pkg
test-verbose-pkg:
	RUST_BACKTRACE=1 $(CARGO) test -p $(PKG) -- --nocapture

.PHONY: fmt-pkg
fmt-pkg:
	$(CARGO) fmt -p $(PKG)

.PHONY: fmt-check-pkg
fmt-check-pkg:
	$(CARGO) fmt -p $(PKG) -- --check

.PHONY: clippy-pkg
clippy-pkg:
	$(CARGO) clippy -p $(PKG) --all-targets --all-features $(CLIPPY_FLAGS)

.PHONY: clippy-pkg-strict
clippy-pkg-strict:
	$(CARGO) clippy -p $(PKG) --all-targets --all-features -- -D warnings

.PHONY: ci-pkg
ci-pkg: fmt-check-pkg clippy-pkg test-verbose-pkg

.PHONY: ci-pkg-strict
ci-pkg-strict: fmt-check-pkg clippy-pkg-strict test-verbose-pkg

.PHONY: doc-pkg
doc-pkg:
	$(CARGO) doc -p $(PKG) --no-deps

.PHONY: watch-test-pkg
watch-test-pkg:
	$(CARGO) watch -x 'test -p $(PKG)'

.PHONY: watch-build-pkg
watch-build-pkg:
	$(CARGO) watch -x 'check -p $(PKG)'

# Shortcut targets
.PHONY: handler
handler: PKG=monumentum_handler
handler: ci-pkg

.PHONY: core
core: PKG=monumentum_core
core: ci-pkg
