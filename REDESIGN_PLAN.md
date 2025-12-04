# Redesign Plan: Material 3 & Structural Improvements

This document outlines the plan to restructure the project files for better maintainability and redesign the Slint GUI to follow Google's Material 3 (Material You) guidelines.

## 1. File Structure Reorganization

The current structure is relatively flat. We will adopt a more layered architecture to separate concerns (UI vs. Logic vs. Data).

### **Proposed Directory Layout**

```text
annotator/
├── Cargo.toml
├── build.rs
├── src/
│   ├── main.rs            # Entry point: Application lifecycle & window creation
│   ├── app.rs             # Main application controller/state logic
│   ├── model/             # Data structures (pure Rust)
│   │   ├── mod.rs
│   │   ├── annotation.rs  # Annotation structs & helpers
│   │   ├── dataset.rs     # Dataset loading/saving logic
│   │   ├── config.rs      # App configuration
│   │   └── classes.rs     # Class definitions
│   ├── service/           # Business logic & algorithms
│   │   ├── mod.rs
│   │   ├── export/        # Export logic (COCO, VOC)
│   │   ├── auto_resize.rs # Auto-resize algorithms
│   │   └── history.rs     # Undo/Redo stack implementation
│   └── ui_bridge/         # Rust-side UI callbacks & state mapping
│       └── mod.rs         # Connects 'app.rs' logic to Slint callbacks
└── ui/                    # Slint UI Definitions
    ├── app_window.slint   # Root window
    ├── assets/            # Icons, fonts, static images
    ├── theme/             # Design System
    │   ├── material_theme.slint  # Material 3 Color Tokens & Typography
    │   └── styles.slint   # Shared styles/shapes
    ├── components/        # Reusable UI Components
    │   ├── atoms/         # Low-level widgets
    │   │   ├── button.slint
    │   │   ├── icon_button.slint
    │   │   └── text_field.slint
    │   ├── molecules/     # Complex widgets
    │   │   ├── dialog.slint
    │   │   └── list_item.slint
    │   └── organisms/     # Major UI sections
    │       ├── top_bar.slint     # Replaces MenuBar
    │       ├── navigation_rail.slint # Replaces Sidebar (Desktop-friendly)
    │       └── canvas.slint      # The image annotation area
    └── views/             # Full screen views (if we add more modes)
```

### **Key Improvements**
-   **`model/` vs `ui/`**: separation prevents `main.rs` from becoming a monolith.
-   **`components/`**: organized by Atomic Design principles (atoms, molecules, organisms) to encourage reuse.
-   **`ui_bridge/`**: Explicit layer for Rust-Slint communication, keeping `main.rs` clean.

---

## 2. Material 3 UI Redesign

We will implement a custom Material 3 design system within Slint. Since Slint doesn't have a built-in Material library, we will define the necessary tokens and components.

### **Theme System (`ui/theme/material_theme.slint`)**
We will replace the current semantic colors (`bg-dark`, `accent-primary`) with Material 3 Role-based tokens:

-   **Primary / OnPrimary / PrimaryContainer / OnPrimaryContainer**: For key actions (drawing tools).
-   **Secondary / ...**: For less prominent actions.
-   **Surface / OnSurface**: Backgrounds and text.
-   **Outline**: Borders.
-   **Error**: Destructive actions (delete).

### **Component Redesign Strategy**

1.  **Top App Bar (Replaces `MenuBar`)**
    -   **Visual:** Flat surface color, small shadow on scroll (or always), distinct title.
    -   **Structure:**
        -   *Left:* Menu icon (hamburger) or Logo.
        -   *Center:* Title / Breadcrumbs.
        -   *Right:* Actions (Save, Export, Settings).
    -   **Improvement:** cleaner look than the traditional OS-style menu bar, easier to hit targets.

2.  **Navigation Rail (Replaces `Sidebar`)**
    -   **Why:** The current sidebar acts like a tool palette mixed with a class list. A Navigation Rail is standard for desktop apps.
    -   **Structure:**
        -   *Top:* Tool switching (FAB or Segmented Button).
        -   *Middle:* Vertical list of icon-based destinations or specific class shortcuts.
        -   *Bottom:* Help / Settings.
    -   **Panel:** Clicking a rail item can open a detailed drawer (e.g., the full Class List).

3.  **Canvas Area**
    -   **Floating Controls:** Instead of a fixed status bar, use floating pills (Chips) for status info (Zoom level, current image name).
    -   **FAB (Floating Action Button):** A prominent "Next Image" or "Add Annotation" button if appropriate.

4.  **Dialogs**
    -   Implement standard Material 3 Dialogs with rounded corners (`28px`), scrim, and distinct action buttons (TextButton for "Cancel", FilledButton for "Confirm").

### **UX Improvements**

-   **First-Timers:**
    -   **Empty States:** When no dataset is loaded, show a large, centered "Open Dataset" card with an illustration/icon.
    -   **Tooltips:** Add hover tooltips to all icon-only buttons.
    -   **Onboarding Overlay:** A dismissible overlay explaining `B` (Box), `C` (Point), `S` (Polygon) shortcuts.

-   **Super-Users:**
    -   **Visual Feedback:** clearer selection states (thick outlines, corner handles with high contrast).
    -   **Keyboard Visualizer:** A small persistent indicator showing currently held keys (e.g., `Hold S for Polygon`).

---

## 3. Execution Steps

1.  **Move & Split Rust Code:**
    -   Create new folders in `src/`.
    -   Extract structs to `src/model/`.
    -   Extract logic to `src/service/`.
    -   Update `main.rs` to use these modules.

2.  **Setup Slint Structure:**
    -   Create `ui/theme/material_theme.slint`.
    -   Create `ui/components/atoms/`.

3.  **Refactor UI Components (Iterative):**
    -   *Phase 1:* Replace `MenuBar` with `TopBar`.
    -   *Phase 2:* Replace `Sidebar` with `NavigationRail` + `Drawer`.
    -   *Phase 3:* Update `Canvas` overlays and rendering to use new Theme colors.
    -   *Phase 4:* Redesign `SettingsDialog`.

4.  **Bind & Test:**
    -   Ensure all Rust callbacks still fire correctly with the new UI components.
