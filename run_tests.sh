#!/bin/bash
# Test runner for Annotator Phase 3 tests
# This script helps run individual tests or groups of tests

set -e

echo "╔════════════════════════════════════════════════════════╗"
echo "║     Annotator Phase 3 Test Suite Runner               ║"
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Color codes for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

function show_menu() {
    echo "Test Categories:"
    echo ""
    echo "  ${BLUE}1)${NC} Selection and Hit-Testing (P3-010 to P3-015)"
    echo "  ${BLUE}2)${NC} Pan/Zoom Integration (P3-020 to P3-025)"
    echo "  ${BLUE}3)${NC} Dynamic Updates (P3-030 to P3-034)"
    echo "  ${BLUE}4)${NC} Performance (P3-040 to P3-041)"
    echo "  ${BLUE}5)${NC} Run specific test by ID"
    echo "  ${BLUE}6)${NC} Build all tests"
    echo "  ${BLUE}q)${NC} Quit"
    echo ""
}

function run_selection_tests() {
    echo "${GREEN}Running Selection and Hit-Testing Tests...${NC}"
    echo ""
    echo "${YELLOW}Test P3-010: Select Single BBox${NC}"
    cargo run --bin p3_010_select_bbox
    echo ""
    echo "${YELLOW}Test P3-011: Select Point${NC}"
    cargo run --bin p3_011_select_point
    echo ""
    echo "${YELLOW}Test P3-012: Deselect Canvas${NC}"
    cargo run --bin p3_012_deselect_canvas
    echo ""
    echo "${YELLOW}Test P3-013: Overlapping Selection${NC}"
    cargo run --bin p3_013_overlapping_selection
}

function run_panzoom_tests() {
    echo "${GREEN}Running Pan/Zoom Integration Tests...${NC}"
    echo ""
    echo "${YELLOW}Test P3-020: Initial Alignment${NC}"
    cargo run --bin p3_020_initial_alignment
    echo ""
    echo "${YELLOW}Test P3-021: Pan Alignment${NC}"
    cargo run --bin p3_021_pan_alignment
    echo ""
    echo "${YELLOW}Test P3-022: Zoom Alignment${NC}"
    cargo run --bin p3_022_zoom_alignment
}

function run_dynamic_tests() {
    echo "${GREEN}Running Dynamic Update Tests...${NC}"
    echo ""
    echo "${YELLOW}Test P3-030: Add Annotation${NC}"
    cargo run --bin p3_030_add_annotation
    echo ""
    echo "${YELLOW}Test P3-031: Remove Annotation${NC}"
    cargo run --bin p3_031_remove_annotation
}

function run_performance_tests() {
    echo "${GREEN}Running Performance Tests...${NC}"
    echo ""
    echo "${YELLOW}Test P3-040: Many Annotations${NC}"
    cargo run --bin p3_040_performance_many
}

function run_specific_test() {
    echo "Available tests:"
    echo "  p3_010_select_bbox"
    echo "  p3_011_select_point"
    echo "  p3_012_deselect_canvas"
    echo "  p3_013_overlapping_selection"
    echo "  p3_020_initial_alignment"
    echo "  p3_021_pan_alignment"
    echo "  p3_022_zoom_alignment"
    echo "  p3_030_add_annotation"
    echo "  p3_031_remove_annotation"
    echo "  p3_040_performance_many"
    echo ""
    read -p "Enter test name: " test_name
    echo ""
    echo "${GREEN}Running test: ${test_name}${NC}"
    cargo run --bin "$test_name"
}

function build_all() {
    echo "${GREEN}Building all tests...${NC}"
    cargo build --bins
    echo ""
    echo "${GREEN}✓ All tests built successfully${NC}"
}

# Main loop
while true; do
    show_menu
    read -p "Select an option: " choice
    echo ""

    case $choice in
        1)
            run_selection_tests
            ;;
        2)
            run_panzoom_tests
            ;;
        3)
            run_dynamic_tests
            ;;
        4)
            run_performance_tests
            ;;
        5)
            run_specific_test
            ;;
        6)
            build_all
            ;;
        q|Q)
            echo "Exiting test runner."
            exit 0
            ;;
        *)
            echo "${YELLOW}Invalid option. Please try again.${NC}"
            ;;
    esac

    echo ""
    echo "Press Enter to continue..."
    read
    clear
done
