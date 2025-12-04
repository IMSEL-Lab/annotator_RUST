# Redesign Plan: Material 3 GUI (Frontend Only)

This document outlines the plan to redesign the Slint GUI using the **official Slint Material 3 library**, while strictly preserving the existing Rust interface to avoid modifying application logic.

## 1. Strategy: "Frontend-First" Redesign

We will replace the custom UI components with official Material 3 widgets.
**Constraint:** The `src/` directory (Rust logic) will NOT be restructured.
**Requirement:** `ui/appwindow.slint` must maintain its current public API (properties and callbacks) so `main.rs` continues to work without changes.

## 2. Material 3 Integration

### **Integration Steps**
1.  **Source:** Copy the `material-1.0` folder from the official `slint-ui/material-rust-template` into `ui/material`.
2.  **Configuration:** Update `build.rs` *only* to map the `material` library path:
    ```rust
    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        std::collections::HashMap::from([(
            "material".to_string(),
            std::path::Path::new("ui/material/material.slint").to_path_buf(),
        )]),
    );
    ```

## 3. Component Redesign (UI Only)

We will refactor `ui/` files to use `import { ... } from "@vtable/material/material.slint";`.

### **A. Top Bar (Replaces MenuBar)**
*   **Visual:** Material 3 `TopAppBar` style.
*   **Implementation:** A new component wrapping Material Buttons/IconButtons.
*   **Connection:** Buttons will trigger the *existing* callbacks (e.g., `root.open-dataset()`).

### **B. Navigation Rail (Replaces Sidebar)**
*   **Visual:** A vertical rail for tool selection (`BBox`, `Point`, `Polygon`).
*   **Logic:** Clicking a rail item updates `root.current-tool`, preserving the existing binding.
*   **Classes:** A "Drawer" or specific panel for class selection, using Material Lists.

### **C. Dialogs**
*   **Settings:** Redesign `SettingsDialog` using standard Material `Switch`, `Button`, and `slider` (if available) or custom inputs styled to match.

## 4. Execution Steps

1.  **Setup:**
    -   Copy `material-1.0` library to `ui/material`.
    -   Configure `build.rs`.

2.  **Component Replacement (Iterative):**
    -   **Step 1:** Create `ui/components/top_bar.slint` (Material version of MenuBar).
    -   **Step 2:** Create `ui/components/nav_rail.slint` (Material version of Sidebar).
    -   **Step 3:** Update `ui/appwindow.slint` to import and use these new components.
    -   **Step 4:** Style the "Canvas" overlays (selection boxes, etc.) to use Material colors.

3.  **Validation:**
    -   Run `cargo run` to verify the UI looks new but behaves exactly as before.
