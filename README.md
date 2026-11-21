# Nimbus

A UI framework and engine.

Developed under the QAOS umbrella, Nimbus combines a declarative UI markup language (**QDL**), a Vulkan-powered 2D rendering pipeline, dynamic Material Design 3 / Monet palette generation, and interactive physics-based spring animations.

Other languages: [tr](README.tr.md)

---

## Features

- **GPU-Accelerated 2D Renderer:** Low-level Vulkan abstraction (`nimbus_rhi_ash`) driving a custom 2D canvas with procedural GPU noise shaders and high-quality SDF/font glyph atlas rendering (`fontdue`).
- **Declarative QDL Markup:** Clean and human-friendly UI markup language for defining layout trees, style blocks, and event bindings.
- **Dynamic Theming (Monet):** OKLCH-based dynamic tonal palettes, elevation tokens, radius and stroke styling with full dark/light theme support.
- **Spring Animations:** Responsive spring physics (`Spring::INTERACTIVE`, `BOUNCY`, `SMOOTH`) and modern easing functions (`EmphasizedDecelerate`, etc.).
- **Component Library (`nimbus_ui`):**
  - **Inputs & Controls:** Button (Filled, Tonal, Elevated, Outlined), Checkbox, Radio, Switch, Slider, Interval slider, TextInput (`Edit`).
  - **Navigation & Selection:** Tabs, Choice chips, Popup menus (with animated submenus and custom chevron indicators).
  - **Feedback & Display:** ProgressBar, MultiProgressBar, Tooltip / ToolTip (with directional positioning and fade/slide animations).
  - **Layout & Containers:** HBox, VBox, Flow, Card, ScrollView.

---

## Visuals

![Nimbus Interface Test](/images/test.png)
*Interface test.*

---

## License

This project is licensed under the GNU General Public License version 3 (GPL3).

Copyright (c) 2025-2026 Kadir Aydın.

---

## QAOS

Built with ❤️ by the open-source community.
