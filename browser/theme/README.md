# @mayflower-sys/theme

Mayflower's shared palette, typography, spacing, motion, and logomarks for web apps, pamphlets, and flowcharts. Change brand values in [`src/tokens.css`](src/tokens.css). [`src/tailwind.css`](src/tailwind.css) maps those variables to Tailwind v4 utilities.

## React and Tailwind v4

Add the package from the consuming workspace app.

```sh
pnpm add "@mayflower-sys/theme@workspace:*"
```

Import Tailwind and the theme in the app's main stylesheet.

```css
@import "tailwindcss";
@import "@mayflower-sys/theme";
@import "@mayflower-sys/theme/typography.css"; /* optional editorial defaults */
```

```tsx
export function MarketSummary({ title }: { title: string }) {
  return (
    <section className="bg-card text-card-foreground border border-border p-6">
      <h2 className="font-serif text-xl leading-snug">{title}</h2>
    </section>
  )
}
```

The root import includes the shared variables, Tailwind mappings, and document defaults. Import `@mayflower-sys/theme/tailwind.css` for the variables and Tailwind mappings. React apps that use plain CSS can import `@mayflower-sys/theme/tokens.css` and opt into `base.css` and `typography.css` as needed.

## Plain HTML and inline SVG

Serve the stylesheets from `src/` alongside the document. The browser loads them directly.

```html
<link rel="stylesheet" href="./theme/tokens.css" />
<link rel="stylesheet" href="./theme/base.css" />
<link rel="stylesheet" href="./theme/typography.css" />
<link rel="stylesheet" href="./theme/fonts.css" />

<h1>Reward Clearing</h1>
<svg viewBox="0 0 240 80" role="img" aria-label="Bid flow">
  <rect x="0" y="0" width="240" height="80" fill="var(--accent-soft)" />
  <text
    x="16"
    y="48"
    fill="var(--fg-on-accent)"
    style="font-family: var(--font-sans); font-size: var(--text-lg)"
  >
    Submit a bid
  </text>
</svg>
```

`base.css` provides the document background, body type, selection colors, and focus outline. `typography.css` adds Fraunces headings and the `.label`, `.eyebrow`, `.meta`, `.lead`, `.numeric`, and `.ticker` helpers. Inline SVG inherits the document's tokens.

## Standalone SVG

When viewing the SVG as a document, link the token stylesheet with an XML stylesheet instruction above the opening `<svg>` tag.

```xml
<?xml-stylesheet type="text/css" href="./theme/tokens.css"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 240 80">
  <rect width="240" height="80" fill="var(--primary)" />
  <path d="M24 40H216" stroke="var(--primary-foreground)" stroke-width="2" />
</svg>
```

For an SVG shared as an image, embed the contents of `tokens.css` in a `<style>` element inside the SVG. A Node-based generator can read the packaged stylesheet with `readFileSync(new URL(import.meta.resolve("@mayflower-sys/theme/tokens.css")), "utf8")` from `node:fs` and place it in a `<![CDATA[...]]>` section. Image contexts disable external stylesheet loading. [SVG 2 processing modes](https://www.w3.org/TR/SVG2/conform.html)

For renderers that require literal SVG paint values, resolve the theme in a browser and write the computed `fill` and `stroke` into the exported SVG. Embed font data or convert text to paths when the output needs to carry its typography with it.

## Fonts and artwork

The font stacks use Fraunces for display and Inter for UI and body copy. To load them, link [`fonts.css`](src/fonts.css) as a separate stylesheet or supply the fonts through your app's existing font loader. The package's loader uses Google Fonts, and the stacks include local fallbacks.

The two logomark SVGs are self-contained. Import `@mayflower-sys/theme/assets/logomark.svg` or `@mayflower-sys/theme/assets/logomark-inverted.svg` as an asset URL in a bundler or copy the files from [`src/assets/`](src/assets/) alongside a document.

[`splash.svg`](src/assets/splash.svg) stacks the logomark above an outlined Fraunces Semibold wordmark. It switches colors with the viewer's light or dark preference and works as an image in a README. Import it as `@mayflower-sys/theme/assets/splash.svg`.

## Using the tokens

| Purpose               | CSS variables                                                    | Tailwind utilities                                                    |
| --------------------- | ---------------------------------------------------------------- | --------------------------------------------------------------------- |
| Page and cards        | `--background`, `--foreground`, `--card`, `--card-foreground`    | `bg-background`, `text-foreground`, `bg-card`, `text-card-foreground` |
| Primary action        | `--primary`, `--primary-foreground`                              | `bg-primary`, `text-primary-foreground`                               |
| Brand pink            | `--accent`, `--accent-foreground`                                | `bg-accent`, `text-accent-foreground`                                 |
| Soft accent surface   | `--accent-soft`, `--fg-on-accent`                                | `bg-accent-soft`, `text-accent-soft-foreground`                       |
| Lines and data series | `--border`, `--chart-1` through `--chart-5`                      | `border-border`, `fill-chart-1`, `stroke-chart-2`                     |
| Type and rhythm       | `--font-serif`, `--text-xl`, `--tracking-wide`, `--leading-snug` | `font-serif`, `text-xl`, `tracking-wide`, `leading-snug`              |
| Spacing and shape     | `--space-5`, `--radius`, `--shadow`                              | `p-6`, `rounded-none`, `shadow-none`                                  |
| Motion                | `--duration-fast`, `--ease-brand`, `--animate-fade-in`           | `duration-fast`, `ease-brand`, `animate-fade-in`                      |

`--space-1` is the 4px unit used by Tailwind's numeric spacing utilities. The named CSS steps `--space-1` through `--space-10` are 4, 8, 12, 16, 24, 32, 48, 64, 96, and 128px.

Apply `.dark` or `data-theme="dark"` to set the inverted palette on the document or a section. Both also activate Tailwind's `dark:` variant. The light palette is the default.

## Brand guidance

See the [Mayflower brand guidance](BRAND.md) for visual and editorial conventions.
