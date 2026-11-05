# Architecture Specification: `egui-macview`

**A Deterministic macOS AppKit Layout Engine, Authentic Control Primitive Suite, and CoreAnimation Physics Runtime for egui.**

---

## 1. Executive Summary & Design Vision

`egui-macview` is an independent, cross-platform Rust library built on top of `egui` and `wgpu`. Its primary mission is to bridge the architectural chasm between **macOS AppKit (Retained Mode, Strict AutoLayout & MVC)** and **egui (Immediate Mode, Streaming Content-Driven Flow)**.

In standard `egui`, widgets dynamically push cursor positions based on font metrics, automatic margins, and padding. When attempting to recreate pixel-exact macOS interfaces designed in AppKit NIB files (such as Glyphs 4, Sketch, or Xcode), immediate-mode streaming causes layout instability, unpredictable margins, and clipping.

`egui-macview` introduces a formal **MacView / Model-View-Controller (MVC)** abstraction layer:
1. **Mathematical Determinism**: Coordinate bounds, cell dimensions ($18.0\,\text{pt}$ compact inputs), and container frames are calculated *prior* to rendering. Clipping and overflowing become mathematically impossible.
2. **Authentic macOS Controls**: Pixel-perfect controls matching Apple Human Interface Guidelines (HIG) with native template image tinting, 1px hairlines, and subpixel baseline alignment.
3. **CoreAnimation Physics**: High-fidelity reproduction of Apple's `CASpringAnimation` utilizing analytical closed-form differential equation integration.

---

## 2. Cross-Platform Display & Rendering Stack Comparison

To achieve true macOS visual fidelity on non-macOS hardware (Windows DirectX and Linux Wayland/X11), the engine accounts for the four structural layers of the operating system display pipeline:

### 2.1 The 4 Architectural Layers per Platform

| Layer | macOS (Apple) | Windows (Microsoft) | Linux (X11 / Wayland) |
| :--- | :--- | :--- | :--- |
| **1. UI Framework (Widgets)** | **AppKit** (`NSView`, Cocoa) & SwiftUI | **WinUI 3 / WPF / Win32** (`HWND`) | **GTK 4 / Qt 6** |
| **2. 2D Vector & Font Engine** | **Quartz 2D** (`CoreGraphics`) & `CoreText` | **Direct2D** & `DirectWrite` (legacy GDI) | **Cairo** & `FreeType` / `HarfBuzz` |
| **3. Compositor & Windowing** | **CoreAnimation** (`Quartz Compositor`) | **DWM** (Desktop Window Manager) | **Mutter / KWin** (Wayland / X11) |
| **4. Hardware Driver API** | **Metal** | **DirectX** (Direct3D 11/12) | **Vulkan** & OpenGL (Mesa) |

---

## 3. Fundamental Graphics & Coordinate Differences

### 3.1 Coordinate System Origins
- **macOS / Quartz 2D**: Origin is Cartesian **Bottom-Left $(0, 0)$**, with $y$ ascending upwards. In AppKit, views flip this to Top-Left via `isFlipped = YES`.
- **Windows & Linux**: Origin is Top-Left $(0, 0)$, with $y$ descending downwards.
- **`egui-macview` Standard**: Normalized Top-Left $(0, 0)$ in logical AppKit Points (`pt`). Conversion from Cartesian geometries is handled cleanly at the view-projection layer.

### 3.2 Typographic Philosophy & Font Antialiasing
- **macOS (Quartz / CoreText)**:
  - *Core Principle*: **Optical Consistency (Shape Preservation)**.
  - CoreText never distorts letterforms to align with hardware grid boundaries. Text is rasterized using pure **Greyscale Antialiasing**. Outlines appear slightly fuller and weighted exactly as the type designer authored them.
- **Windows (Direct2D / DirectWrite / ClearType)**:
  - *Core Principle*: **High-Contrast Legibility via Pixel Hinting**.
  - DirectWrite snaps stems and crossbars directly to physical pixel boundaries. This yields high legibility on low-DPI displays (96 DPI), but slightly distorts outline proportions and makes letterforms appear thinner than on macOS.
- **Linux (FreeType)**:
  - Configurable hinting modes (`hintslight`, `hintfull`).

### 3.3 Color Space & Gamma Curves
- **macOS Displays**: Default to wide-gamut **Display P3** with a linear/P3 gamma curve.
- **Windows & Linux**: Default to standard **sRGB** with a $2.2$ gamma curve.
- *Impact*: Raw RGB hexadecimal codes (e.g. `#0078D4` or `#F6F6F6`) produce different chromatic and luminance sensations across displays without explicit color profile management.

---

## 4. The 3 Coordinate Spaces: Model vs. View vs. Screen

A typographic studio or precision graphics tool operates in three strictly isolated dimensional domains:

```
[ 1. MODEL SPACE ]         Font Units (u)          UPM = 1000u, LSB = 45u
        │
        ▼ (Viewport Transform: Zoom & Pan)
[ 2. VIEW SPACE ]          AppKit Points (pt)      Container = 250x63 pt, Cell = 54x18 pt
        │
        ▼ (Display Scale: 1.0x / 1.5x / 2.0x)
[ 3. SCREEN SPACE ]        Device Pixels (px)      Hardware raster on display panel
```

### 4.1 Concrete Dimensional Calculation Example
Consider a standard macOS Glyphs 4 Inspector Infobox on a Windows display configured at **150% fractional scaling** (`scale = 1.5`):

1. **Model Domain**:
   - The active glyph ('H') defines a Left Sidebearing of $45\,\text{u}$.
   - The Controller retrieves $45\,\text{u}$ and formats the string `"45"`. The Model knows nothing about UI bounds.
2. **View Domain (`egui-macview`)**:
   - The Infobox container has fixed NIB bounds: **$250.0 \times 63.0\,\text{pt}$**.
   - Compartment budget: Header takes $22.0\,\text{pt}$; lower metrics area has **$41.0\,\text{pt}$**.
   - Row 1 Cell Height: $18.0\,\text{pt}$
   - Vertical Gap: $1.0\,\text{pt}$
   - Row 2 Cell Height: $18.0\,\text{pt}$
   - **Total View Budget Claimed**: $18.0 + 1.0 + 18.0 = \mathbf{37.0\,\text{pt}}$.
   - **Result**: $37.0\,\text{pt} \le 41.0\,\text{pt}$ $\implies$ **$4.0\,\text{pt}$ margin remaining**.
3. **Screen Domain (150% Display Hardware)**:
   - GPU Rasterization: $\text{px} = \text{pt} \times 1.5$.
   - Infobox Width: $250.0 \times 1.5 = \mathbf{375.0\,\text{px}}$.
   - Infobox Height: $63.0 \times 1.5 = \mathbf{94.5\,\text{px}}$.
   - Cell Rows Total: $37.0 \times 1.5 = \mathbf{55.5\,\text{px}}$.
   - Lower Compartment: $41.0 \times 1.5 = \mathbf{61.5\,\text{px}}$.
   - **Mathematical Proof of Zero-Clipping**: $61.5\,\text{px} - 55.5\,\text{px} = \mathbf{6.0\,\text{px}}$ surplus hardware margin. Row 2 can never clip.

---

## 5. Overcoming Hardware Divergence: "De 4 Listen" (The 4 Strategies)

### Strategy 1: Virtual Retina 2.0× Offscreen Sandbox (Automated Visual Diff Testing)
- **Problem**: Testing on a 150% Windows screen against official Apple Golden Master PNGs (captured on a 2.0x Retina MacBook Pro) produces false mismatch errors due to scaling differentials.
- **Solution**: The headless test harness runs in a virtual offscreen framebuffer locked to:
  - Exact **$2.000\times$ Retina scale factor**.
  - Greyscale Antialiasing (eliminating Windows ClearType color fringes).
  - Standardized sRGB/P3 color conversion.
- **Outcome**: The test pipeline generates bit-exact renders directly comparable to macOS ground truth.

### Strategy 2: Native Vector GPU Meshing (Zero Downsampling Blur)
- **Problem**: Rendering an interface into a 2.0x bitmap and downscaling it via GPU to a 150% screen creates spatial interpolation blur (the classic muddy Windows scaling effect).
- **Solution**: `egui-macview` **never** resamples bitmaps. It outputs direct GPU vector vertex meshes (`wgpu`) rendered at the target display's exact native physical resolution, rasterizing font curves directly for that scale.

### Strategy 3: Closed-Form Analytical CASpring Integration
- **Problem**: Frame-iterative Euler spring physics (`pos += vel * dt`) diverges between 60Hz office screens, 120Hz ProMotion MacBooks, and 144Hz/240Hz gaming panels.
- **Solution**: We employ the closed-form analytical solution of the damped harmonic oscillator:
  $$x(t) = x_{\text{target}} + e^{-\zeta \omega_n t} \left( C_1 \cos(\omega_d t) + C_2 \sin(\omega_d t) \right)$$
  where $\omega_n = \frac{2\pi}{\text{response}}$ and $\zeta = \text{dampingRatio}$.
- **Outcome**: Position at any millisecond $t$ is mathematically exact across any hardware refresh rate.

### Strategy 4: Perceptual Color & Visual Regression Tolerance (ISO 29119)
- **Problem**: Minor gamma differences between Display P3 and sRGB cause raw byte-by-byte assertions (`p1 == p2`) to fail.
- **Solution**: Automated regression audits utilize Structural Similarity Index (SSIM) and Pixel Mismatch Ratio (PMR) with a perceptual color tolerance threshold ($\Delta E_{2000}$ or RGB tolerance $\le 15/255$).
- **Passing Quality Gates**: **SSIM $\ge 0.985$**, **PMR $\le 1.0\%$**.

---

## 6. Architecture & Module Decomposition

```
crates/egui-macview/
├── Cargo.toml
├── README.md
├── ARCHITECTURE.md                  (This canonical reference)
├── src/
│   ├── lib.rs                       (Crate root & public facade)
│   ├── layout/
│   │   ├── mod.rs
│   │   ├── mac_frame.rs             (NIB geometry, Rect pt math)
│   │   ├── fixed_view.rs            (Absolute pt positioning)
│   │   └── flex_view.rs             (AutoLayout springs & struts)
│   ├── controls/
│   │   ├── mod.rs
│   │   ├── input_cell.rs            (18pt zero-margin compact cell)
│   │   ├── button_cell.rs           (AppKit template image tinting)
│   │   ├── segmented_control.rs     (Segmented pill switcher)
│   │   ├── toggle_switch.rs         (macOS pill toggle)
│   │   └── slider.rs                (Continuous axis slider)
│   ├── motion/
│   │   ├── mod.rs
│   │   ├── ca_spring.rs             (Closed-form damped oscillator)
│   │   └── timing_functions.rs      (Apple ease-in / ease-out curves)
│   └── surfaces/
│       ├── mod.rs
│       ├── panel.rs                 (NSPanel HUD floating container)
│       └── popover.rs               (NSPopover with anchor chevron)
└── tests/
    ├── layout_oracle_tests.rs       (Mathematical zero-clipping proofs)
    ├── spring_physics_tests.rs      (Damped oscillator convergence)
    └── visual_regression_tests.rs   (Headless SSIM diffing)
```
