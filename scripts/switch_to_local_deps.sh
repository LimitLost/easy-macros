#!/usr/bin/env bash

# Script to switch from version dependencies to local path dependencies
# This creates backups of Cargo.toml files and replaces version = "..." with path = "..."
# Use switch_to_version_deps.sh to revert back to version dependencies

set -e

# Get the project root directory (parent of scripts folder)
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKUP_DIR="$PROJECT_ROOT/.cargo-toml-backups"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to create backups of all Cargo.toml files
backup_cargo_tomls() {
    echo -e "${YELLOW}Creating backups of Cargo.toml files...${NC}"
    
    # Check if backups already exist
    if [ -d "$BACKUP_DIR" ]; then
        echo -e "${RED}✗ Backup directory already exists!${NC}"
        echo -e "${RED}  You may already be using local dependencies.${NC}"
        echo -e "${RED}  Use switch_to_version_deps.sh to restore, or manually delete:${NC}"
        echo -e "${RED}  $BACKUP_DIR${NC}"
        exit 1
    fi
    
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
    done < <(find "$PROJECT_ROOT" -name "Cargo.toml" -not -path "*/target/*" -not -path "*/.cargo-toml-backups/*" -print0) || true
    
    echo -e "${GREEN}✓ Backups created in $BACKUP_DIR${NC}"
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
    
    local changes_made=0
    
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
            # - { package = "...", version = "...", ... } -> replace with path
            # - {package = "...",version = "...", ... } -> replace with path (no spaces)
            
            # Pattern 1: package = "NAME", version = "VERSION"
            if sed -i -E "s|(package\s*=\s*\"$package_name\"\s*,\s*)version\s*=\s*\"[^\"]+\"|\1path = \"$path_from_current\"|g" "$toml_file"; then
                ((changes_made++)) || true
            fi
            
            # Pattern 2: package = "NAME",version = "VERSION" (no space after comma)
            if sed -i -E "s|(package\s*=\s*\"$package_name\",)version\s*=\s*\"[^\"]+\"|\1path = \"$path_from_current\"|g" "$toml_file"; then
                ((changes_made++)) || true
            fi
        done
    done < <(find "$PROJECT_ROOT" -name "Cargo.toml" -not -path "*/target/*" -not -path "*/.cargo-toml-backups/*" -print0) || true
    
    echo -e "${GREEN}✓ Dependencies replaced with local paths${NC}"
}

# Main execution
echo
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}Switching to Local Path Dependencies${NC}"
echo -e "${BLUE}=========================================${NC}"
echo

# Step 1: Backup
backup_cargo_tomls
echo

# Step 2: Replace dependencies
replace_versions_with_paths
echo

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}Successfully switched to local dependencies!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo
echo -e "${YELLOW}To switch back to version dependencies, run:${NC}"
echo -e "${YELLOW}  ./scripts/switch_to_version_deps.sh${NC}"
echo
