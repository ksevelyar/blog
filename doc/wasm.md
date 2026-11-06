# Wasm
## Requirements
* Embedded components are written in Rust.
* Embedded component Rust code is compiled to WebAssembly.
* A post can embed the circle component.
* The circle is visible in the generated page.
* The circle repeatedly changes color from purple to lavender and back.

## Implementation
* Each embedded component is a separate Rust crate compiled for `wasm32-unknown-unknown`.
* The circle component creates an SVG circle in its mount element.
* The circle component uses `requestAnimationFrame` to update its SVG fill color.
* The component builder returns generated artifact paths without writing site output.
* The site output writer copies component artifacts into the generated site directory.
* Playwright feature tests are in `tests/features/markdown.spec.ts` and `tests/features/wasm.spec.ts`.

## User scenarios
* Given a post containing the circle shortcode, when its generated page is opened, then an SVG circle is visible.
* Given a visible circle, when its fill color is sampled twice over time, then the sampled colors differ.
* Given a visible circle, when its fill color is sampled over its animation, then it transitions between purple and lavender.
