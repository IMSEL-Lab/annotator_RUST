# Visual Test Reference Guide
## Image Dimensions: 2560 x 1440 pixels

This guide shows where annotations should appear as percentages of your image dimensions.

---

## P3-010: Select Single BBox

**Single red rectangle:**
```
Position: (300, 200) = 11.7% from left, 13.9% from top
Size: 300x200 = 11.7% wide, 13.9% tall
Center: (450, 300) = 17.6% from left, 20.8% from top
```

**Visual location:**
```
┌─────────────────────────────────────────┐
│                                         │
│    ┌──────┐  ← Box should be here      │
│    │      │     (upper-left area)      │
│    │      │                             │
│    └──────┘                             │
│                                         │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

---

## P3-011: Select Center Point

**Three red dots:**
```
Point 1: (300, 200) = 11.7% from left, 13.9% from top
Point 2: (600, 200) = 23.4% from left, 13.9% from top
Point 3: (450, 400) = 17.6% from left, 27.8% from top
```

**Visual location:**
```
┌─────────────────────────────────────────┐
│                                         │
│    •           •      ← Two dots        │
│                          (top row)      │
│                                         │
│         •              ← One dot        │
│                          (middle)       │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

---

## P3-022: Zoom Alignment ⚠️ CRITICAL TEST

**Regular Box (ID 1):**
```
Position: (200, 150) = 7.8% from left, 10.4% from top
Size: 300x200 = 11.7% wide, 13.9% tall
Spans to: (500, 350) = 19.5% from left, 24.3% from top
```

**Rotated Box (ID 2) - THE CRITICAL ONE:**
```
Position: (550, 350) = 21.5% from left, 24.3% from top
Size: 200x150 = 7.8% wide, 10.4% tall
Rotation: 30 degrees
Center: (650, 425) = 25.4% from left, 29.5% from top
```

**Point (ID 3):**
```
Position: (400, 250) = 15.6% from left, 17.4% from top
```

**Visual location:**
```
┌─────────────────────────────────────────┐
│                                         │
│  ┌────┐                                │ ← Regular box (upper-left)
│  │  • │                                │    with point inside
│  │    │                                │
│  └────┘    ╱────╲                      │ ← Rotated box (middle)
│           ╱      ╲                     │    tilted 30°
│          ╲        ╱                    │
│           ╲────╱                       │
│                                         │
└─────────────────────────────────────────┘
```

---

## P3-013: Overlapping Selection

**Box 1 (lower):**
```
Position: (200, 200) = 7.8% from left, 13.9% from top
Size: 300x200 = 11.7% wide, 13.9% tall
```

**Box 2 (upper) - offset by (50, 50):**
```
Position: (250, 250) = 9.8% from left, 17.4% from top
Size: 300x200 = 11.7% wide, 13.9% tall
```

**Visual location:**
```
┌─────────────────────────────────────────┐
│                                         │
│  ┌─────────┐                           │
│  │         │                            │
│  │   ┌─────────┐  ← Boxes overlap      │
│  └───┼─────│   │     in this region    │
│      │     │   │                        │
│      └─────────┘                        │
│                                         │
└─────────────────────────────────────────┘
```

**Overlap Region:**
- Top-left: (250, 250) = 9.8%, 17.4%
- Bottom-right: (500, 400) = 19.5%, 27.8%

---

## P3-020: Initial Alignment

**Four annotations to verify:**
```
Box 1:      (100, 100) = 3.9%, 6.9%     ← Upper-left corner
Rotated 2:  (400, 300) = 15.6%, 20.8%   ← Center-left, tilted 30°
Point 3:    (600, 200) = 23.4%, 13.9%   ← Upper-center
Box 4:      (800, 400) = 31.3%, 27.8%   ← Center-right
```

**Visual location:**
```
┌─────────────────────────────────────────┐
│                                         │
│ ┌──┐          •               ┌──┐    │
│ └──┘                           └──┘    │
│                                         │
│         ╱────╲                          │
│        ╱      ╲                         │
│       ╲        ╱                        │
│        ╲────╱                           │
│                                         │
└─────────────────────────────────────────┘
```

---

## How to Verify Alignment

### At Initial View (Fit-to-View):
1. All annotations should be visible
2. Use the percentages to mentally divide your screen
3. Annotations should be in the approximate screen regions shown

### When Zoomed In:
1. Pick a feature in your image (like a corner or distinctive object)
2. Note an annotation's position relative to that feature
3. Zoom in/out - the annotation should stay locked to that feature
4. **For rotated box**: It should maintain its 30° angle at ALL zoom levels

### When Panning:
1. Note an annotation's position relative to image features
2. Pan in any direction
3. The annotation should move exactly with the image
4. No lag, drift, or offset should appear

---

## Testing Checklist

For **P3-022** (Critical Zoom Test):

- [ ] At fit-to-view, all 3 annotations visible at expected positions
- [ ] Regular box in upper-left area (~8-20% from edges)
- [ ] Rotated box in center area (~21-25% from left, ~24-30% from top)
- [ ] Rotated box appears tilted approximately 30° (11 o'clock position)
- [ ] Point visible inside or near regular box
- [ ] Zoom IN: All annotations get larger proportionally
- [ ] Zoom IN: Rotated box STAYS at 30° angle (doesn't rotate more/less)
- [ ] Zoom OUT: All annotations get smaller proportionally
- [ ] Zoom OUT: Rotated box STAYS at 30° angle
- [ ] At all zoom levels: Annotations locked to same image features
- [ ] No parallax, drift, or offset at any zoom level

---

## Quick Position Calculator

If you want to check any annotation position:

```
X percentage = (X coordinate / 2560) × 100%
Y percentage = (Y coordinate / 1440) × 100%
```

Example: Annotation at (400, 300)
- X: (400 / 2560) × 100 = 15.6% from left
- Y: (300 / 1440) × 100 = 20.8% from top

---

## What "Correct Alignment" Looks Like

**✅ CORRECT:**
- Annotation appears at same percentage of image at all zoom levels
- Rotated box angle stays constant (30°) at all zoom levels
- When you zoom into a corner, annotation in that corner gets larger
- When you pan, annotations move perfectly with image

**❌ INCORRECT:**
- Annotation position shifts when zooming
- Rotated box angle changes when zooming
- Annotations lag behind when panning
- Annotations appear offset from expected positions
- Parallax effect (annotations move at different rate than image)
