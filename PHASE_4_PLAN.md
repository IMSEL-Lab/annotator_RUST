# Phase 4 Implementation Plan

## Overview
Phase 4 adds interactive annotation creation, deletion, and classification while maintaining all Phase 0-3 functionality.

## Implementation Strategy

We'll implement in this order (simplest to most complex):
1. ✅ Data model extensions
2. ✅ Tool state management
3. ✅ Bbox creation (B key)
4. ✅ Point creation (C key)
5. ✅ Fast deletion (Q+click, double-click)
6. ✅ Classification (digit keys)
7. ✅ Visual feedback (preview, status)
8. ✅ Polygon creation (S key)
9. ✅ Testing & refinement

---

## Changes Needed

### 1. Data Model (Slint struct)
**File**: `ui/appwindow.slint`

```slint
struct Annotation {
    id: int,
    type: string,
    x: float,
    y: float,
    width: float,
    height: float,
    rotation: float,
    selected: bool,
    class: int,  // NEW: classification (0-999 for multi-digit)
}
```

### 2. Tool State (Rust)
**File**: `src/main.rs`

Add state management:
```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Debug)]
enum ToolState {
    Neutral,
    BboxActive,  // B key held
    PointActive, // C key held
    PolyActive,  // S key held
}

struct DrawingState {
    tool: ToolState,
    start_x: f32,
    start_y: f32,
    current_class: i32,
    poly_vertices: Vec<(f32, f32)>,
}
```

### 3. UI Additions
**File**: `ui/appwindow.slint`

Add properties and callbacks:
```slint
export component AppWindow inherits Window {
    // Existing...

    // NEW: Tool state
    in-out property <string> current-tool: "Neutral";
    in-out property <int> current-class: 1;

    // NEW: Preview shape (while drawing)
    in-out property <bool> show-preview: false;
    in-out property <float> preview-x;
    in-out property <float> preview-y;
    in-out property <float> preview-width;
    in-out property <float> preview-height;

    // NEW: Callbacks
    callback key-pressed(string);
    callback key-released(string);
    callback mouse-pressed-at(float, float);
    callback mouse-moved-to(float, float);
    callback mouse-released-at(float, float);
    callback delete-annotation(int);
    callback set-annotation-class(int, int);
}
```

### 4. Keyboard Handling
**Approach**: Use Slint's FocusScope and key event handlers

```slint
FocusScope {
    key-pressed(event) => {
        if (event.text == "b") { root.key-pressed("B"); }
        if (event.text == "c") { root.key-pressed("C"); }
        if (event.text == "s") { root.key-pressed("S"); }
        if (event.text == "q") { root.key-pressed("Q"); }
        // ... digits 1-5
        accept
    }

    key-released(event) => {
        root.key-released(event.text);
        accept
    }
}
```

### 5. Mouse Coordinate Conversion
Critical function - convert screen coordinates to image coordinates:

```rust
fn screen_to_image_coords(
    screen_x: f32,
    screen_y: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32
) -> (f32, f32) {
    let img_x = (screen_x - pan_x) / zoom;
    let img_y = (screen_y - pan_y) / zoom;
    (img_x, img_y)
}
```

---

## Implementation Steps

### Step 1: Extend Data Model ✅
- Add `class: int` field to Annotation struct
- Update all existing test annotations to include class: 1
- Verify compilation

### Step 2: Add Tool State Management ✅
- Create ToolState enum and DrawingState struct
- Initialize in main.rs
- Add keyboard callbacks to UI
- Wire up keyboard handlers in Rust
- Test: Press B/C/S, status bar should show tool name

### Step 3: Implement Bbox Creation ✅
**Workflow**:
1. User holds B key → tool = BboxActive
2. User presses mouse → record start position
3. User drags → update preview rectangle
4. User releases mouse → create annotation, tool = Neutral
5. User releases B → ensure tool = Neutral

**Code additions**:
- Keyboard handler sets tool state
- Mouse press captures start position (image coords)
- Mouse move updates preview (if tool active)
- Mouse release creates annotation

### Step 4: Implement Point Creation ✅
**Workflow**:
1. User holds C key → tool = PointActive
2. User clicks → create point annotation at click position
3. Tool → Neutral

**Simpler than bbox** - just need click position.

### Step 5: Implement Fast Deletion ✅
**Two methods**:

**Q + Click**:
- Track if Q key is held
- On click, if Q is held, delete annotation at cursor
- Use existing hit-testing (TouchArea callbacks)

**Double-click**:
- Track click timing
- If two clicks within 300ms on same annotation, delete it

### Step 6: Implement Classification ✅
**Workflow**:
1. User hovers annotation OR has one selected
2. User presses digit 1-5
3. Annotation's class field is updated
4. Optional: visual indication (color/label) updates

**Multi-digit support**:
- Buffer recent digit presses (e.g., 500ms window)
- Combine into multi-digit number (e.g., "2" then "3" → class 23)

### Step 7: Add Visual Feedback ✅
**Preview shapes**:
- While drawing bbox: show preview rectangle
- While drawing polygon: show polyline of vertices
- Cursor indication: change cursor or show text near cursor

**Status bar**:
- "Neutral" | "BBox Tool (B)" | "Point Tool (C)" | "Polygon Tool (S)"
- "Class: 2-3" when classified

### Step 8: Implement Polygon Creation ✅
**Workflow** (most complex):
1. User holds S → tool = PolyActive
2. User clicks → add vertex to list
3. User clicks more → add more vertices
4. User presses Enter → close polygon, create annotation
5. User presses Esc → cancel, clear vertices, tool = Neutral

**Challenges**:
- Need to store vertex list during creation
- Need to render preview polyline
- Need Enter/Esc handling
- Need to create polygon annotation type

---

## Testing Strategy

After each step, test:
1. Feature works as expected
2. Tool returns to Neutral after use
3. Pan/zoom still work
4. Phase 3 selection still works
5. No crashes or hangs

Create test executables similar to Phase 3 tests.

---

## Potential Issues & Solutions

### Issue 1: Keyboard events in Slint
**Problem**: Slint keyboard handling can be tricky
**Solution**: Use FocusScope at root level, ensure it has focus

### Issue 2: Mouse coordinates
**Problem**: Need to convert screen → image coords correctly
**Solution**: Use existing pan-x, pan-y, zoom-level properties

### Issue 3: Tool state synchronization
**Problem**: Slint UI and Rust state need to stay in sync
**Solution**: Use callbacks and properties bidirectionally

### Issue 4: Polygon type doesn't exist yet
**Problem**: Current struct only supports bbox, rbbox, point
**Solution**: Add "polygon" type with vertices list

---

## File Changes Summary

**Modified**:
- `ui/appwindow.slint` - Add tool state, callbacks, keyboard handling, preview
- `src/main.rs` - Add state management, creation logic, deletion, classification

**No changes needed**:
- `build.rs` - Remains the same
- `Cargo.toml` - No new dependencies

---

## Estimated Complexity

- **Bbox creation**: Medium (coordinate conversion, preview)
- **Point creation**: Easy (just click position)
- **Deletion**: Easy (use existing hit-testing)
- **Classification**: Easy (just update field)
- **Polygon creation**: Hard (vertex management, Enter/Esc handling)
- **Visual feedback**: Medium (preview rendering)

**Total estimate**: 3-4 hours of focused work

---

## Next Steps

1. ✅ Get user approval for this plan
2. Start with Step 1 (extend data model)
3. Implement incrementally, testing after each step
4. Create test executables for Phase 4
5. Document any issues encountered

Ready to proceed?
