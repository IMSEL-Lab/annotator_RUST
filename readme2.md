# RADICAL Image Annotator - Architecture Documentation

This document explains how the Rust and Slint code work together in this image annotation tool, with architecture diagrams and improvement recommendations.

## High-Level Architecture

```mermaid
flowchart TB
    subgraph Build["🔨 Build Time"]
        BuildRS[build.rs]
        SlintFiles[ui/*.slint]
        BuildRS -->|"slint_build::compile"| SlintFiles
        SlintFiles -->|generates| GeneratedCode["Generated Rust Code<br/>(include_modules!())"]
    end

    subgraph Runtime["⚡ Runtime"]
        subgraph RustBackend["Rust Backend"]
            Main[main.rs]
            Config[config.rs]
            Classes[classes.rs]
            Hierarchy[hierarchy.rs]
            AutoResize[auto_resize.rs]
            Export[export/mod.rs]
        end

        subgraph SlintUI["Slint UI Layer"]
            AppWindow[AppWindow Component]
            TopBar[TopBar]
            SidePanel[SidePanel]
            Canvas[Canvas + TouchArea]
            BottomBar[BottomBar]
        end

        subgraph State["Shared State (Rc<RefCell<T>>)"]
            DatasetState[DatasetState]
            Annotations[VecModel&lt;Annotation&gt;]
            DrawState[DrawState]
            UndoHistory[UndoHistory]
            HierarchyNav[HierarchyNavigator]
        end
    end

    GeneratedCode --> Main
    Main -->|"AppWindow::new()"| AppWindow
    Main -->|manages| State
    AppWindow <-->|"callbacks & properties"| Main
```

## Data Flow Architecture

```mermaid
flowchart LR
    subgraph Input["User Input"]
        Mouse["🖱️ Mouse Events"]
        Keyboard["⌨️ Keyboard Events"]
        Menu["📁 Menu Actions"]
    end

    subgraph SlintLayer["Slint UI (appwindow.slint)"]
        TouchArea["TouchArea<br/>pointer-event / moved"]
        FocusScope["FocusScope<br/>key-pressed/released"]
        Callbacks["Callbacks<br/>(40+ defined)"]
        Properties["Properties<br/>(in/out bindings)"]
    end

    subgraph RustLayer["Rust Backend (main.rs)"]
        EventHandler["Event Handlers<br/>on_* callbacks"]
        StateUpdate["State Updates<br/>Rc&lt;RefCell&lt;T&gt;&gt;"]
        BusinessLogic["Business Logic<br/>(save, load, resize)"]
    end

    subgraph Output["UI Updates"]
        ModelUpdate["VecModel Updates"]
        PropertySet["set_* calls"]
        Rerender["Slint Re-render"]
    end

    Mouse --> TouchArea --> Callbacks
    Keyboard --> FocusScope --> Callbacks
    Menu --> Callbacks
    Callbacks -->|"invoke callback"| EventHandler
    EventHandler --> StateUpdate
    StateUpdate --> BusinessLogic
    BusinessLogic --> ModelUpdate & PropertySet
    ModelUpdate & PropertySet --> Rerender
```

## Module Responsibilities

```mermaid
graph TB
    subgraph core["Core Modules"]
        main["main.rs<br/>━━━━━━━━━━━━━<br/>• UI initialization<br/>• Callback binding<br/>• State management<br/>• Navigation logic<br/>• Drawing handlers"]
        
        config["config.rs<br/>━━━━━━━━━━━━━<br/>• AppConfig structs<br/>• TOML persistence<br/>• Default values"]
        
        classes["classes.rs<br/>━━━━━━━━━━━━━<br/>• ClassDefinition<br/>• YAML parsing<br/>• Color handling"]
    end
    
    subgraph support["Support Modules"]
        hierarchy["hierarchy.rs<br/>━━━━━━━━━━━━━<br/>• Tree navigation<br/>• Breadcrumb state<br/>• Depth validation"]
        
        auto_resize["auto_resize.rs<br/>━━━━━━━━━━━━━<br/>• Sobel edge detect<br/>• Smart bbox fit<br/>• Image processing"]
        
        export["export/<br/>━━━━━━━━━━━━━<br/>• COCO format<br/>• VOC XML format<br/>• YOLO (inline)"]
    end
    
    main --> config & classes
    main --> hierarchy & auto_resize & export
```

## Slint-Rust Callback Pattern

```mermaid
sequenceDiagram
    participant UI as Slint UI
    participant CB as Callback Bridge
    participant State as Shared State
    participant Logic as Business Logic

    Note over UI,Logic: Example: Creating a Bounding Box
    
    UI->>CB: start-drawing(x, y)
    CB->>State: draw_state.borrow_mut()
    State-->>CB: &mut DrawState
    CB->>State: Update start_x, start_y
    
    UI->>CB: update-drawing(x, y)
    CB->>State: Calculate preview rect
    CB->>UI: set_preview_* properties
    
    UI->>CB: finish-drawing(x, y)
    CB->>State: undo_history.push(snapshot)
    CB->>State: annotations.push(new_ann)
    CB->>Logic: save_current_state()
    CB->>UI: set_show_preview(false)
```

## State Management Pattern

```mermaid
graph LR
    subgraph ownership["Ownership Pattern"]
        RC["Rc&lt;T&gt;<br/>Reference Counted"]
        RefCell["RefCell&lt;T&gt;<br/>Interior Mutability"]
        Clone["clone() for callbacks"]
    end
    
    subgraph usage["Common Usage"]
        code1["let state = Rc::new(RefCell::new(...));"]
        code2["let state_ref = state.clone();"]
        code3["ui.on_callback(move || {<br/>  state_ref.borrow_mut().<br/>})"]
    end
    
    RC --> RefCell --> Clone
    code1 --> code2 --> code3
```

## UI Component Hierarchy

```mermaid
graph TB
    subgraph AppWindow["AppWindow (760 lines)"]
        subgraph Layout["VerticalLayout"]
            TopBar2["TopBar<br/>• File/View/Tools menus<br/>• Navigation buttons"]
            
            subgraph HLayout["HorizontalLayout"]
                SidePanel2["SidePanel<br/>• Tool selection<br/>• Class selection<br/>• Hierarchy nav"]
                
                subgraph Canvas2["image-container"]
                    Image["Image + pan/zoom"]
                    TouchArea2["TouchArea<br/>• Click/drag handling<br/>• Scroll zoom"]
                    Preview["Drawing Preview"]
                    Annotations2["for annotation in annotations"]
                end
            end
            
            BottomBar2["BottomBar<br/>• Status info<br/>• Position display"]
        end
        
        Dialogs["Dialogs<br/>• AppearanceDialog<br/>• LayoutDialog<br/>• KeybindingsDialog"]
        
        FocusScope2["FocusScope<br/>• All keyboard shortcuts"]
    end
```

---

## Improvement Proposals

### 1. **Split main.rs into Multiple Modules** ⭐ High Priority

**Problem**: `main.rs` is 2,266 lines with 40+ callback handlers, making it hard to navigate and maintain.

**Proposed Structure**:
```
src/
├── main.rs              # Entry point, initialization (~100 lines)
├── app.rs               # AppWindow setup and state wiring (~200 lines)
├── callbacks/
│   ├── mod.rs           # Re-exports
│   ├── navigation.rs    # next/prev/first/last image handlers
│   ├── drawing.rs       # start/update/finish drawing
│   ├── annotation.rs    # select, delete, classify handlers  
│   ├── resize.rs        # start/update/finish resize
│   └── file.rs          # open, save, export handlers
├── state/
│   ├── mod.rs
│   ├── dataset.rs       # DatasetState, DatasetEntry
│   ├── undo.rs          # UndoHistory
│   └── view.rs          # ViewState, DrawState, ResizeState
└── (existing modules...)
```

### 2. **Extract Annotation Types to Dedicated Module** ⭐ Medium Priority

**Problem**: Annotation handling logic (parsing, saving, rendering) is scattered throughout `main.rs`.

**Proposed Solution**:
```rust
// src/annotation/mod.rs
pub struct AnnotationManager {
    model: Rc<VecModel<Annotation>>,
    undo: Rc<RefCell<UndoHistory>>,
}

impl AnnotationManager {
    pub fn add(&self, ann: Annotation) { ... }
    pub fn delete(&self, index: usize) { ... }
    pub fn select(&self, index: usize) { ... }
    pub fn snapshot(&self) -> Vec<Annotation> { ... }
    pub fn restore(&self, snapshot: Vec<Annotation>) { ... }
}
```

### 3. **Use Slint Globals for Shared State** ⭐ Medium Priority

**Problem**: Currently, state like `current-tool` and `current-class` is managed via properties passed between components.

**Proposed Solution**:
```slint
// ui/globals.slint
export global AppState {
    in-out property <string> current-tool: "Neutral";
    in-out property <int> current-class: 1;
    in-out property <bool> is-drawing: false;
    in-out property <float> zoom-level: 1.0;
}
```

Benefits:
- Any component can access state directly
- Reduces property drilling through component hierarchy
- Cleaner Rust code (single point of state access)

### 4. **Implement Command Pattern for Undo/Redo** 🔄 Nice to Have

**Problem**: Current undo stores full annotation snapshots, which is memory-inefficient.

**Proposed Solution**:
```rust
enum Command {
    AddAnnotation { annotation: Annotation },
    DeleteAnnotation { index: usize, annotation: Annotation },
    ModifyAnnotation { index: usize, old: Annotation, new: Annotation },
    BatchCommand { commands: Vec<Command> },
}

impl Command {
    fn execute(&self, model: &VecModel<Annotation>) { ... }
    fn undo(&self, model: &VecModel<Annotation>) { ... }
}
```

### 5. **Add Error Handling with thiserror** 🔄 Nice to Have

**Problem**: Error handling uses `Result<T, String>` throughout, losing type information.

**Proposed Solution**:
```rust
// src/error.rs
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Failed to load dataset: {0}")]
    DatasetLoad(String),
    
    #[error("Image not found: {path}")]
    ImageNotFound { path: PathBuf },
    
    #[error("Export failed: {0}")]
    Export(#[from] std::io::Error),
}
```

### 6. **Lazy Image Loading for Large Datasets** ⭐ High Priority

**Problem**: All images are loaded synchronously, causing delays with large datasets.

**Proposed Solution**:
```rust
// Async image preloading
struct ImageCache {
    loaded: HashMap<usize, slint::Image>,
    loading: HashSet<usize>,
}

// Preload adjacent images in background
fn preload_adjacent(&self, current: usize, range: usize) {
    for i in current.saturating_sub(range)..=current + range {
        if !self.loaded.contains_key(&i) {
            // Spawn async task to load
        }
    }
}
```

### 7. **Type-Safe Tool Selection** 🔄 Nice to Have

**Problem**: Tools are identified by magic strings like `"BBox (B)"`, `"Point (C)"`.

**Proposed Solution**:
```rust
#[derive(Clone, PartialEq)]
enum Tool {
    Neutral,
    BBox,
    Point,
    Polygon,
    RotatedBBox,
}

impl Tool {
    fn to_display_string(&self) -> &'static str {
        match self {
            Tool::BBox => "BBox (B)",
            // ...
        }
    }
}
```

---

## Priority Summary

| Priority | Improvement | Impact | Effort |
|----------|-------------|--------|--------|
| ⭐ High | Split main.rs | Maintainability | Medium |
| ⭐ High | Lazy image loading | Performance | Medium |
| ⭐ Medium | Extract AnnotationManager | Code organization | Low |
| ⭐ Medium | Slint Globals for state | Clean architecture | Low |
| 🔄 Nice | Command pattern undo | Memory efficiency | High |
| 🔄 Nice | thiserror for errors | Developer experience | Low |
| 🔄 Nice | Type-safe Tool enum | Type safety | Low |

---

## Quick Reference: Key Integration Points

| Slint → Rust | Rust → Slint |
|--------------|--------------|
| `callback name(args)` | `ui.on_name(move \|args\| { ... })` |
| In Slint: `root.name()` | `ui.invoke_name()` |

| Rust → Slint | |
|--------------|--------------|
| Set property | `ui.set_property_name(value)` |
| Get property | `ui.get_property_name()` |
| Update model | `model.push(item)`, `model.set_row_data(i, item)` |

