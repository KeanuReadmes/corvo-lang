#!/bin/bash 
cargo build --release
cp target/release/corvo ~/.local/bin/corvo
echo "Corvo installed successfully"