# jdpub justfile.

# List directives.
[private]
default:
    @just --list --unsorted

# Set up dependencies.
setup:
    rustup default stable
    rustup component add rust-std-x86_64-unknown-linux-musl

# Install locally.
install:
    cargo install

# Build (mostly) static release for many versions of linux.
build-many:
    cargo zigbuild --target x86_64-unknown-linux-gnu.2.28 --release
    # cargo build --target x86_64-unknown-linux-musl --release

# Publish to crates.io.
publish:
    cargo publish --workspace
