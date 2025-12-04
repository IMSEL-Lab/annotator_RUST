# Material 3 Color System Reference

This file documents all available colors in the Material 3 design system for both light and dark modes.

## Color Role Categories

Material 3 provides semantic color roles that automatically adapt between light and dark themes.

### 1. PRIMARY COLORS
The primary color is used for key components and high-emphasis actions.

- `MaterialPalette.primary` - Main brand color
- `MaterialPalette.on-primary` - Text/icons on primary
- `MaterialPalette.primary-container` - Standout fill color for key components
- `MaterialPalette.on-primary-container` - Text/icons on primary-container

**Typical Colors:**
- Light mode: Deep blue/teal (#6750A4 purple or #006A6A teal)
- Dark mode: Lighter blue/teal (#D0BCFF purple or #4DD0E1 teal)

**Best for:** Main actions, selected states, primary buttons

---

### 2. SECONDARY COLORS
The secondary color provides more ways to accent and distinguish your product.

- `MaterialPalette.secondary` - Less prominent than primary
- `MaterialPalette.on-secondary` - Text/icons on secondary
- `MaterialPalette.secondary-container` - Tonal fill for secondary components
- `MaterialPalette.on-secondary-container` - Text/icons on secondary-container

**Typical Colors:**
- Light mode: Teal/cyan (#625B71 purple-gray or #4A6363 dark teal)
- Dark mode: Light teal/cyan (#CCC2DC light purple or #B1CCCC light teal)

**Best for:** Secondary actions, chips, badges, filters

---

### 3. TERTIARY COLORS
The tertiary color provides additional accent colors for visual variety.

- `MaterialPalette.tertiary` - Accent and emphasis
- `MaterialPalette.on-tertiary` - Text/icons on tertiary
- `MaterialPalette.tertiary-container` - Containers for tertiary components
- `MaterialPalette.on-tertiary-container` - Text/icons on tertiary-container

**Typical Colors:**
- Light mode: Pink/orange (#7D5260 mauve or #9C4146 brick red)
- Dark mode: Light pink/orange (#EFB8C8 light pink or #FFB4AB light coral)

**Best for:** Highlights, special states, complementary actions

---

### 4. ERROR COLORS
Error colors communicate critical states and destructive actions.

- `MaterialPalette.error` - Error state color
- `MaterialPalette.on-error` - Text/icons on error
- `MaterialPalette.error-container` - Error message backgrounds
- `MaterialPalette.on-error-container` - Text/icons on error-container

**Typical Colors:**
- Light mode: Red (#BA1A1A)
- Dark mode: Light red (#F2B8B5)

**Best for:** Errors, warnings, delete actions, critical states

---

### 5. SURFACE COLORS
Surface colors affect surfaces of components, such as cards and sheets.

- `MaterialPalette.surface` - Base surface color
- `MaterialPalette.on-surface` - Text/icons on surface
- `MaterialPalette.surface-variant` - Emphasized surface
- `MaterialPalette.on-surface-variant` - Text/icons on surface-variant

**Typical Colors:**
- Light mode: White/off-white (#FFFBFE)
- Dark mode: Dark gray (#1C1B1F)

**Best for:** Cards, sheets, dialogs, main backgrounds

---

### 6. SURFACE CONTAINER COLORS (Tonal Elevation)
These create depth through tonal variation rather than shadows.

- `MaterialPalette.surface-container-lowest` - Level 0 elevation
- `MaterialPalette.surface-container-low` - Level 1 elevation (subtle)
- `MaterialPalette.surface-container` - Level 2 elevation (default containers)
- `MaterialPalette.surface-container-high` - Level 3 elevation (hover states)
- `MaterialPalette.surface-container-highest` - Level 4 elevation (highest emphasis)

**Typical Colors (Light Mode):**
- lowest: #FFFFFF (pure white)
- low: #F7F2FA (very light tint)
- default: #F3EDF7 (light tint)
- high: #ECE6F0 (medium tint)
- highest: #E6E0E9 (darker tint)

**Typical Colors (Dark Mode):**
- lowest: #0F0D13 (almost black)
- low: #1D1B20 (very dark)
- default: #211F26 (dark)
- high: #2B2930 (lighter dark)
- highest: #36343B (lightest dark)

**Best for:** Creating depth, layering, hover states, emphasis

---

### 7. OUTLINE COLORS
Outline colors are used for borders and dividers.

- `MaterialPalette.outline` - Subtle borders
- `MaterialPalette.outline-variant` - Very subtle borders/dividers

**Typical Colors:**
- Light mode: #79747E (medium gray) / #CAC4D0 (light gray)
- Dark mode: #938F99 (light gray) / #49454F (dark gray)

**Best for:** Borders, dividers, separators

---

### 8. INVERSE COLORS
Inverse colors provide contrast for elements like snackbars.

- `MaterialPalette.inverse-surface` - Inverted surface
- `MaterialPalette.inverse-on-surface` - Text on inverse surface
- `MaterialPalette.inverse-primary` - Primary color on inverse surface

**Best for:** Snackbars, tooltips, floating elements

---

## Color Usage Guidelines

### ✅ DO:
- Use primary for main actions and key components
- Use secondary for less prominent actions
- Use tertiary for accents and variety
- Use error for destructive/critical actions
- Use surface containers for depth and hierarchy
- Match on-colors with their base (e.g., on-primary with primary)

### ❌ DON'T:
- Mix light and dark mode colors manually
- Use similar colors (gray/blue) next to each other
- Use error color for non-critical actions
- Ignore the on-color pairings (causes contrast issues)

---

## Quick Color Picker for Buttons

| Button Type | Recommended Color | Reasoning |
|------------|------------------|-----------|
| Primary action | `primary-container` → `primary` | Main user action |
| Secondary action | `secondary-container` → `secondary` | Supporting action |
| Tertiary action | `tertiary-container` → `tertiary` | Additional option |
| Delete/Remove | `error-container` → `error` | Destructive action |
| Info/Help | `tertiary-container` | Informational |
| Settings | `secondary-container` | Configuration |
| Navigation | `secondary-container` | Movement |
| Special/Highlight | `tertiary-container` | Standout feature |
| Disabled | `surface-container` | Inactive state |

---

## Example Color Combinations

### Navigation Controls:
```slint
// First/Last (endpoints)
bg-color: MaterialPalette.tertiary-container;
icon-color: MaterialPalette.on-tertiary-container;

// Prev/Next (main navigation)
bg-color: MaterialPalette.primary-container;
icon-color: MaterialPalette.on-primary-container;

// Random/Shuffle (action)
bg-color: MaterialPalette.secondary-container;
icon-color: MaterialPalette.on-secondary-container;
```

### Tool Buttons:
```slint
// Primary tool (Bounding Box)
bg-color: MaterialPalette.primary-container;
bg-active-color: MaterialPalette.primary;

// Secondary tool (Points)
bg-color: MaterialPalette.secondary-container;
bg-active-color: MaterialPalette.secondary;

// Advanced tool (Polygon)
bg-color: MaterialPalette.tertiary-container;
bg-active-color: MaterialPalette.tertiary;
```

---

## Color Contrast Requirements (WCAG)

All Material 3 color pairings meet WCAG 2.1 Level AA standards:
- Normal text: 4.5:1 contrast ratio
- Large text: 3:1 contrast ratio
- UI components: 3:1 contrast ratio

Always use the matching on-color with its base:
- `primary` with `on-primary`
- `primary-container` with `on-primary-container`
- etc.

---

## Additional Resources

- [Material 3 Color System](https://m3.material.io/styles/color/the-color-system/key-colors-tones)
- [Material Theme Builder](https://material-foundation.github.io/material-theme-builder/)
- [Slint Material Palette Docs](https://slint.dev/releases/1.0/docs/slint/src/builtins/palette)
