#!/bin/sh
set -e

# Read first cli argument and default to 5000 otherwise.
iterations="${1:-5000}"
cargo build --profile profiling --bin benchmark
exec samply record target/profiling/benchmark --profile-long "$iterations"
