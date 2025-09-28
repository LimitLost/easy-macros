#!/bin/bash

# test.sh - Run tests for each feature in the helpers crate

set -e

echo "========================================="
echo "Running tests for helpers crate features"
echo "========================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run tests with a specific feature
run_test() {
    local feature_name=$1
    if [ -z "$3" ]; then
        local feature_flag=${2:-$feature_name}
    else
        local feature_flag=$2
    fi
    local arg=${3:-"--no-default-features --features"}
    
    echo -e "${BLUE}Testing feature: ${feature_name}${NC}"
    echo "----------------------------------------"
    
    if cargo test ${arg} ${feature_flag}; then
        echo -e "${GREEN}✓ ${feature_name} tests passed${NC}"
    else
        echo -e "${RED}✗ ${feature_name} tests failed${NC}"
        exit 1
    fi
    echo
}

# Run tests with no features (default)
run_test "no features (default)" "" "--no-default-features"

# Run tests for each individual feature
run_test "context"
run_test "indexed-name"
run_test "tokens-builder"
run_test "expr-error-wrap"
run_test "readable-token-stream"
run_test "find-crate"
run_test "token-stream-consistent"
run_test "parse-macro-input"

# Run tests with all features enabled
run_test "full (all features)" "full"

echo -e "${GREEN}========================================="
echo "All feature tests completed successfully!"
echo -e "=========================================${NC}"