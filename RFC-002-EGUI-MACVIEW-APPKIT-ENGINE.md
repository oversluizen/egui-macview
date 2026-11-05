# RFC-002: egui-macview — Deterministic macOS AppKit Layout, Component & Motion Engine

- **RFC Number:** RFC-002
- **Title:** Architecture, Mathematical Foundations, MVC Datamodel, and Verification Harness for `egui-macview`
- **Authors:** Larry (AI Team Orchestrator), Jaco Hoving
- **Status:** Proposed / Draft
- **Target Crate:** `egui-macview` (Standalone Rust Crate, `version = "0.1.0"`)
- **Normative Standards:** ISO/IEC/IEEE 29119-3:2021, ISO/IEC/IEEE 42010:2011, RFC 2119 / RFC 8174, W3C DTCG Specification, Apple macOS HIG, Cassowary Constraint Solving Algorithm.

---

## 1. Doel & Probleemstelling (Problem Statement)

### 1.1 De Kloof tussen `egui` en macOS AppKit
`egui` is een immediate-mode GUI framework in Rust. Het blinkt uit in reactiviteit en eenvoud, maar mist fundamentele structuren die noodzakelijk zijn voor een getrouwe macOS gebruikersinterface:
1. **Single-Pass Geometrie:** `egui` berekent de layout in één enkele pass van boven naar beneden. Hierdoor hebben containers geen weet van de benodigde ruimte van hun kinderen totdat deze getekend zijn. Dit leidt onvermijdelijk tot *clipping* of *overflow* bij dynamische teksten.
2. **Afwezigheid van Apple Auto Layout:** macOS AppKit baseert al zijn vensters en controls op een stelsel van lineaire vergelijkingen (het Cassowary-algoritme). Elementen hebben een intrinsieke inhoudsmaat (`intrinsicContentSize`), *Content Hugging Priority* en *Compression Resistance Priority*.
3. **Ontbreken van Optische Typografie:** macOS hanteert dynamische optical tracking op het San Francisco-lettertype (letterafstand varieert non-lineair per fontgrootte).
4. **Vervuiling van Verantwoordelijkheden:** Zonder strikte MVC-abstractielaag worden domeingegevens (applicatielogica) verweven met de tekenlogica van componenten.

### 1.2 Doelstelling van `egui-macview`
`egui-macview` is een **volledig programma-agnostische** abstractielaag bovenop `egui`. De crate:
- Bevat **GEEN ENKELE** referentie naar specifieke applicaties (geen fonts, geen audio, geen documenttypes).
- Implementeert een zuivere **Model-View-Controller (MVC)** architectuur.
- Biedt een wiskundig sluitende layout-engine gebaseerd op een 2-pass constraint solver (`cassowary-rs`).
- Garandeert subpixel-nauwkeurige weergave en gedrag ten opzichte van macOS AppKit, geverifieerd via geautomatiseerde differentiële beeldvergelijking (SSIM $\ge 0.985$).

---

## 2. Toegepaste Industriestandaarden

Dit ontwerp en het bijbehorende ontwikkelproces zijn gebaseerd op de volgende formele standaarden:

| Standaard | Toepassingsgebied in dit project |
| :--- | :--- |
| **ISO/IEC/IEEE 42010:2011** | *Systems and software engineering — Architecture description.* Bepaalt de formele opbouw van deze RFC: stakeholders, viewpoints, architecturale besluiten en rationale. |
| **ISO/IEC/IEEE 29119-3:2021** | *Software and systems engineering — Software testing.* Normatieve basis voor de testspecificaties, test suites, oracles en de regressiematrix. |
| **RFC 2119 & RFC 8174** | Definitie van normatieve sleutelwoorden: `MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHOULD`, `MAY`. |
| **Apple Human Interface Guidelines (macOS HIG)** | Grondwaarheid voor componentafmetingen, interactiezones, focusringen, typografische schalen en contrastratio's. |
| **Cassowary Linear Arithmetic Constraint Solver** | Het wiskundige fundament achter Apple's Auto Layout (Badros & Borning, 1998/2001). |
| **W3C DTCG (Design Tokens Community Group) Spec** | Formele specificatie voor data-driven tokens: semantische kleuren, afmetingen, radiussen en animatiecurves. |
| **ITU-T / IEEE SSIM (Structural Similarity Index)** | Wiskundige norm voor de geautomatiseerde visuele pixel-diff testen ($\text{SSIM} \ge 0.985$). |
| **SemVer 2.0.0** | Semantische versienummering voor de crate API stabiliteit. |

---

## 3. Architecturale Viewpoints & MVC Datamodel

### 3.1 Model Viewpoint (State Machines & Data)
Elk component in `egui-macview` scheidt zijn interactietoestand volledig van zijn presentatie.

```
+--------------------------------------------------------------+
|                     MacControlState                          |
+--------------------------------------------------------------+
| - is_enabled: bool                                           |
| - is_hovered: bool                                           |
| - is_pressed: bool (tracking)                                |
| - is_focused: bool (first responder)                         |
| - window_state: MacWindowState (Key, Main, Inactive)         |
| - tracking_phase: MacTrackingPhase                           |
+--------------------------------------------------------------+
```

Componenten implementeren een formele transitie-state machine:
$$\text{State}_{t+1} = f(\text{State}_t, \text{Event})$$
Waarbij $\text{Event} \in \{\text{MouseEnter}, \text{MouseLeave}, \text{MouseDown}, \text{MouseUp}, \text{FocusIn}, \text{FocusOut}, \text{WindowKeyChange}\}$.

### 3.2 View Viewpoint (Rendering & Compositing Pipeline)
De rendering volgt Apple's `CALayer` hiërarchie:
1. **Background Layer:** Geometrie met hardware-antialiasing en semantische tokenvulling.
2. **Content Layer:** Typografie (met optical tracking) en template vectoriconen.
3. **Overlay Layer (Window-level):** Buitenwaartse Focus Ring (`NSFocusRingPlacementOutside`). Deze laag wordt getekend ná alle child-rects, waardoor afkapping door parent-containers onmogelijk is.

### 3.3 Controller Viewpoint (Auto Layout via Cassowary)
In plaats van de lineaire `egui` layout-pass, introduceert `egui-macview` de `MacLayoutSolver`:
1. **Pass 1: Intrinsic Measurement (Bottom-Up):**
   Elke view rapporteert zijn eigenschappen aan de solver:
   $$W_{\text{intrinsic}} = \text{TextWidth} + \text{Padding}_{\text{left}} + \text{Padding}_{\text{right}}$$
   $$H_{\text{intrinsic}} = \text{FontCapHeight} + \text{Padding}_{\text{top}} + \text{Padding}_{\text{bottom}}$$
2. **Pass 2: Constraint Resolution (Top-Down):**
   De solver lost het stelsel op:
   $$\text{minimize} \sum w_i \cdot (\text{slack}_i + \text{surplus}_i)$$
   onder restricties van margins, paddings en ouder-begrenzingen.
3. **Pass 3: Exact Frame Dispatch:**
   `egui` krijgt uitsluitend vooraf berekende, exacte `Rect`-toewijzingen.

---

## 4. Wiskundige Grondslagen & Formules

### 4.1 Coördinatentransformatie (3-Ruimten Model)
De transformatie tussen de logische puntenruimte ($\text{pt}$), de virtuele sandbox en het fysieke apparaat ($\text{px}$) is gedefinieerd als:

$$\mathbf{p}_{\text{device}} = \mathbf{M}_{\text{viewport}} \cdot \mathbf{M}_{\text{scale}} \cdot \mathbf{p}_{\text{logical}}$$

Waarbij:
$$\mathbf{M}_{\text{scale}} = \begin{pmatrix} s & 0 & 0 \\ 0 & s & 0 \\ 0 & 0 & 1 \end{pmatrix}, \quad s = \text{pixels\_per\_point}$$

Om subpixel-onscherpte op Windows schermen met fractionele schaling ($s = 1.5$) te elimineren, geldt het **Subpixel Snap Criterium**:
$$x_{\text{pixel}} = \frac{\lfloor x_{\text{logical}} \cdot s + 0.5 \rfloor}{s}$$

### 4.2 Typografische Tracking Formule (San Francisco)
Apple's dynamische letter-tracking volgt een kubische interpolatie over de korpsgrootte $p$ (in punten) voor San Francisco Pro:

$$\text{Tracking}(p) = a \cdot p^3 + b \cdot p^2 + c \cdot p + d$$
Voor $p \in [6.0, 20.0]\,\text{pt}$:
- $11\,\text{pt} \implies -0.23\,\text{pt}$ tracking
- $13\,\text{pt} \implies -0.08\,\text{pt}$ tracking
- $17\,\text{pt} \implies 0.00\,\text{pt}$ tracking

### 4.3 CoreAnimation Fysica (CASpring)
Micro-animaties volgen de analytische oplossing van een gedempte harmonische oscillator:
$$m \ddot{x} + c \dot{x} + k x = 0$$

Geparametriseerd naar respons $T_0$ en dempingsverhouding $\zeta$:
$$\omega_0 = \frac{2\pi}{T_0}, \quad \zeta = \frac{c}{2\sqrt{m k}}$$

Voor de ondergedempte toestand ($\zeta < 1$):
$$x(t) = 1 - e^{-\zeta \omega_0 t} \left( \cos(\omega_d t) + \frac{\zeta \omega_0}{\omega_d} \sin(\omega_d t) \right)$$
waarbij $\omega_d = \omega_0 \sqrt{1 - \zeta^2}$.

---

## 5. De Test- en Verificatie-Architectuur (ISO/IEC/IEEE 29119)

### 5.1 Grondwaarheid Extractie (Ground-Truth Harvesting)
1. **macOS Native Test-Bed:** Een minimaal Swift/AppKit command-line tool compileert op macOS en instantieert native AppKit componenten.
2. **Export Matrix:**
   - Component: `NSTextField`, `NSButton`, `NSSegmentedControl`, `NSPopover`, `NSSlider`.
   - Resoluties: $1.0\times$ (Standaard), $2.0\times$ (Retina).
   - Thema's: Aqua Light, Aqua Dark, Inactive Window.
   - Toestanden: Normal, Hover, Pressed, Focused, Disabled.
   - Formaat: PNG met ongecomprimeerde RGBA8 data + JSON metadata met NIB metrics.

### 5.2 Geautomatiseerde Headless Test-Runner (In Rust)
In `egui-macview` wordt een test-harnas gebouwd via `wgpu` (headless compute/rendering):
1. **Render Pass:** Render de Rust-component in een offscreen framebuffer met identieke parameters.
2. **Differentiële Beoordeling:**
   - **PMR (Peak Mismatch Rate):** Percentage pixels met kleurverschil $\Delta E > 2.0$. Eis: $\le 1.5\%$.
   - **SSIM (Structural Similarity Index):** Wiskundige structurele gelijkenis. Eis: $\ge 0.985$.
   - **Diff Heatmap:** Bij falen wordt een automatische visual diff-afbeelding opgeslagen met afwijkingen gemarkeerd in felrood (`#FF0055`).

---

## 6. Gefaseerd Implementatieplan (Plan-Write-Act)

| Fase | Benaming | Scope & Deliverables | Acceptatiecriterium |
| :---: | :--- | :--- | :--- |
| **0** | **Spec & Tokens** | W3C DTCG JSON tokenset (alle 38 semantische AppKit kleuren en afmetingen). | Validatie tegen schema; zero hardcoded constants. |
| **1** | **Layout Solver** | Integratie van `cassowary-rs` in `egui-macview::layout`. Two-pass constraint evaluatie. | Wiskundige proof-tests: zero clipping bij dynamische breedtes. |
| **2** | **Test Harness** | Headless WGPU offscreen runner + SSIM/PMR diffing engine. | Red-baseline tests falen gecontroleerd op ontbrekende controls. |
| **3** | **Core Controls** | Implementatie van `MacTextField`, `MacButton`, `MacSegmentedControl`, `MacPopover`. | SSIM $\ge 0.985$ tegen macOS Golden Master exports. |
| **4** | **Crate Release** | Documentatie, benchmarks, voorbeelden en `v0.1.0` release. | `cargo test` 100% green, 0 warnings, clippy pedantic clean. |
