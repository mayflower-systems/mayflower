# Mayflower brand guidance

Mayflower's visual language uses flat surfaces, a strict grid, compact data density, and whitespace as an active layout element. Use the light palette for the main surface and dark sections for occasional inversions.

## Palette

- **Mayflower Pink** draws from trailing arbutus. Use it for selection states, category chips, secondary actions, chart highlights, and feature cards.
- **Slate Navy** is a deep blue-black for headlines, primary actions, dark feature cards, and chart strokes.
- **Paper** has a faint warm tint and serves as the page background. Use **Mist** for secondary blocks and alternating table rows.
- **Ink** provides the neutral range. Use the semantic foreground tokens for body, metadata, and disabled text.
- Use muted green and red for positive and negative change. Use the chart tokens for data series.

Pair each surface token with its intended foreground. The [token examples](README.md#using-the-tokens) show the pairings for page, cards, actions, and pink surfaces.

## Typography

Use Fraunces for display headings and key numeric readouts. Use Inter for body copy, navigation, labels, tables, and buttons. These fonts are the substitutions from the original design handoff.

Set labels in uppercase with `--tracking-wider`. Use tabular lining figures in the sans for tables and tickers. Use tabular serif figures for display values.

## Layout and imagery

Align content to an 8-point grid. Use a 12-column grid on large viewports, with cards aligned to the grid and 24px gutters. Typical card padding is 24 or 32px. Typical section padding is 64 or 96px. Place labels just above their values, with 8px between them.

Use square corners and flat fills. Separate sections with whitespace or a change in surface color. Use the hairline border for definition where needed, such as between table rows or around a card on Paper. Mist works for default cards, Slate Navy for inverted cards, and soft pink for feature cards.

Use small, geometric icons, typically 16–20px, with a 1.75 stroke weight where applicable. Icons inherit the surrounding text color and can sit in square Mist or pale pink badges. For photography, favor cool-toned, architectural imagery with high contrast and low saturation.

## Interaction

Keep motion brisk. Use the shared timing and easing tokens for fades and state changes. An animated chart can use a simple path draw.

Underline links on hover and highlight hovered table rows with Mist. Use a pink left rule for active sidebar items. Make hover and press states visible through color changes. Use the pink focus outline for keyboard navigation.

## Voice

Write with the precision of a builder of financial infrastructure. Use short, declarative sentences. For product copy, use first-person plural when speaking for the project and address the reader directly when describing their actions.

- **Headlines.** Use title case.
- **Labels and tickers.** Use uppercase for short labels and ticker symbols. Use the middle dot to separate categories, as in “EQUITY · DIVERSIFIED.”
- **Navigation and actions.** Use title case in public navigation and uppercase in dense product navigation and action labels.
- **Body.** Use sentence case. Show the precision a value needs, such as `$1,024.12` for a currency amount, `+1.24%` for a change, and `8.4%` for a yield.
