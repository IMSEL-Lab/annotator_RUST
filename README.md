# Annotator

A high-performance, native, keyboard-driven image annotation tool built in Rust with [Slint UI](https://slint.dev).

Designed for rapid, high-throughput labeling of datasets, particularly optimized for **Radar PPI (Plan Position Indicator)** frames and other high-contrast imagery where speed and edge-precision are paramount.

## Why use Annotator over Label Studio or CVAT?

While tools like Label Studio and CVAT are powerful, enterprise-grade solutions, they often come with significant overhead. **Annotator** focuses on a different set of priorities:

1.  **Zero Setup / Zero Infrastructure**:
    *   **Annotator**: A single native binary. Run it on a folder of images, and it works instantly. No databases, no Docker containers, no web servers, no login accounts.
    *   **Them**: Requires setting up PostgreSQL, Redis, Docker containers, or managing a complex web deployment.

2.  **Native Performance**:
    *   **Annotator**: Written in Rust. It loads images instantly, handles thousands of files without browser lag, and uses minimal RAM.
    *   **Them**: Web-based (Electron or Browser). Can feel sluggish with large images or rapid switching, dependent on browser memory limits.

3.  **"Hold-to-Act" Workflow**:
    *   **Annotator**: Uses a gaming-inspired "hold key to activate tool" workflow (e.g., hold `B` to box, release to stop). This reduces clicks and mode-switching fatigue, allowing for extremely fast "flow state" labeling.
    *   **Them**: Traditional "click tool icon -> draw -> click save -> click next" workflow.

4.  **Smart Edge Detection (The "Radar" Advantage)**:
    *   **Annotator**: Includes a **Smart Auto-Resize** feature (Hotkey: `A` + Click) that uses Sobel edge detection to automatically snap bounding boxes to the edges of high-contrast blobs.
    *   **Use Case**: Perfect for radar returns, thermal hotspots, or binary masks where manual pixel-perfect boxing is tedious and slow.

5.  **Data Privacy & Locality**:
    *   **Annotator**: Operates directly on your local file system. Labels are saved as `.txt` (YOLO format) or JSON side-by-side with your images. You own your data completely; nothing leaves your machine.

6.  **Hierarchical Class Selection**:
    *   **Annotator**: Solves the "too many classes" problem with a hierarchical keyboard system. Encode over 100 classes accessible entirely with the left hand, without ever leaving the mouse.
    *   **Workflow**: Uses a rapid 3-level selection (5 keys for high-level category -> 5 keys for sub-class -> 5 keys for leaf class). This enables muscle-memory access to complex taxonomies.
    *   **Them**: Often requires selecting a class *then* drawing, or scrolling through massive dropdowns/searching text when the number of classes grows large.

## Key Features

*   **Multi-Modal Annotation**: Support for **Bounding Boxes**, **Points**, and **Polygons** (Segmentation).
*   **Smart Auto-Resize**: `A` + Click on a target to automatically fit the bounding box to the object's edges.
*   **Rapid Navigation**: Preloads images for instant switching. Mark frames as "Complete" (`F`) to track progress.
*   **Standard Exports**:
    *   Native **YOLO** (`.txt` files next to images).
    *   Export to **COCO** JSON.
    *   Export to **Pascal VOC** XML.
*   **Keyboard Centric**: Every action has a hotkey. Designed to be used with one hand on the keyboard and one on the mouse.
*   **Adaptive UI**: Native support for both **Dark** and **Light** modes.
