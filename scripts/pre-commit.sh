#!/bin/bash
# Pre-commit hook to verify code quality before committing

set -e

# Format outputs nicely
RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m' # No Color

echo -e "=== Running Local Pre-Commit Verification ==="

echo -e "Checking code formatting..."
if ! cargo fmt --check; then
    echo -e "${RED}Formatting checks failed! Run 'cargo fmt' to fix.${NC}"
    exit 1
fi

echo -e "Running Clippy linter..."
if ! cargo clippy --all-targets -- -D warnings; then
    echo -e "${RED}Clippy linter found issues! Fix the warnings above.${NC}"
    exit 1
fi

echo -e "Running all tests..."
if ! cargo test --all-targets --workspace; then
    echo -e "${RED}Tests failed! Fix the failures above.${NC}"
    exit 1
fi

echo -e "${GREEN}=== All checks passed! Ready to commit. ===${NC}"
exit 0
