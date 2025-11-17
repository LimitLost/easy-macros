#!/usr/bin/env bash

# Script to run tests with local path dependencies instead of version dependencies
# This temporarily replaces version = "..." with path = "..." for internal crates,
# runs tests, then restores the original Cargo.toml files.

set -e

# Get the project root directory (parent of scripts folder)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKUP_DIR="$PROJECT_ROOT/.cargo-toml-backups"
QUIET_MODE=false

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to create backups of all Cargo.toml files
backup_cargo_tomls() {
    echo -e "${YELLOW}Creating backups of Cargo.toml files...${NC}"
    
    # Create backup directory
    mkdir -p "$BACKUP_DIR"
    
    # Find and backup all Cargo.toml files
    while IFS= read -r -d '' toml_file; do
        # Get relative path from project root
        rel_path="${toml_file#$PROJECT_ROOT/}"
        backup_path="$BACKUP_DIR/$rel_path"
        
        # Create backup directory structure
        mkdir -p "$(dirname "$backup_path")"
        
        # Copy the file
        cp "$toml_file" "$backup_path"
    done < <(find "$PROJECT_ROOT" -name "Cargo.toml" -not -path "*/target/*" -not -path "*/.cargo-toml-backups/*" -print0)
    
    echo -e "${GREEN}✓ Backups created${NC}"
}

# Function to restore all Cargo.toml files from backup
restore_cargo_tomls() {
    if [ "$QUIET_MODE" = true ]; then
        # Silent mode - no output
        if [ -d "$BACKUP_DIR" ]; then
            while IFS= read -r -d '' backup_file; do
                rel_path="${backup_file#$BACKUP_DIR/}"
                original_path="$PROJECT_ROOT/$rel_path"
                cp "$backup_file" "$original_path"
            done < <(find "$BACKUP_DIR" -name "Cargo.toml" -print0)
            rm -rf "$BACKUP_DIR"
        fi
    else
        echo -e "${YELLOW}Restoring original Cargo.toml files...${NC}"
        
        if [ -d "$BACKUP_DIR" ]; then
            # Find and restore all backed up files
            while IFS= read -r -d '' backup_file; do
                # Get relative path from backup dir
                rel_path="${backup_file#$BACKUP_DIR/}"
                original_path="$PROJECT_ROOT/$rel_path"
                
                # Restore the file
                cp "$backup_file" "$original_path"
            done < <(find "$BACKUP_DIR" -name "Cargo.toml" -print0)
            
            # Clean up backup directory
            rm -rf "$BACKUP_DIR"
            
            echo -e "${GREEN}✓ Original files restored${NC}"
        else
            echo -e "${RED}✗ No backup directory found${NC}"
        fi
    fi
}

# Function to replace version with path for internal dependencies
replace_versions_with_paths() {
    echo -e "${YELLOW}Replacing version dependencies with path dependencies...${NC}"
    
    # Map of package names to their directory paths
    declare -A package_paths=(
        ["easy-macros-all-syntax-cases"]="all-syntax-cases"
        ["easy-macros-all-syntax-cases-helpers"]="all-syntax-cases-helpers"
        ["easy-macros-always-context"]="always-context"
        ["easy-macros-always-context-build"]="always-context-build"
        ["easy-macros-anyhow-result"]="anyhow-result"
        ["easy-macros-attributes"]="attributes"
        ["easy-macros-attributes-macros"]="attributes-macros"
        ["easy-macros-context-internal"]="context-internal"
        ["easy-macros-helpers"]="helpers"
        ["easy-macros-proc-macro-tests"]="proc_macro_tests"
    )
    
    # Process each Cargo.toml file
    while IFS= read -r -d '' toml_file; do
        # Get the directory of the current Cargo.toml
        toml_dir="$(dirname "$toml_file")"
        
        # Calculate relative path to project root
        rel_to_root=$(realpath --relative-to="$toml_dir" "$PROJECT_ROOT")
        
        # Process each package
        for package_name in "${!package_paths[@]}"; do
            local_path="${package_paths[$package_name]}"
            path_from_current="$rel_to_root/$local_path"
            
            # Use sed to replace version with path for this package
            # This handles various formats:
            # - { package = "...", version = "...", ... } -> add path
            # - {package = "...",version = "...", ... } -> add path (no spaces)
            
            # Pattern 1: package = "NAME", version = "VERSION"
            sed -i -E "s|(package\s*=\s*\"$package_name\"\s*,\s*)version\s*=\s*\"[^\"]+\"|\1path = \"$path_from_current\"|g" "$toml_file"
            
            # Pattern 2: package = "NAME",version = "VERSION" (no space after comma)
            sed -i -E "s|(package\s*=\s*\"$package_name\",)version\s*=\s*\"[^\"]+\"|\1path = \"$path_from_current\"|g" "$toml_file"
        done
    done < <(find "$PROJECT_ROOT" -name "Cargo.toml" -not -path "*/target/*" -not -path "*/.cargo-toml-backups/*" -print0)
    
    echo -e "${GREEN}✓ Dependencies replaced with local paths${NC}"
}

# Function to check if we're already using local dependencies
check_if_using_local_deps() {
    # Check if backup directory exists (indicates we're using local deps)
    if [ -d "$BACKUP_DIR" ]; then
        return 0  # Already using local deps
    fi
    
    # Also check if any Cargo.toml has path dependencies to our internal packages
    while IFS= read -r -d '' toml_file; do
        if grep -q 'package = "easy-macros-.*", *path = ' "$toml_file"; then
            return 0  # Found local path dependency
        fi
    done < <(find "$PROJECT_ROOT" -name "Cargo.toml" -not -path "*/target/*" -not -path "*/.cargo-toml-backups/*" -print0)
    
    return 1  # Not using local deps
}

# Check if we're already using local dependencies
if check_if_using_local_deps; then
    echo -e "${YELLOW}=========================================${NC}"
    echo -e "${YELLOW}Already using local path dependencies!${NC}"
    echo -e "${YELLOW}=========================================${NC}"
    echo
    echo -e "${YELLOW}Running tests with current (local) dependencies...${NC}"
    echo
    
    # Run tests without any setup/teardown
    if [ -n "$1" ]; then
        bash "$1"
        TEST_RESULT=$?
    else
        bash "$PROJECT_ROOT/scripts/test.sh"
        TEST_RESULT=$?
    fi
    
    # Display results
    if [ $TEST_RESULT -eq 0 ]; then
        echo
        echo -e "${GREEN}=========================================${NC}"
        echo -e "${GREEN}Tests completed successfully!${NC}"
        echo -e "${GREEN}=========================================${NC}"
    else
        echo
        echo -e "${RED}=========================================${NC}"
        echo -e "${RED}Tests failed!${NC}"
        echo -e "${RED}=========================================${NC}"
    fi
    
    exit $TEST_RESULT
fi

# Trap to ensure restoration happens even if script fails
trap 'restore_cargo_tomls' EXIT

# Main execution
# Capture setup output
SETUP_OUTPUT=$(mktemp)
QUIET_MODE=true

{
    # Step 1: Backup
    backup_cargo_tomls
    
    # Step 2: Replace dependencies
    replace_versions_with_paths
} > "$SETUP_OUTPUT" 2>&1

QUIET_MODE=false

# Step 3: Run tests
# Check if a specific test script was passed as argument
if [ -n "$1" ]; then
    # Run specific test script
    bash "$1"
    TEST_RESULT=$?
else
    # Run the main test script
    bash "$PROJECT_ROOT/scripts/test.sh"
    TEST_RESULT=$?
fi

# Display results
if [ $TEST_RESULT -eq 0 ]; then
    # Tests passed
    echo
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN}Tests completed successfully!${NC}"
    echo -e "${GREEN}=========================================${NC}"
else
    # Tests failed - show setup output for debugging
    echo
    echo -e "${RED}=========================================${NC}"
    echo -e "${RED}Setup output (for debugging):${NC}"
    echo -e "${RED}=========================================${NC}"
    cat "$SETUP_OUTPUT"
    echo
    echo -e "${RED}=========================================${NC}"
    echo -e "${RED}Tests failed!${NC}"
    echo -e "${RED}=========================================${NC}"
fi

# Clean up temp files
rm -f "$SETUP_OUTPUT"

# Exit with the test result
exit $TEST_RESULT

# Note: Restoration happens automatically via the EXIT trap
