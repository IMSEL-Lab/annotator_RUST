# RADICAL Image Annotator

Rust/Slint desktop tool for fast, keyboard-first image labeling in air‑gapped or otherwise locked-down environments. Everything runs locally: no browser, no server, no database, no telemetry.

## Highlights
- Modern Slint GUI with dark/light theme and zero web stack — inspired by our FrameTrails sibling app.
- Native binary; immediate startup; works offline on macOS/Linux/Windows.
- Bounding boxes, points, and polygon segmentation with 50-level undo/redo and multi-item copy/paste.
- Hierarchical class picker (up to 3 levels, 5 options per level) driven entirely by the `1–5` keys.
- Smart auto-resize for boxes: hold `A` and click inside a box to snap edges to image gradients (Sobel-based).
- Autosave every 5 s to YOLO text + compact `*.state.json` sidecars; manual save/export buttons stay available.
- COCO JSON and Pascal VOC XML export directly from the UI.
- Real-time status: filename, position, completion flag, tool/class, and inline debug log writer.
- Persistent settings and layout (like FrameTrails): theme, sidebar width/side, enabled tools, dataset randomization.
- Optional sidebar/hierarchy debug controls available from stdin (`width <px>`, `hide`, `show`).

## Quick Start
1. Install Rust (edition 2024 compatible; tested with stable toolchains).  
2. Build: `cargo build --release`
3. Run with an existing dataset manifest:  
   `target/release/annotator /path/to/manifest.json`
   - Or start without args and use **File → Open Dataset** or **File → New Dataset**.
4. Toggle theme/layout/keybinding dialogs from the **Tools** menu; settings persist to `~/.config/annotator/config.toml`.

## Dataset Format
- A dataset is driven by `manifest.json`:
  ```json
  {
    "images": [
      { "image": "images/frame_0001.jpg", "labels": "labels/frame_0001.txt" },
      { "image": "images/frame_0002.jpg", "labels": "labels/frame_0002.txt" }
    ]
  }
  ```
- Images paths are resolved relative to the manifest. `labels` is optional; when omitted the app expects `<image>.txt`.
- Label files follow YOLO (v5/8) normalized bbox lines: `class cx cy w h` (class is 0-based on disk; the app shows 1-based in UI).
- The app also writes `<labels>.state.json` per image to keep polygon vertices, selection flags, and view state; those files are internal but portable.
- **Create a manifest automatically:** `File → New Dataset` scans a folder for image files and writes `manifest.json` plus empty label stubs.

## Controls (default build)
- **BBox:** hold `B`, drag LMB, release to finish (releases back to Neutral).
- **Point:** hold `C`, click.
- **Polygon:** hold `S`, click vertices, release `S` or press `Tab`/`Enter` to finish; `Esc` cancels.
- **Pan/Zoom:** Neutral mode drag; mouse wheel zooms at cursor; `H` or `Ctrl+0` fits view.
- **Classify:** digits `1–5` set class for selection; hold digit + click to reclassify under cursor. Hierarchy mode routes `1–5` through tree levels.
- **Delete:** `Q` + click or double-click; `Del/Backspace` deletes selected.
- **Auto-resize:** hold `A` + click inside a bbox to edge-snap it.
- **Undo/Redo:** `Ctrl+Z` / `Ctrl+Shift+Z` (or `Ctrl+Y`).
- **Copy/Paste selection:** `Ctrl+C` / `Ctrl+V` (pastes with slight offset).
- **Navigation:** Space/Right for next, Shift+Space/Left for previous, `F` toggles frame complete, menus offer first/last/random.

See `KEYBINDINGS.txt` for the exact list used by this build.

## Configuration
- Stored at `~/.config/annotator/config.toml` (created on first run). Controls theme, sidebar size/visibility, enabled annotation types, dataset randomization, autosave interval, and default export format.
- Classes come from (in priority order): an explicit path, `./classes.yaml`, `./coco_hierarchy.yaml`, then `~/.config/annotator/classes.yaml`. Provide either a flat `classes:` list or a hierarchical tree with `key` 1–5 per node.

## Exporting
- **COCO JSON:** **File → Export → COCO** writes `annotations.json` with images, categories (from `classes.yaml`), and segmentation/polygon data.
- **Pascal VOC:** **File → Export → VOC** writes one XML per image with bboxes and class names.

## Working in Secure / Air‑Gapped Environments
- Single native binary; no Docker, PostgreSQL, Redis, Node, or browser.
- All data stays on disk you choose (manifest/labels/state); no outbound requests or analytics.
- Minimal attack surface: local UI only, no HTTP server or open ports.
- Deterministic, human-readable file formats (YOLO, JSON, XML) suited for change control and offline review.
- Autosave keeps work safe even without background services.
- Cross-platform Slint UI and native file dialogs (same hardening story as FrameTrails) — deploy the one binary your enclave allows.

## Why This Beats CVAT or Label Studio for Secure Sites
- **Zero services to harden:** CVAT/Label Studio require a running web stack (Django/FastAPI, DB, Redis), open ports, and user auth; this tool is a self-contained GUI executable.
- **Air-gap friendly:** Runs fully offline; no CDN assets or container pulls. Perfect for enclaves where CVAT/LS installs are blocked by registry egress.
- **File-first workflow:** Operates directly on folders and YOLO labels—no project imports/exports, DB migrations, or task queues.
- **Keyboard-native speed:** Hierarchical classing with `1–5`, hold-to-draw tools, instant undo/redo, and gradient-based auto-resize minimize mouse travel; web UIs typically have higher latency and heavier hitboxes.
- **Tiny footprint:** Rust + Slint build; starts in milliseconds and consumes far less RAM/CPU than browser + server stacks, making it viable on stripped-down bastion hosts.
- **Predictable persistence:** Autosaves to sidecar files you can version; no opaque database to back up or migrate.

## Development Notes
- UI source: `ui/appwindow.slint` (themes in `ui/app_theme.slint`, components under `ui/components/`).
- Rust entrypoint: `src/main.rs`; UI callbacks live in `src/callbacks/`.
- Auto-resize logic: `src/auto_resize.rs` (Sobel + Gaussian blur search within ±30% of the box).
- Dataset and export code: `src/state/` and `src/export/`.
- Build script (`build.rs`) wires the Slint compiler with the bundled Material theme.

## Troubleshooting
- Blank window or no dataset: ensure you pass a `manifest.json` path or open a dataset from the File menu.
- Boxes not visible: check that `classes.yaml` exists and IDs start at 1; YOLO files on disk are 0-based by design.
- Side panel too wide/narrow: type `width 240`/`width 300` into the terminal where the app was launched; `hide`/`show` toggles visibility for quick debugging.
