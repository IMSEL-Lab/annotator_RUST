# Icon Reference

This document describes all the SVG icons used in the annotator application.

## Drawing Tool Icons

### Bounding Box
**File**: `ui/material/ui/icons/bounding_box.svg`
**Usage**: Main annotation tool for rectangular selections
**Icon**: A square outline (hollow rectangle)
```
┌─────────┐
│         │
│         │
└─────────┘
```

### Center Point
**File**: `ui/material/ui/icons/center_point.svg`
**Usage**: Point annotation tool for marking specific locations
**Icon**: A filled circle
```
    ●
```

### Polygon/Segmentation
**File**: `ui/material/ui/icons/polygon.svg`
**Usage**: Advanced segmentation tool for irregular shapes
**Icon**: An outlined hexagon/polygon shape
```
   ╱─╲
  ╱   ╲
 │     │
  ╲   ╱
   ╲─╱
```

---

## Navigation Icons

### Skip Previous (First)
**File**: `ui/material/ui/icons/skip_previous.svg`
**Usage**: Jump to first image in dataset
**Icon**: Bar on left, triangle pointing left
```
▐◀
```

### Chevron Backward (Previous)
**File**: `ui/material/ui/icons/chevron_backward.svg`
**Usage**: Go to previous image
**Icon**: Left-pointing chevron
```
◀
```

### Shuffle (Random)
**File**: `ui/material/ui/icons/shuffle.svg`
**Usage**: Jump to random image
**Icon**: Crossed arrows
```
🔀
```

### Chevron Forward (Next)
**File**: `ui/material/ui/icons/chevron_forward.svg`
**Usage**: Go to next image
**Icon**: Right-pointing chevron
```
▶
```

### Skip Next (Last)
**File**: `ui/material/ui/icons/skip_next.svg`
**Usage**: Jump to last image in dataset
**Icon**: Triangle pointing right, bar on right
```
▶▌
```

---

## Other Icons

- **check.svg** - Completion indicator
- **close.svg** - Close/cancel actions
- **edit.svg** - Edit mode
- **menu.svg** - Menu toggle
- **calendar_today.svg** - Date picker
- **schedule.svg** - Time picker
- **keyboard.svg** - Keyboard shortcuts
- **arrow_back.svg** - Back navigation
- **arrow_right.svg** - Forward navigation
- **arrow_drop_down.svg** - Dropdown menu (down)
- **arrow_drop_up.svg** - Dropdown menu (up)
- **remove.svg** - Remove/delete

---

## Icon Properties in Slint

All icons are accessed through the global `Icons` object:

```slint
import { Icons } from "../material/ui/icons/icons.slint";

Icon {
    source: Icons.bounding_box;
    colorize: MaterialPalette.on-primary-container;
    width: 28px;
    height: 28px;
}
```

### Available Icon Properties

```slint
Icons.bounding_box      // Bounding box tool
Icons.center_point      // Center point tool
Icons.polygon           // Segmentation/polygon tool
Icons.skip_previous     // First image
Icons.skip_next         // Last image
Icons.chevron_backward  // Previous image
Icons.chevron_forward   // Next image
Icons.shuffle           // Random image
Icons.check             // Checkmark
Icons.close             // X close
Icons.edit              // Pencil edit
Icons.menu              // Hamburger menu
// ... and more
```

---

## SVG Format Requirements

All icons must follow this format for consistency:

```svg
<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24">
  <path d="...path data..."/>
</svg>
```

**Required attributes**:
- `xmlns="http://www.w3.org/2000/svg"` - XML namespace
- `width="24"` - Fixed width
- `height="24"` - Fixed height
- `viewBox="0 0 24 24"` - Coordinate system

**Path element**:
- Use `<path>` for complex shapes
- Use `<circle>` for circles
- Use `<rect>` for rectangles
- Use `<polygon>` for polygons

**No fill attribute**: The `colorize` property in Slint handles coloring

---

## Creating New Icons

1. Create SVG file in `ui/material/ui/icons/`
2. Follow the format above (24x24, no fill)
3. Add to `ui/material/ui/icons/icons.slint`:
   ```slint
   out property <image> my_icon: @image-url("my_icon.svg");
   ```
4. Use in components:
   ```slint
   Icon {
       source: Icons.my_icon;
       colorize: MaterialPalette.on-surface;
   }
   ```

---

## Icon Guidelines

### Design
- **Simple shapes**: Icons should be recognizable at small sizes (20-32px)
- **2px stroke**: Use 2px stroke width for consistency
- **Centered**: Keep icons centered in the 24x24 viewBox
- **Clear silhouettes**: Avoid fine details

### Color
- **Never hardcode colors** in SVG (no `fill` attribute)
- Use `colorize` property to apply Material palette colors
- Icons automatically adapt to light/dark themes

### Size
- **Default**: 24px (icon source size)
- **Small buttons**: 20px
- **Tool buttons**: 28px
- **Large actions**: 32px

---

## Accessibility

All icons should have:
- High contrast with their background
- Clear, recognizable shapes
- Tooltips or labels for screen readers
- Minimum 3:1 contrast ratio (WCAG AA)

Material 3 automatically ensures proper contrast when using palette colors with the `colorize` property.
