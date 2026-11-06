# SPEC-001: Deterministic AppKit Layout Engine & Consumer Parity Specification

- **Document ID:** SPEC-001
- **Status:** Approved / Normative
- **Version:** 0.2.0
- **Scope:** Standalone crate `egui-macview` and its integration into consumer applications on Windows 11.
- **Normative Standards:** ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG Specification, ITU-T/IEEE SSIM, SemVer 2.0.0.

---

## 1. Normatieve Industriestandaarden & Decompositie

Dit document definieert het overkoepelende specificatiekader voor `egui-macview`. De formele deelspecificaties zijn ondergebracht in de modulaire specificatiesuite onder `specs/`:

1. [**SPEC-001**](specs/SPEC-001-DESIGN-TOKENS-AND-WCAG.md): Design Tokens, Optical Tracking & WCAG AA Contrast.
2. [**SPEC-002**](specs/SPEC-002-CONTROLS-AND-MICRO-ANIMATIONS.md): Authentic AppKit Controls & Micro-Animation Oracles.
3. [**SPEC-003**](specs/SPEC-003-LAYOUT-ENGINES.md): Deterministic Layout Engines & Subpixel Snap Rasterization.
4. [**SPEC-004**](specs/SPEC-004-CORE-ANIMATION-PHYSICS.md): CoreAnimation Spring Physics Runtime (`CASpring`).
5. [**SPEC-005**](specs/SPEC-005-WINDOW-FRAMING-AND-CONTROLS.md): Window Framing, Drag Handling & Configurable Caption Controls.
6. [**SPEC-006**](specs/SPEC-006-INSPECTOR-ARCHITECTURE.md): Three-Tier Inspector Architecture & Floating Surfaces.

---

## 2. Systeemarchitectuur & MVC Datamodel

De architectuur splitst de presentatielaag rigoureus in drie gescheiden verantwoordelijkheden:

```
+------------------------------------------------------------------------+
|                            MODEL (State)                              |
|  - MacControlState: enabled, hovered, pressed, focused, window_state   |
|  - LocalEditBuffer: actieve tekstbuffer, dirty-flag, validatiestatus   |
|  - WindowControlsStyle: AppleTrafficLights, WindowsCaptionButtons,      |
|                         NativeHost                                     |
+------------------------------------------------------------------------+
                                    |
                                    v
+------------------------------------------------------------------------+
|                         CONTROLLER (Layout)                            |
|  - MacFixedView / MacRow / MacStackView: AppKit layout engines         |
|  - TabFocusChain: deterministische focus-cyclus tussen invoervelden    |
|  - Window drag handling & corner-fling hit areas (Fitts's Law)         |
+------------------------------------------------------------------------+
                                    |
                                    v
+------------------------------------------------------------------------+
|                          VIEW (Rendering)                              |
|  - Background: 1px border stroke, subpixel antialiasing                |
|  - Content: SF Pro / Inter typografie met optische tracking tabellen   |
|  - Caption Controls: Apple traffic lights links of Windows rechts      |
|  - Overlay: Window-level focus ring (NSFocusRingPlacementOutside)       |
+------------------------------------------------------------------------+
```

---

## 3. Strategie 2: Configureerbare Window Controls (`WindowControlsStyle`)

Window caption controls volgen **Strategie 2 (Token-Driven / Configureerbare Presentatie)**, waarmee dynamisch geschakeld kan worden tussen macOS traffic lights, Windows 11 caption buttons of OS-managed vensterdecoraties:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowControlsStyle {
    #[default]
    AppleTrafficLights,
    WindowsCaptionButtons,
    NativeHost,
}
```

### 3.1 `AppleTrafficLights` (Leading / Linkerzijde macOS Stijl)
- **Plaatsing & Marges:**
  - Verankerd aan de linkerzijde met $18.0\,\text{pt}$ inset (`toolbar.traffic_lights_inset` = 70.0pt totale marge inclusief cluster).
  - Verticaal gecentreerd in de unified toolbar ($38.0\,\text{pt}$ / $52.0\,\text{pt}$).
- **Schijfjes Geometrie:**
  - Straal $r = 6.0\,\text{pt}$ (diameter $12.0\,\text{pt}$, token `toolbar.traffic_light_radius`).
  - Onderlinge afstand $= 8.0\,\text{pt}$ (steek $20.0\,\text{pt}$, token `toolbar.traffic_light_spacing`).
  - Totale clusterbreedte: $52.0\,\text{pt}$.
- **Semantische Kleuren:**
  - Close: Fill `traffic_light_close` (`#FF5F56`), stroke $0.5\,\text{pt}$ `traffic_light_close_border` (`#E0443E`).
  - Minimize: Fill `traffic_light_minimize` (`#FFBD2E`), stroke $0.5\,\text{pt}$ `traffic_light_minimize_border` (`#DEA123`).
  - Zoom: Fill `traffic_light_zoom` (`#27C93F`), stroke $0.5\,\text{pt}$ `traffic_light_zoom_border` (`#1AAB29`).
- **Cluster-Brede Hover Affordance:**
  - Hoveren over enig deel van de cluster onthult gelijktijdig de symbolen op alle drie de knoppen.
  - Close: Diagonaal kruis `✕`, span $5.0\,\text{pt}$, stroke $1.0\,\text{pt}$.
  - Minimize: Horizontale balk `—`, span $5.6\,\text{pt}$, stroke $1.1\,\text{pt}$.
  - Zoom: Plusteken `+`, span $5.0\,\text{pt}$, stroke $1.0\,\text{pt}$.

### 3.2 `WindowsCaptionButtons` (Trailing / Rechterzijde Fluent Stijl)
- **Plaatsing & Fitts's Law:**
  - Verankerd aan de rechterzijde met $0.0\,\text{pt}$ marge: de buitenste rand van de Sluitknop raakt direct de vensterrand ($x = W_{\text{window}}$) ter ondersteuning van hoek-kliks volgens de wet van Fitts.
  - Volgorde: Minimaliseren (`—`), Maximaliseren/Herstellen (`□` / `❐`), Sluiten (`✕`).
- **Dimensies:**
  - Knopbreedte: $46.0\,\text{pt}$ (`toolbar.caption_button_width`).
  - Knophoogte: Beslaat de volledige hoogte van de titelbalk.
  - Totale groepbreedte: $138.0\,\text{pt}$.
- **Hover & Actieve Statussen:**
  - Minimaliseren / Maximaliseren: Neutrale hover-achtergrond (`rgba(255, 255, 255, 0.08)` donker, `rgba(0, 0, 0, 0.06)` licht).
  - Sluitknop: Karmozijnrode hover-achtergrond `#E81123`. Bij hover kleurt het kruissymbool helder wit (`#FFFFFF`).
- **Vector Glyphs:**
  - Minimaliseren: Horizontale lijn `—`, span $10.0\,\text{pt}$, stroke $1.0\,\text{pt}$.
  - Maximaliseren: Vierkante omtrek `□`, $10.0 \times 10.0\,\text{pt}$, stroke $1.0\,\text{pt}$. Herstelmodus toont twee overlappende vierkanten van $8.0\,\text{pt}$.
  - Sluiten: $45^\circ$ diagonaal kruis `✕`, $10.0 \times 10.0\,\text{pt}$, stroke $1.0\,\text{pt}$.

### 3.3 `NativeHost`
- Schakelt synthetische vensterknoppen volledig uit.
- Besturingssysteem levert de decoraties.
