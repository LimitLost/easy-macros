#!/bin/bash

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

bash "$PROJECT_ROOT/scripts/build_readme_docs.sh"
bash "$PROJECT_ROOT/scripts/readme_join.sh"
bash "$PROJECT_ROOT/scripts/test_with_local_deps.sh"