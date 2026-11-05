# SPEC-001: Deterministic AppKit Layout Engine & Consumer Parity Specification

- **Document ID:** SPEC-001
- **Status:** Approved / Normative
- **Version:** 1.0.0
- **Scope:** Standalone crate `egui-macview` and its integration into consumer applications on Windows 11.
- **Normative Standards:** ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, W3C DTCG Specification, ITU-T/IEEE SSIM, SemVer 2.0.0.

---

## 1. Normatieve Industriestandaarden

Elk onderdeel van dit document en het resulterende software-artefact is onderworpen aan de volgende normatieve standaarden:

1. **ISO/IEC/IEEE 42010:2011 (Architecture Description):**
   - Strikt onderscheid tussen *Stakeholders*, *Concerns* en *Viewpoints* (Model Viewpoint, View Viewpoint, Controller Viewpoint).
2. **ISO/IEC/IEEE 29119-3:2021 (Software Testing):**
   - Formele testspecificaties en oracles: *Plan-Write-Act* cadans.
   - Geen functionele code zonder vooraf gedefinieerde, falende rode tests.
3. **RFC 2119 & RFC 8174 (Normative Requirement Terminology):**
   - Sleutelwoorden `MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHOULD`, `MAY` hebben formele binding.
4. **Apple macOS Human Interface Guidelines (macOS HIG):**
   - Grondwaarheid voor componentgeometrie, interactiezones, focusringen en typografie.
5. **W3C Design Tokens Community Group (DTCG) Specification:**
   - 100% data-driven tokens via JSON (`design_tokens.json`). Zero hardcoded magic numbers of hex-kleuren.
6. **ITU-T / IEEE SSIM (Structural Similarity Index Measure):**
   - Geautomatiseerde visuele regressietesten met acceptatiedrempel $\text{SSIM} \ge 0.985$ en $\text{PMR} \le 1.5\%$.
7. **SemVer 2.0.0:**
   - Formele semantische versienummering voor publieke API-stabiliteit.

---

## 2. Systeemarchitectuur & MVC Datamodel

De architectuur splitst de presentatielaag rigoureus in drie gescheiden verantwoordelijkheden:

```
+------------------------------------------------------------------------+
|                            MODEL (State)                              |
|  - MacControlState: enabled, hovered, pressed, focused, window_state   |
|  - LocalEditBuffer: actieve tekstbuffer, dirty-flag, validatiestatus   |
+------------------------------------------------------------------------+
                                    |
                                    v
+------------------------------------------------------------------------+
|                         CONTROLLER (Layout)                            |
|  - MacFixedView / MacRow: vaste AppKit NIB compartimenten              |
|  - TabFocusChain: deterministische focus-cyclus tussen invoervelden    |
|  - Transactional Command Dispatcher: emit pas bij Enter, Tab of blur   |
+------------------------------------------------------------------------+
                                    |
                                    v
+------------------------------------------------------------------------+
|                          VIEW (Rendering)                              |
|  - Background: 1px border stroke, subpixel antialiasing                |
|  - Content: SF Pro / Inter typografie met optische tracking tabellen   |
|  - Overlay: Window-level focus ring (NSFocusRingPlacementOutside)       |
+------------------------------------------------------------------------+
```

### 2.1 Wiskundige Grondslagen

#### A. Coördinatentransformatie & Subpixel Snap Criterium
Om op Windows beeldschermen met fractionele schaling ($s = 1.25$ of $s = 1.50$) onscherpte en modderige randen uit te sluiten, MOET de rasterizer direct vectorcontouren projecteren naar fysieke hardware-pixels via het **Subpixel Snap Criterium**:

$$x_{\text{pixel}} = \frac{\lfloor x_{\text{logical}} \cdot s + 0.5 \rfloor}{s}$$

Een lijn van $1.0\,\text{pt}$ wordt op een $150\%$ scherm gerasterd als exact $1.5 \approx 2\,\text{px}$ hardware-dikte op pixelgrenzen, waardoor subpixel-uitveging wiskundig onmogelijk is.

#### B. Typografische Optische Tracking (San Francisco / Inter)
Apple's letter-tracking voor San Francisco Pro / Inter MOET worden berekend volgens de kubische polynoom:

$$\text{Tracking}(p) = a \cdot p^3 + b \cdot p^2 + c \cdot p + d$$
- $11.0\,\text{pt} \implies -0.23\,\text{pt}$ (compacte labels en invoervelden)
- $13.0\,\text{pt} \implies -0.08\,\text{pt}$ (standaard UI-tekst)
- $17.0\,\text{pt} \implies 0.00\,\text{pt}$ (titels en headers)

#### C. CoreAnimation Gedempte Veer (CASpring)
Micro-animaties MOETEN de analytische oplossing van de gedempte harmonische oscillator volgen:

$$x(t) = 1 - e^{-\zeta \omega_0 t} \left( \cos(\omega_d t) + \frac{\zeta \omega_0}{\omega_d} \sin(\omega_d t) \right)$$
- Responsietijd: $T_0 = 0.35\,\text{s} \implies \omega_0 = \frac{2\pi}{0.35} \approx 17.95\,\text{rad/s}$
- Dempingsfactor: $\zeta = 0.85$ (kritisch-ondergedempt, geen harde stuit maar organisch vloeiend)
- Gedempte frequentie: $\omega_d = \omega_0 \sqrt{1 - \zeta^2} \approx 9.46\,\text{rad/s}$

---

## 3. De 4 Kritische Randvoorwaarden (De Opgeloste Valkuilen)

1. **Tab-Focus Chain:**
   - Invoercellen implementeren een expliciete `next_focus_id`.
   - Drukken op `Tab` verplaatst de focus onmiddellijk naar het volgende veld in de cyclus ($X \to Y \to \Delta X \to \Delta Y \to \text{LSB} \to \text{RSB} \to W$) binnen hetzelfde frame, zonder focusverlies of cursor-sprongen.
2. **Lokale Edit-Buffering & Transactionele Binding:**
   - Een invoerveld bindt lokaal aan een `String`.
   - Wijzigingen worden PAS naar het applicatiemodel (undo/redo transactiestack) gecommit bij `Enter`, `Tab` of focusverlies. Dit voorkomt dat het bewerken van een getal de actieve canvas-selectie verbreekt.
3. **Vector Thumbnail Renderer (Shape Sidebar):**
   - De renderer MOET een willekeurige `kurbo::BezPath` realtime schalen en fitten binnen een $24 \times 24\,\text{pt}$ bounding box met behoud van aspect ratio.
   - De contour wordt getekend met een $1.0\,\text{pt}$ stroke en semi-transparante fill ($15\%$ dekkend).
4. **Typografische Metriek Pariteit:**
   - De UI gebruikt San Francisco / Inter metrics met fixed baseline alignment ($14.0\,\text{pt}$ baseline vanaf bovenzijde cel), zodat cijfers en iconen exact uitlijnen.

---

## 4. Specificatie van de Drie Functionele Stappen

### 4.1 Stap 1: Floating Infobox & Top Toolbar
- **Base Infobox Afmetingen:** $178.0\,\text{pt} \times 63.0\,\text{pt}$, radius $6.0\,\text{pt}$.
  - Header compartiment: $22.0\,\text{pt}$ hoog.
  - Body compartiment: $41.0\,\text{pt}$ hoog.
  - Rij 1: Invoercellen voor $X$, $Y$, breedte $44.0\,\text{pt}$, hoogte $18.0\,\text{pt}$.
  - Verticale tussenruimte: exact $1.0\,\text{pt}$.
  - Rij 2: Invoercellen voor LSB, RSB, Advance Width, hoogte $18.0\,\text{pt}$.
  - Totale hoogte body: $18.0 + 1.0 + 18.0 = 37.0\,\text{pt} \le 41.0\,\text{pt}$ $\implies$ **Zero Clipping Garantie**.
- **Wing Compartiment:** $84.0\,\text{pt} \times 44.0\,\text{pt}$, rechts onderaan uitgelijnd met de basis.
- **Volledige Popover:** $178.0\,\text{pt} \times 259.0\,\text{pt}$, geanimeerd via `CASpring`.
- **Top Toolbar:** Hoogte $44.0\,\text{pt}$, achtergrond `#F6F6F6`, onderrand $1.0\,\text{pt}$ `#D7D7D7`.
  - Iconen: $20.0 \times 20.0\,\text{pt}$ vector template met $32.0 \times 28.0\,\text{pt}$ interactiezones.

### 4.2 Stap 2: Shape Sidebar in de GlyphEditor
- **Outline Boomstructuur:**
  - Paden (`Path 1`, `Path 2`): mini-vector thumbnail, richting-indicator (wijzers van de klok mee / tegen de klok in), knooppunt-telling.
  - Componenten (`Component: a.sc`): mini-vector thumbnail, basisnaam, vergrendelstatus.
- **Interactieve Toggles:**
  - Oog-icoon (zichtbaarheid): klik toggles rendering op het canvas.
  - Slot-icoon (vergrendeling): klik bevriest punten tegen bewerking.
- **Bidirectionele Selectie:**
  - Klikken op een rij in de sidebar selecteert het corresponderende pad op het canvas.
  - Selecteren van knooppunten op het canvas markeert de corresponderende rij in de sidebar.

### 4.3 Stap 3: Axes Sidebar (De 3 UX States)
- **State A (Single Master Font):**
  - Weergave van een informatieve lege-staat kaart met hint: *"Geen assen geconfigureerd in Font Info"*.
- **State B (Statische Multi-Master):**
  - Discrete master-switcher radio-knoppen (bijv. `Light`, `Regular`, `Bold`).
  - Schakelen wisselt direct de actieve master-laag op het canvas.
- **State C (Continu Variabel Font):**
  - Traploze sliders voor geregistreerde assen (`wght`, `wdth`, `opsz`).
  - Numerieke invoercel ($18.0\,\text{pt}$) gekoppeld aan elke slider.
  - Real-time live outline interpolatie op het canvas ($60/120\,\text{FPS}$) via `patrix-core`.
  - Incompatibiliteits-indicator (rode badge) indien contouren van masters niet compatibel zijn.

---

## 5. Iteratief Uitvoeringsplan (Plan-Write-Act)

Het implementatieproces volgt een strikt stappenplan van zes iteratieve sprints. Elke stap begint met falende tests (Red) en eindigt pas na groen licht van de testsuite (Green):

```
+-------------------------------------------------------------------+
| Sprint 1: Formele W3C Design Tokens & Red COTS Test Baseline      |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Sprint 2: egui-macview Kerncomponenten & Layout Engine             |
|   - MacFixedView & MacRow                                         |
|   - MacInputCell met Tab-Chain & lokale buffer                    |
|   - CASpring & MacPopover                                         |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Sprint 3: Stap 1 — Infobox & Top Toolbar Integratie                |
|   - Wiskundige eliminatie van infobox clipping                    |
|   - Template vector icons & toolbar hit areas                     |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Sprint 4: Stap 2 — Shape Sidebar & Mini-Vector Thumbnail Renderer  |
|   - Kurbo BezPath thumbnailing (24x24pt)                          |
|   - Outline boom met oog/slot toggles                             |
|   - Bidirectionele canvas selectie-synchronisatie                 |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Sprint 5: Stap 3 — Axes Sidebar (3 UX States)                     |
|   - State A: Single Master empty card                             |
|   - State B: Multi-Master radio switcher                          |
|   - State C: Variable Font sliders & live canvas interpolatie     |
+-------------------------------------------------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
| Sprint 6: Headless Pixel-Diffing (SSIM >= 0.985) & Release         |
+-------------------------------------------------------------------+
```

---

## 6. Verificatie- & Kwaliteitscriteria (Acceptance Criteria)

1. **Geen enkele visuele clipping:** De Infobox behoudt onder alle omstandigheden zijn hoogte van $63.0\,\text{pt}$ en snijdt geen getallen of labels af.
2. **Tab-navigatie:** Het doorlopen van alle invoervelden via de `Tab`-toets werkt zonder cursorverspringing of focusverlies.
3. **Visuele Regressie:** Offscreen headless differential tests tonen een $\text{SSIM} \ge 0.985$ en $\text{PMR} \le 1.5\%$ aan ten opzichte van de macOS Golden Masters.
4. **Architecturale Zuiverheid:**
   - `egui-macview` bevat nul regels domein- of applicatielogica.
   - Alle Rust bronbestanden voldoen aan de modulaire decompositie-eis ($< 300\,\text{eSLOC}$, max 599 fysieke regels).
   - `cargo test --workspace` slaagt met $0$ fouten en $0$ warnings.
