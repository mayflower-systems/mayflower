# Documentation assets

The incentive flowcharts use a fixed palette of eleven colors from the [Mayflower theme](../../browser/theme/src/tokens.css), plus shared font families and letter spacing.

- [Overview](incentive-flow-charts/00-master.svg)
- [Earn credits](incentive-flow-charts/01-earn.svg)
- [Direct future emission](incentive-flow-charts/02-direct.svg)
- [Spend credits](incentive-flow-charts/03-spend.svg)

| Theme token  | Use                                       |
| ------------ | ----------------------------------------- |
| `--paper`    | Page, light cards, and lettering on slate |
| `--navy-900` | Headings and the Mayflower mark           |
| `--navy-800` | Slate panels and reserve-asset flows      |
| `--navy-500` | Body text                                 |
| `--navy-400` | Secondary text and asset B                |
| `--pink-500` | Credit-flow arrows                        |
| `--pink-600` | Rose text                                 |
| `--pink-200` | Asset C and logo chevrons                 |
| `--pink-100` | Pink panels                               |
| `--mist`     | Gray panels                               |
| `--ink-100`  | Hairline dividers                         |

## Use in a webpage or app

Load `@mayflower-sys/theme/tokens.css` and render the markup starting at `<svg>` inline. Its fills, strokes, and type inherit the page or section's tokens. Open a source SVG directly from a local checkout to preview it with the same stylesheet.

## Export a standalone image

Run this from the repository root to embed the tokens used by a figure in a copy for GitHub Markdown, an `<img>` element, or sharing as a file. The exporter writes an ordinary `<style>` block containing the required variables from the theme's default palette. Figures that use `--font-serif` also include the bundled Fraunces font through `@font-face`.

```sh
node docs/assets/export-svg.mjs docs/assets/incentive-flow-charts/00-master.svg > /tmp/mayflower-incentives.svg
```

Image contexts require the embedded stylesheet because they disable external stylesheet loading. [SVG 2 processing modes](https://www.w3.org/TR/SVG2/conform.html#referencing-modes)

The [example article](../articles/mayflower-incentives-demo.md) embeds the generated copies in `incentive-flow-charts/standalone/`. Regenerate those copies after editing the chart sources, theme tokens, or bundled font.

```sh
for svg in docs/assets/incentive-flow-charts/*.svg; do
  node docs/assets/export-svg.mjs "$svg" \
    > "docs/assets/incentive-flow-charts/standalone/${svg##*/}"
done
```

## Reuse styles and shapes

CSS classes apply the same styling to multiple elements. A group can supply inherited paint or typography to its children. Put reusable geometry in `<defs>` and reference it with `<use>`.

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 480 100">
  <style>
    .panel { fill: var(--pink-100); }
    .label { fill: var(--navy-900); font-family: var(--font-sans); }
  </style>
  <defs>
    <rect id="panel" width="220" height="80" class="panel" />
  </defs>
  <use href="#panel" x="0" y="10" />
  <use href="#panel" x="260" y="10" />
  <g class="label" font-size="20">
    <text x="24" y="56">Earn credits</text>
    <text x="284" y="56">Spend credits</text>
  </g>
</svg>
```

With the theme loaded in the host document, the inline figure inherits its variables. Exporting the figure includes the variables needed for a standalone image. Fraunces lives in the theme package with its license and is embedded in the exports that use it. The source figures link the local Fraunces stylesheet for previewing directly from the checkout. [SVG styling](https://www.w3.org/TR/SVG2/styling.html)
