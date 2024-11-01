#!/bin/bash

echo "Deploying..."
git fetch && git reset --hard origin/main

echo "Building..."
rustup run nightly cargo build --release

echo "Restarting..."
sudo systemctl restart jv.service
