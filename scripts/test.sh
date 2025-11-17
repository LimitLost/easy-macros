#!/usr/bin/env bash

# Script to run test.sh in all subfolders of the project (non-recursive)
# Skips the scripts folder itself

# Get the project root directory (parent of scripts folder)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Running tests in project: $PROJECT_ROOT"
echo "========================================="

# Track if any tests were run
TESTS_RUN=0
TESTS_FAILED=0
FAILED_FOLDERS=()

# Iterate through all directories in the project root
for dir in "$PROJECT_ROOT"/*/ ; do
    # Get the directory name without trailing slash
    dir_name=$(basename "$dir")
    
    # Skip the scripts folder and hidden directories
    if [[ "$dir_name" == "scripts" ]] || [[ "$dir_name" == .* ]] || [[ "$dir_name" == "target" ]]; then
        continue
    fi
    
    # Check if test.sh exists in this directory
    if [[ -f "$dir/test.sh" ]]; then
        # Capture output to a temporary file
        temp_output=$(mktemp)
        
        # Run the test script and capture all output
        if (cd "$dir" && bash test.sh) > "$temp_output" 2>&1; then
            echo "✓ Tests passed in $dir_name"
            TESTS_RUN=$((TESTS_RUN + 1))
        else
            echo ""
            echo "✗ Tests failed in $dir_name"
            echo "-----------------------------------------"
            cat "$temp_output"
            echo "-----------------------------------------"
            TESTS_RUN=$((TESTS_RUN + 1))
            TESTS_FAILED=$((TESTS_FAILED + 1))
            FAILED_FOLDERS+=("$dir_name")
        fi
        
        # Clean up temporary file
        rm -f "$temp_output"
    fi
done

echo ""
echo "========================================="
if [[ $TESTS_RUN -eq 0 ]]; then
    echo "No test.sh files found in subfolders"
else
    echo "Ran tests in $TESTS_RUN folder(s)"
    if [[ $TESTS_FAILED -eq 0 ]]; then
        echo "✓ All tests passed!"
    else
        echo "✗ $TESTS_FAILED folder(s) had failing tests"
        echo "Failed folders:"
        for folder in "${FAILED_FOLDERS[@]}"; do
            echo "  - $folder"
        done
        exit 1
    fi
fi
