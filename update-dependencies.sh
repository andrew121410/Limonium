#!/bin/bash
set -e

echo "🔍 Checking for required tools..."
cargo install cargo-edit cargo-outdated --quiet

echo "📦 Upgrading all dependencies to latest versions (breaking changes allowed)..."
cargo upgrade --incompatible allow --pinned allow --recursive true

echo "🔄 Syncing lockfile..."
cargo update

echo "📊 Showing outdated status post-upgrade..."
cargo outdated || echo "✅ All dependencies are up to date."

echo "🎉 Upgrade complete. You’re now running the latest versions."

