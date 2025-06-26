#!/bin/bash

echo "Deploying..."
git fetch && git reset --hard origin/main

echo "Testing..."
cargo test || { echo "Tests failed, aborting deployment."; exit 1; }

echo "Building..."
rustup run nightly cargo build --release

echo "Restarting..."
sudo systemctl restart jv.service
