#!/bin/bash

# No tests yet, run build for every feature now

cargo build --no-default-features
cargo build --features easy-sql