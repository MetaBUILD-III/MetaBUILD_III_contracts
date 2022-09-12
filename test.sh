#!/bin/bash
set -e

cargo test --manifest-path ./Cargo.toml -- --nocapture
