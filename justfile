# Integrase task runner.
#
# One entry point regardless of which language a contributor came from. A
# synthetic biologist fixing a typo in the Python SDK should not have to learn
# three toolchains, and `just check` must run exactly what CI runs so that
# "works locally" means something.

set shell := ["bash", "-uc"]

default:
    @just --list

# --- setup -----------------------------------------------------------------

# Install everything needed to build and test. Idempotent.
setup: setup-rust setup-python
    @echo "setup complete. run 'just check'."

setup-rust:
    rustup component add rustfmt clippy
    @command -v cargo-nextest >/dev/null || cargo install cargo-nextest --locked
    @command -v cargo-deny    >/dev/null || cargo install cargo-deny --locked

setup-python:
    @command -v uv >/dev/null || { echo "uv not found: https://docs.astral.sh/uv/"; exit 1; }
    cd python && uv sync
    just build-ext

# Build the Rust extension into the Python environment.
#
# Not optional before running Python tests. The extension is a compiled
# artifact, so a stale copy will happily run against old Rust code and report
# green. That happened once: a fix to domain-verdict precedence passed in Rust
# and failed in Python purely because the extension had not been rebuilt, which
# wasted a debugging cycle on a non-bug.
build-ext:
    cd python && uv run maturin develop --uv

# --- the gate --------------------------------------------------------------

# Everything CI runs. If this passes, CI should pass.
check: fmt-check lint test docs-check voice diagrams
    @echo "all checks passed"

fmt:
    cargo fmt --all
    cd python && uv run ruff format .

fmt-check:
    cargo fmt --all -- --check
    cd python && uv run ruff format --check .

lint: build-ext
    cargo clippy --workspace --all-targets -- -D warnings
    cd python && uv run ruff check .
    cd python && uv run mypy --strict src

test: build-ext
    cargo nextest run --workspace 2>/dev/null || cargo test --workspace
    cd python && uv run pytest -q

# Doctests are how documentation is stopped from rotting (R-4.3): an example
# that stops compiling fails the build.
docs-check:
    cargo test --doc --workspace
    cargo doc --workspace --no-deps --document-private-items

# The brief's section 11 writing rules, enforced mechanically.
voice:
    ./tools/check-voice.sh

diagrams:
    ./tools/check-mermaid.sh

# --- verification that needs the network or extra tools --------------------

# Minimum supported Rust version. Developed on stable, floor checked here.
msrv:
    rustup toolchain install 1.93 --profile minimal
    cargo +1.93 check --workspace

audit:
    cargo deny check
    cd python && uv run pip-audit

bench:
    cargo bench --workspace

# --- convenience -----------------------------------------------------------

clean:
    cargo clean
    rm -rf python/.venv python/.pytest_cache python/.ruff_cache python/.mypy_cache
