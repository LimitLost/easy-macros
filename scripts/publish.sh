#!/usr/bin/env bash

# Script to publish a selected crate from the easy-macros project
# Allows interactive selection and runs cargo publish

set -e  # Exit on error

# Get the project root directory (parent of scripts folder)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Define all crates with their directories and package names
declare -A CRATES
CRATES["-main-crate"]="easy-macros"
CRATES["all-syntax-cases"]="easy-macros-all-syntax-cases"
CRATES["all-syntax-cases-helpers"]="easy-macros-all-syntax-cases-helpers"
CRATES["always-context"]="easy-macros-always-context"
CRATES["always-context-build"]="easy-macros-always-context-build"
CRATES["anyhow-result"]="easy-macros-anyhow-result"
CRATES["attributes"]="easy-macros-attributes"
CRATES["attributes-macros"]="easy-macros-attributes-macros"
CRATES["context-internal"]="easy-macros-context-internal"
CRATES["helpers"]="easy-macros-helpers"
CRATES["proc_macro_tests"]="easy-macros-proc-macro-tests"

# Sort the crate directories for consistent display
SORTED_DIRS=($(printf '%s\n' "${!CRATES[@]}" | sort))

# Check if a crate directory was provided as an argument
if [[ -n "$1" ]]; then
    SELECTED_DIR="$1"
    if [[ ! -v "CRATES[$SELECTED_DIR]" ]]; then
        echo "Error: Unknown crate '$SELECTED_DIR'"
        echo "Available crates: ${!CRATES[@]}"
        exit 1
    fi
else
    # Interactive selection
    echo "Select a crate to publish:"
    echo "=========================="
    
    # Display numbered list
    i=1
    for dir in "${SORTED_DIRS[@]}"; do
        printf "%2d) %-30s (%s)\n" "$i" "$dir" "${CRATES[$dir]}"
        ((i++))
    done
    
    echo ""
    echo "Enter number [1-${#CRATES[@]}], 'all' to publish all, or 'q' to quit:"
    read -r selection
    
    # Handle quit
    if [[ "$selection" == "q" ]] || [[ "$selection" == "Q" ]]; then
        echo "Cancelled."
        exit 0
    fi
    
    # Handle 'all' option
    if [[ "$selection" == "all" ]] || [[ "$selection" == "ALL" ]]; then
        PUBLISH_ALL=true
    else
        # Validate numeric selection
        if ! [[ "$selection" =~ ^[0-9]+$ ]] || [[ "$selection" -lt 1 ]] || [[ "$selection" -gt "${#CRATES[@]}" ]]; then
            echo "Error: Invalid selection '$selection'"
            exit 1
        fi
        
        # Get the selected directory
        SELECTED_DIR="${SORTED_DIRS[$((selection - 1))]}"
    fi
fi

# Check if dry-run mode is requested
DRY_RUN=""
if [[ "${DRY_RUN_MODE:-}" == "true" ]] || [[ "$2" == "--dry-run" ]]; then
    DRY_RUN="--dry-run"
    echo "Running in DRY-RUN mode (no actual publishing will occur)"
    echo ""
fi

# Function to publish a single crate
publish_crate() {
    local dir="$1"
    local package_name="${CRATES[$dir]}"
    local crate_path="$PROJECT_ROOT/$dir"
    
    echo "========================================="
    echo "Publishing: $package_name"
    echo "Directory: $dir"
    echo "Path: $crate_path"
    echo "========================================="
    
    if [[ ! -d "$crate_path" ]]; then
        echo "Error: Directory does not exist: $crate_path"
        return 1
    fi
    
    if [[ ! -f "$crate_path/Cargo.toml" ]]; then
        echo "Error: Cargo.toml not found in: $crate_path"
        return 1
    fi
    
    # Show current version
    echo ""
    echo "Current version:"
    grep "^version = " "$crate_path/Cargo.toml" || echo "Version not found"
    echo ""
    
    # Run cargo publish
    if (cd "$crate_path" && cargo publish $DRY_RUN); then
        echo "✓ Successfully published $package_name"
        return 0
    else
        echo "✗ Failed to publish $package_name"
        return 1
    fi
}

# Publish crates
if [[ "${PUBLISH_ALL:-}" == "true" ]]; then
    echo "Publishing all crates in dependency order..."
    echo ""
    
    FAILED_CRATES=()
    
    # Publish in a reasonable dependency order (helpers first, main crate last)
    for dir in "${SORTED_DIRS[@]}"; do
        if ! publish_crate "$dir"; then
            FAILED_CRATES+=("$dir")
        fi
        echo ""
    done
    
    # Summary
    echo "========================================="
    echo "Publishing complete!"
    if [[ ${#FAILED_CRATES[@]} -eq 0 ]]; then
        echo "✓ All crates published successfully"
    else
        echo "✗ Failed to publish ${#FAILED_CRATES[@]} crate(s):"
        for failed in "${FAILED_CRATES[@]}"; do
            echo "  - $failed (${CRATES[$failed]})"
        done
        exit 1
    fi
else
    # Publish single crate
    publish_crate "$SELECTED_DIR"
fi

echo ""
echo "Done!"
