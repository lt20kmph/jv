#!/bin/bash

# Abort on any failed command so we never silently redeploy stale code.
set -e

echo "Fetching..."
git fetch origin
git reset --hard origin/main

echo "Testing..."
cargo test

echo "Building..."
rustup run nightly cargo build --release

echo "Restarting..."
sudo systemctl restart jv.service

echo "Recording deploy info..."
{ date +%s; git rev-parse --short HEAD; } > deploy_info

echo "Done."
