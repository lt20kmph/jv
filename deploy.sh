#!/bin/bash

echo "Deploying..."
git pull

echo "Building..."
rustup run nightly cargo build --release

echo "Restarting..."
sudo systemctl restart jv.service
