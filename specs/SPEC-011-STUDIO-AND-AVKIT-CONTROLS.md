# SPEC-011: Studio & AVKit Media, Timeline and Publishing Controls

| Attribute | Specification Value |
| :--- | :--- |
| **Document ID** | SPEC-011 |
| **Document Title** | Studio & AVKit Media, Timeline and Publishing Controls |
| **Document Version** | `0.5.0` |
| **Status** | Accepted / Normative |
| **Target Milestone** | Milestone 0.5.0 |
| **Target Host** | Standalone crate `egui-macview` (Rust, egui 0.37) |
| **Author** | Jaco Oversluizen / `egui-macview` contributors |
| **Normative Test Suite** | `tests/studio_controls_v05_tests.rs`, `tests/zero_hardcoding_and_primitive_tests.rs`, `tests/wcag_contrast_and_visuals_tests.rs` |
| **Normative Standards** | ISO/IEC/IEEE 42010:2011, ISO/IEC/IEEE 29119-3:2021, RFC 2119/8174, Apple macOS HIG, Apple AppKit / AVKit Reference, W3C DTCG, SemVer 2.0.0 |

---

## Revision History

| Version | Date | Author | Description / Milestone |
| :--- | :--- | :--- | :--- |
| `0.5.0` | 2026-11-08 | Architecture Team | Milestone 0.5.0 Release: Formal specification of 6 domain-agnostic AppKit & AVKit studio controls (`MacProgressIndicator`, `MacTableView`, `MacTokenField`, `MacLevelIndicator`, `MacTrimScrubber`, `MacPathControl`) and W3C DTCG token extensions |

---

## 1. Scope & Normative Foundation

Modern macOS media production, non-linear editing, audio mastering, and structured publishing suites (such as Final Cut Pro, Logic Pro, QuickTime Player, and Finder) rely on six fundamental AppKit and AVKit controls beyond standard inspector primitives:
1. **`MacProgressIndicator` (`NSProgressIndicator`):** Determinate (`CASpring`-smoothed) and indeterminate bar and radial fin-spinner progress indicators across `Regular`, `Small`, and `Mini` control sizes.
2. **`MacTableView` (`NSTableView` + `NSTableHeaderView`):** Multi-column tabular data grid with sortable column headers, alternating row backgrounds, single/range/multi-row selection, keyboard arrow navigation, and interactive cell closures.
3. **`MacTokenField` (`NSTokenField`):** Multi-pill tag and token input field with flow-wrapping layout, delimiter-driven pill creation (`,` or `Enter`), `Backspace` and `×` pill removal, outer focus halo, and optional character budget counter.
4. **`MacLevelIndicator` (`NSLevelIndicator`):** Continuous and discrete segmented signal/audio level meter in horizontal and vertical orientations with normal (green), warning (yellow/amber), and critical (red) thresholds plus peak-hold marker.
5. **`MacTrimScrubber` (`AVPlayerView` / QuickTime In–Out Scrubber):** Combined normalized waveform timeline, named semantic zones, draggable In/Out trim handles with outside dimming and accent selection frame, and draggable playhead with `MM:SS.d` timecode badge.
6. **`MacPathControl` (`NSPathControl`):** Interactive breadcrumb navigation bar with vector folder/document iconography, `ChevronRight` separators, per-segment click feedback, and optional trailing `Choose...` action button.

All six controls SHALL be 100% domain-agnostic and 100% data-driven from `tokens/design_tokens.json` via `MacTheme`, adhering strictly to Design Token Driven Architecture (Zero Magic Numbers, Zero Cross-Component Token Borrowing, and Modularity Ceilings $< 400$ eSLOC / $< 599$ physical lines).

---

## 2. W3C DTCG Color & Token Extensions (`v0.5.0`)

### 2.1 Unmultiplied RGBA Color Parsing (`REQ-011-00`)
To ensure translucent DTCG tokens (`#RRGGBBAA` and `rgba(r, g, b, a)`) preserve exact sRGB chromaticity when composited by `egui::Painter` or evaluated by WCAG contrast algorithms, `parse_color32` SHALL construct `egui::Color32` via `Color32::from_rgba_unmultiplied(r, g, b, a)`.

### 2.2 Semantic Appearance Colors (`MacAppearanceColors`)
Each appearance (`aqua` and `dark_aqua`) in `tokens/design_tokens.json` SHALL define (calibrated against native `NSColor` sRGB extraction on macOS Sonoma 14.8.9 via PyObjC):
- `system_yellow`: `#FFCC00` (`rgb(255, 204, 0)`, Aqua) / `#FFD60A` (`rgb(255, 214, 10)`, DarkAqua) — `NSColor.systemYellowColor`
- `system_orange`: `#FF9500` (`rgb(255, 149, 0)`, Aqua) / `#FF9F0A` (`rgb(255, 159, 10)`, DarkAqua) — `NSColor.systemOrangeColor`
- `system_red`: `#FF3B30` (`rgb(255, 59, 48)`, Aqua) / `#FF453A` (`rgb(255, 69, 58)`, DarkAqua) — `NSColor.systemRedColor`
- `alternating_row_even`: `#FFFFFF` (`rgb(255, 255, 255)`, Aqua) / `#1E1E1E` (`rgb(30, 30, 30)`, DarkAqua) — `NSColor.alternatingContentBackgroundColors[0]`
- `alternating_row_odd`: `#F4F5F5` (`rgb(244, 245, 245)`, Aqua) / `rgba(255, 255, 255, 0.05)` ($\alpha \approx 0.0471$, DarkAqua) — `NSColor.alternatingContentBackgroundColors[1]`
- `scrubber_selection_border`: `#FFCC00` (Aqua) / `#FFD60A` (DarkAqua) — QuickTime / AVKit yellow trim frame
- `scrubber_outside_dim`: `rgba(0, 0, 0, 0.58)` (Aqua & DarkAqua) — AVKit / QuickTime trimmed-out region scrim overlay
- `scrubber_waveform`: `#007AFF` (Aqua) / `#0A84FF` (DarkAqua) — normalized waveform amplitude fill
- `scrubber_playhead`: `#FFFFFF` (Aqua & DarkAqua) — AVKit white playhead indicator needle

---

## 3. Control Specifications & Mathematical Models

### 3.1 `MacProgressIndicator` (`NSProgressIndicator`) — `REQ-011-01`
- **Reference:** [Apple Developer Documentation — `NSProgressIndicator`](https://developer.apple.com/documentation/appkit/nsprogressindicator)
- **Control Sizes (`MacProgressControlSize`) — Calibrated to AppKit `NSProgressIndicator` & WebKit `RenderThemeMac.mm`:**
  - `Regular` (`NSControlSizeRegular`): bar track height $6.0\,\text{pt}$ (within $20.0\,\text{pt}$ AppKit fitting frame), spinner diameter $32.0 \times 32.0\,\text{pt}$, font size $12.0\,\text{pt}$.
  - `Small` (`NSControlSizeSmall`): bar track height $4.0\,\text{pt}$ (within $12.0\,\text{pt}$ AppKit fitting frame), spinner diameter $16.0 \times 16.0\,\text{pt}$, font size $11.0\,\text{pt}$.
  - `Mini` (`NSControlSizeMini`): bar track height $4.0\,\text{pt}$ (within $12.0\,\text{pt}$ AppKit fitting frame), spinner diameter $12.0 \times 12.0\,\text{pt}$ (native `spin_fitting_h` $10.0\,\text{pt}$), font size $9.0\,\text{pt}$.
- **Determinate Bar Style (`MacProgressStyle::Bar`):**
  - Given input progress $p \in (-\infty, \infty)$, the clamped fraction is $\hat{p} = \text{clamp}(p, 0.0, 1.0)$.
  - Active fill width is $W_{\text{fill}} = W_{\text{track}} \cdot \hat{p}$, smoothed across frames via `CASpring` interpolation when animated.
- **Indeterminate Bar Style:**
  - Renders a sliding pulse pill of width $W_{\text{pulse}} = W_{\text{track}} \cdot \text{indeterminate\_pill\_ratio}$ oscillating smoothly across $[0, W_{\text{track}} - W_{\text{pulse}}]$ with period $1 / f_{\text{bar}}$ where $f_{\text{bar}} = \text{bar\_indeterminate\_speed}$.
- **Spinning Style (`MacProgressStyle::Spinning`):**
  - Renders $N = \text{spoke\_count}$ ($12$) radial fin spokes at $30^\circ$ intervals ($\theta_k = \frac{2\pi k}{12}$) for $k \in \{0, \dots, 11\}$ from inner radius $R_{\text{in}} = R \cdot \text{spoke\_inner\_ratio}$ to outer radius $R_{\text{out}} = R$.
  - Discrete 12 fps step animation ($1.0\,\text{s}$ per full rotation, $0.0833\,\text{s}$ per step): at time $t$, active spoke index is $k_0 = \lfloor t \cdot \text{spin\_speed\_hz} \cdot N \rfloor \pmod N$, and each spoke $k$ is tinted with `theme.secondary_label()` modulated by discrete phase opacity $\alpha_k$ decaying monotonically with angular distance $(k_0 - k) \pmod N$.

### 3.2 `MacTableView` (`NSTableView` + `NSTableHeaderView`) — `REQ-011-02`
- **Reference:** [Apple Developer Documentation — `NSTableView`](https://developer.apple.com/documentation/appkit/nstableview)
- **Column Layout (`MacTableColumn`):**
  - Columns specify `title`, `width: f32`, `alignment: egui::Align`, and `sort_direction: Option<MacSortDirection>` (`Ascending` or `Descending`).
  - Header bar has default height $H_{\text{hdr}} = \text{header\_height}$ ($28.0\,\text{pt}$ macOS Sonoma `NSTableHeaderView` ground truth; $24.0\,\text{pt}$ compact HIG configurable), bottom separator hairline, vertical column dividers, and clickable sort chevrons (`MacSymbol::ChevronUp` / `MacSymbol::ChevronDown` in `theme.secondary_label()`). Clicking a header returns `clicked_header: Option<usize>` in `MacTableViewResponse`.
- **Alternating Row Backgrounds & Selection:**
  - Row $i \in \{0, \dots, N_{\text{rows}}-1\}$ occupies vertical interval $[y_0 + i \cdot H_{\text{row}}, y_0 + (i+1) \cdot H_{\text{row}}]$ where $H_{\text{row}} = \text{row\_height}$ ($24.0\,\text{pt}$ `.inset` medium default; `.small` $17.0\,\text{pt}$, `.medium` plain $20.0\,\text{pt}$, `.large` $28.0\,\text{pt}$ configurable via `.row_height(...)`).
  - When `uses_alternating_row_backgrounds` is true, unselected odd rows ($i \bmod 2 == 1$) fill with `theme.alternating_row_odd()` (`#F4F5F5` Aqua / `rgba(255, 255, 255, 0.05)` DarkAqua), and even rows with `theme.alternating_row_even()` (`#FFFFFF` Aqua / `#1E1E1E` DarkAqua).
  - Selection (`BTreeSet<usize>`):
    - Plain click on row $i$: sets selection to $\{i\}$ and anchor to $i$.
    - `Cmd`/`Ctrl` + click on row $i$: toggles membership of $i$ in selection.
    - `Shift` + click on row $i$: selects inclusive range $[\min(a, i), \max(a, i)]$ from anchor $a$.
  - Keyboard Navigation: When focused and enabled, `Key::ArrowDown` and `Key::ArrowUp` advance the active selection to $\min(i+1, N_{\text{rows}}-1)$ or $\max(i-1, 0)$ respectively, calling `ui.input_mut(|i| i.consume_key(Modifiers::NONE, key))` so key events never leak to an underlying canvas.

### 3.3 `MacTokenField` (`NSTokenField`) — `REQ-011-03`
- **Reference:** [Apple Developer Documentation — `NSTokenField`](https://developer.apple.com/documentation/appkit/nstokenfield)
- **Flow-Wrapping Pill Layout:**
  - Outer field minimum height `min_height` is $22.0\,\text{pt}$ (matching macOS Sonoma `NSTokenField` frame height ground truth).
  - Displays `tokens: &mut Vec<String>` as `.rounded` capsules (`pill_height` $16.0\,\text{pt}$, `pill_corner_radius` $8.0\,\text{pt}$ = exact half-height capsule, interior horizontal padding `pill_padding_horizontal` $6.0\,\text{pt}$) followed by an inline single-line text input buffer (`input_buffer: &mut String`).
  - Pills wrap onto a new row whenever $x_{\text{cursor}} + W_{\text{pill}} > x_{\text{right}} - \text{padding\_horizontal}$, incrementing $y_{\text{cursor}}$ by $\text{pill\_height} + \text{row\_spacing}$.
- **Token Commit & Deletion Mechanics:**
  - Typing a delimiter from `defaultTokenizingCharacterSet` (comma `,` or newline/Enter) inside `input_buffer` or pressing `Key::Enter` trims the non-empty buffer, appends it to `tokens`, clears `input_buffer`, and consumes `Key::Enter` via `ui.input_mut(|i| i.consume_key(...))`.
  - Pressing `Key::Backspace` when `input_buffer.is_empty()` pops the last token from `tokens` and consumes `Key::Backspace`.
  - Clicking the trailing `×` button (`remove_icon_size` $12.0\,\text{pt}$) on pill $i$ removes `tokens[i]`.
- **Character Counter:**
  - When `.max_chars(max)` is configured, renders `<total_chars> / <max>` at the bottom-right corner using `counter_font_size` ($10.0\,\text{pt}$), switching to `theme.system_red()` when $\text{total\_chars} > \text{max}$.

### 3.4 `MacLevelIndicator` (`NSLevelIndicator`) — `REQ-011-04`
- **Reference:** [Apple Developer Documentation — `NSLevelIndicator`](https://developer.apple.com/documentation/appkit/nslevelindicator)
- **Dimensions:**
  - Standard thickness `thickness` is $16.0\,\text{pt}$ (matching AppKit regular `NSLevelIndicator` height; native fitting frame $18.0\,\text{pt}$), discrete segment spacing `segment_spacing` $2.0\,\text{pt}$.
- **Normalized Signal Mapping:**
  - Given range $[v_{\min}, v_{\max}]$ with $v_{\max} > v_{\min}$, any level $v$ maps to normalized ratio:
    $$\hat{v} = \text{clamp}\left(\frac{v - v_{\min}}{v_{\max} - v_{\min}}, 0.0, 1.0\right)$$
- **Semantic Color Zones:**
  - For a segment or position at value $x \in [v_{\min}, v_{\max}]$:
    $$\text{Color}(x) = \begin{cases} \text{theme.system\_red}() & \text{if } x \ge v_{\text{crit}} \\ \text{theme.system\_yellow}() & \text{if } v_{\text{warn}} \le x < v_{\text{crit}} \\ \text{theme.system\_green}() & \text{if } x < v_{\text{warn}} \end{cases}$$
- **Styles & Orientations:**
  - `MacLevelIndicatorStyle::Continuous`: Draws a continuous filled bar up to $\hat{v}$, partitioned at $v_{\text{warn}}$ and $v_{\text{crit}}$.
  - `MacLevelIndicatorStyle::Discrete`: Draws $S = \text{segment\_count}$ discrete blocks separated by $\text{segment\_spacing}$ ($2.0\,\text{pt}$), illuminating blocks whose threshold $\le \hat{v}$.
  - `MacLevelOrientation::Horizontal` fills left-to-right; `MacLevelOrientation::Vertical` fills bottom-to-top.
  - Optional `peak_hold_value: Option<f32>` draws a high-contrast marker line of thickness `peak_line_width` ($2.0\,\text{pt}$) at $\hat{v}_{\text{peak}}$.

### 3.5 `MacTrimScrubber` (`AVPlayerView` / QuickTime Scrubber) — `REQ-011-05`
- **Reference:** [Apple Developer Documentation — `AVPlayerView.canBeginTrimming`](https://developer.apple.com/documentation/avkit/avplayerview/canbegintrimming)
- **Timeline Coordinate Bijection:**
  - Given duration $D > 0$, timeline track inner horizontal span $[x_L, x_R]$ of width $W = x_R - x_L$:
    $$X(t) = x_L + \text{clamp}\left(\frac{t}{D}, 0.0, 1.0\right) \cdot W, \qquad T(x) = \text{clamp}\left(\frac{x - x_L}{W}, 0.0, 1.0\right) \cdot D$$
- **Trim & Playhead Ordering Invariants:**
  - For all interaction frames, the scrubber enforces:
    $$0.0 \le t_{\text{in}} \le t_{\text{out}} \le D \quad \land \quad t_{\text{in}} \le t_{\text{playhead}} \le t_{\text{out}}$$
  - Dragging the left trim handle updates $t_{\text{in}} = \min(T(x_{\text{ptr}}), t_{\text{out}})$ and clamps $t_{\text{playhead}} = \max(t_{\text{playhead}}, t_{\text{in}})$.
  - Dragging the right trim handle updates $t_{\text{out}} = \max(T(x_{\text{ptr}}), t_{\text{in}})$ and clamps $t_{\text{playhead}} = \min(t_{\text{playhead}}, t_{\text{out}})$.
  - Dragging the playhead updates $t_{\text{playhead}} = \text{clamp}(T(x_{\text{ptr}}), t_{\text{in}}, t_{\text{out}})$.
- **Visual Layers:**
  1. Named background zones (`&[MacTimelineZone]`) with subtle tint and label.
  2. Symmetrical vertical waveform bars for `&[f32]` normalized amplitudes $\in [0.0, 1.0]$.
  3. Dimmed scrim overlay rectangles over $[x_L, X(t_{\text{in}})]$ and $[X(t_{\text{out}}), x_R]$ using `theme.scrubber_outside_dim()` (`rgba(0, 0, 0, 0.58)`).
  4. QuickTime selection border (`theme.scrubber_selection_border()`, `selection_border_width` $2.0\,\text{pt}$) around $[X(t_{\text{in}}), X(t_{\text{out}})]$ with left/right grab handles of width `handle_width` ($12.0\,\text{pt}$) and vertical grip slots ($1.0\,\text{pt} \times 10.0\,\text{pt}$).
  5. Playhead vertical needle (`playhead_width` $2.0\,\text{pt}$) at $X(t_{\text{playhead}})$ with top timecode pill badge (`badge_height` $14.0\,\text{pt}$, `badge_corner_radius` $7.0\,\text{pt}$, `badge_font_size` $11.0\,\text{pt}$) formatted as `MM:SS.d`.

### 3.6 `MacPathControl` (`NSPathControl`) — `REQ-011-06`
- **Reference:** [Apple Developer Documentation — `NSPathControl`](https://developer.apple.com/documentation/appkit/nspathcontrol)
- **Breadcrumb Segments (`MacPathComponent`):**
  - Intrinsic height `height` is $22.0\,\text{pt}$ (matching macOS Sonoma `NSPathControl` fitting height), icon bounding size `icon_size` $16.0\,\text{pt}$ (or proportional vector folder/document icon), icon-to-text gap `icon_spacing` $4.0\,\text{pt}$, separator chevron size `chevron_size` $8.0\,\text{pt}$ in `theme.secondary_label()`.
  - Renders a horizontal sequence of path components separated by `MacSymbol::ChevronRight`.
  - Clicking segment $i$ sets `clicked_segment = Some(i)` on `MacPathControlResponse`.
- **Trailing Choose Button:**
  - When `.show_choose_button(true)` is active, renders a compact trailing `Choose...` push button at the right edge of the path bar; clicking it sets `choose_clicked = true` on `MacPathControlResponse`.

---

## 4. ISO/IEC/IEEE 29119-3 Traceability Matrix

| Requirement ID | Control / Subsystem | Test Case ID | Verification Method | Acceptance Criterion |
| :--- | :--- | :--- | :--- | :--- |
| `REQ-011-00` | DTCG Unmultiplied Color & Tokens | `TC-011-00` | Unit & WCAG Test | `parse_color32("#12345678")` equals `Color32::from_rgba_unmultiplied(0x12, 0x34, 0x56, 0x78)` and all v0.5.0 tokens resolve with 0 hardcoding. |
| `REQ-011-01` | `MacProgressIndicator` | `TC-011-01` | Headless `egui` + `proptest` | Determinate bar clamps $[0, 1]$, indeterminate and spinning modes animate smoothly across `Regular`, `Small`, and `Mini` sizes. |
| `REQ-011-02` | `MacTableView` | `TC-011-02` | Headless `egui` + `proptest` | Header click toggles sort column; single, Shift-range, and Cmd-toggle selection and consumed ArrowUp/ArrowDown keys maintain valid row indices. |
| `REQ-011-03` | `MacTokenField` | `TC-011-03` | Headless `egui` + `proptest` | Comma/Enter commits token and consumes key; Backspace on empty buffer or clicking `×` removes token; flow-wrap expands height when tokens wrap. |
| `REQ-011-04` | `MacLevelIndicator` | `TC-011-04` | Headless `egui` + `proptest` | Continuous & discrete modes in horizontal & vertical orientations map $[v_{\min}, v_{\max}]$ and switch green/yellow/red at warning/critical thresholds. |
| `REQ-011-05` | `MacTrimScrubber` | `TC-011-05` | Headless `egui` + `proptest` | Dragging In, Out, and Playhead handles strictly preserves $0 \le t_{\text{in}} \le t_{\text{playhead}} \le t_{\text{out}} \le D$ across all pointer coordinates. |
| `REQ-011-06` | `MacPathControl` | `TC-011-06` | Headless `egui` | Clicking breadcrumb segment $i$ reports `clicked_segment == Some(i)` and clicking `Choose...` reports `choose_clicked == true`. |

---

## 5. Revision History

| Version | Date | Author | Description |
| :--- | :--- | :--- | :--- |
| `0.1.0` | 2026-11-05 | AI Architecture Team | Initial `egui-macview` specification suite baseline. |
| `0.5.0` | 2026-11-08 | AI Architecture Team | Normative specification for v0.5.0 Studio & AVKit controls (`MacProgressIndicator`, `MacTableView`, `MacTokenField`, `MacLevelIndicator`, `MacTrimScrubber`, `MacPathControl`). |
