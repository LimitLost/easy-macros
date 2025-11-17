#!/usr/bin/env bash

# Script to switch from local path dependencies back to version dependencies
# This restores the backed-up Cargo.toml files created by switch_to_local_deps.sh

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

# Function to restore all Cargo.toml files from backup
restore_cargo_tomls() {
    echo -e "${YELLOW}Restoring original Cargo.toml files...${NC}"
    
    if [ ! -d "$BACKUP_DIR" ]; then
        echo -e "${RED}✗ No backup directory found!${NC}"
        echo -e "${RED}  Backups are located at: $BACKUP_DIR${NC}"
        echo -e "${RED}  You may already be using version dependencies.${NC}"
        exit 1
    fi
    
    local files_restored=0
    
    # Find and restore all backed up files
    while IFS= read -r -d '' backup_file; do
        # Get relative path from backup dir
        rel_path="${backup_file#$BACKUP_DIR/}"
        original_path="$PROJECT_ROOT/$rel_path"
        
        # Restore the file
        cp "$backup_file" "$original_path"
        ((files_restored++))
    done < <(find "$BACKUP_DIR" -name "Cargo.toml" -print0) || true
    
    echo -e "${GREEN}✓ Restored $files_restored Cargo.toml file(s)${NC}"
    
    # Clean up backup directory
    rm -rf "$BACKUP_DIR"
    echo -e "${GREEN}✓ Removed backup directory${NC}"
}

# Main execution
echo
echo -e "${BLUE}=========================================${NC}"
echo -e "${BLUE}Switching to Version Dependencies${NC}"
echo -e "${BLUE}=========================================${NC}"
echo

# Restore from backups
restore_cargo_tomls
echo

echo -e "${GREEN}=========================================${NC}"
echo -e "${GREEN}Successfully switched to version dependencies!${NC}"
echo -e "${GREEN}=========================================${NC}"
echo
echo -e "${YELLOW}To switch back to local dependencies, run:${NC}"
echo -e "${YELLOW}  ./scripts/switch_to_local_deps.sh${NC}"
echo
