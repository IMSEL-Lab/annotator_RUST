# Annotator Phase 3 Test Suite Documentation

## Overview

This test suite implements comprehensive testing for the annotation overlay system without significantly modifying the core codebase. Tests are organized as separate executable binaries that can be run individually or in groups.

## Test Architecture

### Test Structure
- **Test Helpers** (`tests/test_helpers.rs`): Shared utilities for creating test scenarios
- **Test Binaries**: Individual executable tests in `tests/` directory
- **Test Runner**: Shell script (`run_tests.sh`) for convenient test execution

### Running Tests

#### Option 1: Interactive Test Runner (Recommended)
```bash
./run_tests.sh
```

This provides a menu-driven interface to:
- Run tests by category
- Run individual tests
- Build all tests

#### Option 2: Run Individual Tests
```bash
cargo run --bin <test_name>
```

Example:
```bash
cargo run --bin p3_010_select_bbox
```

#### Option 3: Build All Tests
```bash
cargo build --bins
```

## Test Categories

### 1. Selection and Hit-Testing Behavior (P3-010 to P3-015)

#### P3-010: Select Single BBox
**File**: `tests/p3_010_select_bbox.rs`
**Command**: `cargo run --bin p3_010_select_bbox`

**Scenario**: Click inside an axis-aligned bounding box
**Expected**:
- Clicked bbox turns GREEN (selected)
- No other annotations are selected
- Clicking empty space deselects (turns RED)

**Manual Test Steps**:
1. Launch the test
2. Click inside the red rectangle
3. Verify it turns green
4. Click outside
5. Verify it turns red

---

#### P3-011: Select Center Point
**File**: `tests/p3_011_select_point.rs`
**Command**: `cargo run --bin p3_011_select_point`

**Scenario**: Click on point markers with hit radius testing
**Expected**:
- Clicking on/near a point selects it (turns GREEN)
- Clicking clearly away from points does NOT select
- Only one point selected at a time

**Manual Test Steps**:
1. Launch the test (3 red dots appear)
2. Click each dot - should turn green
3. Try clicking near but not on a dot - should not select
4. Click empty space to deselect

---

#### P3-012: Deselect by Canvas Click
**File**: `tests/p3_012_deselect_canvas.rs`
**Command**: `cargo run --bin p3_012_deselect_canvas`

**Scenario**: Clicking empty canvas deselects current selection
**Expected**:
- Pre-selected annotation starts GREEN
- Clicking empty canvas turns it RED
- Visual selected style disappears

**Manual Test Steps**:
1. Launch test (green rectangle visible)
2. Click empty canvas area
3. Verify rectangle turns red
4. Re-select and deselect multiple times

---

#### P3-013: Overlapping Selection
**File**: `tests/p3_013_overlapping_selection.rs`
**Command**: `cargo run --bin p3_013_overlapping_selection`

**Scenario**: Two overlapping boxes, test topmost selection
**Expected**:
- Clicking overlap region selects topmost box (Box 2)
- Clicking non-overlap area of Box 1 selects Box 1
- Behavior is consistent across multiple runs

**Manual Test Steps**:
1. Launch test (two overlapping red rectangles)
2. Click in overlapping region (center-right) - Box 2 should select
3. Click left side (non-overlapping) - Box 1 should select
4. Repeat to verify consistency

---

### 2. Integration with Pan and Zoom (P3-020 to P3-025)

#### P3-020: Initial Alignment
**File**: `tests/p3_020_initial_alignment.rs`
**Command**: `cargo run --bin p3_020_initial_alignment`

**Scenario**: Verify annotations align correctly at initial fit-to-view
**Expected**:
- All annotations visible at startup
- No visible offset or scale errors
- Boxes, rotated boxes, and points appear at correct positions

**Manual Test Steps**:
1. Launch test
2. Verify 4 annotations visible (2 boxes, 1 rotated box, 1 point)
3. Check alignment with image
4. Note positions for comparison with pan/zoom tests

---

#### P3-021: Pan Alignment
**File**: `tests/p3_021_pan_alignment.rs`
**Command**: `cargo run --bin p3_021_pan_alignment`

**Scenario**: Pan image and verify annotations move exactly with it
**Expected**:
- Annotations move in perfect sync with image
- No lag, drift, or offset during panning
- Relative position between annotation and image content remains constant

**Manual Test Steps**:
1. Launch test
2. Note the box and point positions
3. Click and drag to pan in various directions
4. Verify annotations move exactly with the image
5. Try rapid panning and smooth panning
6. Check rotated box alignment

---

#### P3-022: Zoom Alignment
**File**: `tests/p3_022_zoom_alignment.rs`
**Command**: `cargo run --bin p3_022_zoom_alignment`

**Scenario**: Zoom in/out and verify annotations scale proportionally
**Expected**:
- Annotations scale with image zoom
- No parallax or offset
- Rotated boxes scale AND rotate correctly
- Points remain visible at all zoom levels

**Manual Test Steps**:
1. Launch test
2. Scroll wheel to zoom in several steps
3. Verify all annotations scale proportionally
4. Verify they stay over same image features
5. Zoom out several steps
6. **CRITICAL**: Test rotated box (ID 2) scaling

---

### 3. Annotation Data Model and Dynamic Updates (P3-030 to P3-034)

#### P3-030: Add Annotation Programmatically
**File**: `tests/p3_030_add_annotation.rs`
**Command**: `cargo run --bin p3_030_add_annotation`

**Scenario**: Dynamically add annotations after startup
**Expected**:
- New annotations appear at correct positions
- Existing annotations remain unchanged
- No intermediate state or flicker

**Manual Test Steps**:
1. Launch test (starts with 1 box + 1 point)
2. After 2s: rotated box appears
3. After 4s: regular box appears
4. After 6s: point appears
5. Verify each appears correctly and others unchanged

---

#### P3-031: Remove Annotation Programmatically
**File**: `tests/p3_031_remove_annotation.rs`
**Command**: `cargo run --bin p3_031_remove_annotation`

**Scenario**: Dynamically remove annotations
**Expected**:
- Removed shapes disappear completely
- No ghost remnants
- Remaining annotations unaffected

**Manual Test Steps**:
1. Launch test (4 annotations visible)
2. After 2s: point disappears
3. After 4s: rotated box disappears
4. After 6s: only 2 regular boxes remain
5. Verify clean removal, no artifacts

---

### 4. Performance and Robustness (P3-040 to P3-041)

#### P3-040: Many Annotations Performance
**File**: `tests/p3_040_performance_many.rs`
**Command**: `cargo run --bin p3_040_performance_many`

**Scenario**: Load 1000+ annotations and test interaction
**Expected**:
- Pan/zoom remain smooth enough for use
- Frame rate may drop but stays usable
- No crashes or out-of-memory errors
- Selection still works

**Manual Test Steps**:
1. Launch test (loads 1000 annotations in grid)
2. Try panning - should remain interactive
3. Try zooming - may be slower but usable
4. Try selecting some annotations
5. Monitor CPU/memory if possible
6. **Note**: Can adjust `annotation_count` in test file

---

## Test Results Tracking

### Pass Criteria
- ✅ **PASS**: All expected behaviors observed
- ⚠️  **PARTIAL**: Most behaviors work, minor issues
- ❌ **FAIL**: Critical functionality broken

### Creating Test Reports

After running tests, document results in this format:

```
Test ID: P3-010
Date: 2024-XX-XX
Result: ✅ PASS
Notes: Selection works correctly, visual feedback clear
Issues: None

Test ID: P3-022
Date: 2024-XX-XX
Result: ⚠️ PARTIAL
Notes: Regular boxes scale correctly
Issues: Rotated box doesn't scale with zoom (CRITICAL BUG)
```

## Extending the Test Suite

### Adding New Tests

1. **Create test file** in `tests/` directory:
```rust
// tests/p3_XXX_test_name.rs
mod test_helpers;
use test_helpers::*;

fn main() -> Result<(), slint::PlatformError> {
    slint::include_modules!();
    let ui = AppWindow::new()?;
    // ... test implementation
    ui.run()
}
```

2. **Add to Cargo.toml**:
```toml
[[bin]]
name = "p3_XXX_test_name"
path = "tests/p3_XXX_test_name.rs"
```

3. **Update run_tests.sh** with new test option

### Helper Functions

Available in `tests/test_helpers.rs`:
- `create_bbox(id, x, y, w, h, selected)` - Create axis-aligned box
- `create_rbbox(id, x, y, w, h, rotation, selected)` - Create rotated box
- `create_point(id, x, y, selected)` - Create point annotation
- `create_basic_scene()` - Standard test scene
- `create_overlapping_boxes_scene()` - Overlapping boxes
- `create_rotated_boxes_scene()` - Multiple rotated boxes
- `create_large_scene(count)` - Large annotation set
- `count_selected(annotations)` - Count selected annotations
- `get_selected_id(annotations)` - Get selected annotation ID
- `verify_none_selected(annotations)` - Check no selection

## Troubleshooting

### Test Won't Compile
```bash
cargo clean
cargo build --bins
```

### Image Not Loading
Update image path in test file:
```rust
let image_path = std::path::PathBuf::from("/path/to/your/image.jpeg");
```

### Test Hangs
Some tests (P3-030, P3-031) use timers. Wait for completion or use Ctrl+C.

### Poor Performance
In P3-040, reduce `annotation_count` variable:
```rust
let annotation_count = 500; // Reduced from 1000
```

## CI/CD Integration

### Automated Testing
While these are manual interaction tests, you can automate building:

```bash
# In CI pipeline
cargo build --bins
# All tests build successfully = PASS
```

### Future Automation
Consider:
- Screenshot comparison tests
- Event injection for automated clicking
- Headless rendering for visual regression

## Implementation Notes

### Why Separate Binaries?
- No modification to main codebase
- Easy to run individual tests
- Clear test isolation
- Simple to understand and maintain

### Limitations
- Tests require manual interaction
- No automated pass/fail determination
- Requires visual verification
- Image path hardcoded (update as needed)

## Test Coverage Summary

| Category | Tests | Status |
|----------|-------|--------|
| Selection | 4 | ✅ Implemented |
| Pan/Zoom | 3 | ✅ Implemented |
| Dynamic Updates | 2 | ✅ Implemented |
| Performance | 1 | ✅ Implemented |
| **TOTAL** | **10** | **✅ Complete** |

## Additional Tests to Consider

The following tests from the spec can be added:

- **P3-014**: Selection independent of image content
- **P3-015**: Selection style stable under redraw
- **P3-023**: Combined zoom+pan sequences
- **P3-024**: Window resize with annotations
- **P3-025**: Selection across pan/zoom
- **P3-032**: Update annotation geometry
- **P3-033**: Update annotation style
- **P3-034**: Dynamic updates under pan/zoom
- **P3-041**: Frequent dynamic updates

Template files can be created using the existing tests as examples.
