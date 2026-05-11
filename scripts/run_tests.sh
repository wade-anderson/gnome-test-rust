#!/bin/bash

# Unified Test Runner for gnome-test-rust

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}=== Starting Unified Test Suite ===${NC}"

# 1. Unit Tests
echo -e "\n${GREEN}[1/3] Running Unit Tests...${NC}"
cargo test
if [ $? -eq 0 ]; then
    echo -e "${GREEN}Unit Tests PASSED${NC}"
else
    echo -e "${RED}Unit Tests FAILED${NC}"
    exit 1
fi

# 2. Coverage
echo -e "\n${GREEN}[2/3] Running Coverage Analysis...${NC}"
cargo tarpaulin
if [ $? -eq 0 ]; then
    echo -e "${GREEN}Coverage Analysis COMPLETED${NC}"
else
    echo -e "${RED}Coverage Analysis FAILED${NC}"
    exit 1
fi

# 3. E2E Tests
echo -e "\n${GREEN}[3/3] Running End-to-End Tests...${NC}"
python3 tests/e2e_test.py
if [ $? -eq 0 ]; then
    echo -e "${GREEN}E2E Tests PASSED${NC}"
else
    echo -e "${RED}E2E Tests FAILED${NC}"
    exit 1
fi

echo -e "\n${GREEN}=== ALL TESTS PASSED ===${NC}"
