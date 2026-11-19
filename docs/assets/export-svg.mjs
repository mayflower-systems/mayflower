import { readFile } from "node:fs/promises"

const [sourcePath] = process.argv.slice(2)
if (process.argv.length !== 3) {
  console.error("Usage: node docs/assets/export-svg.mjs <source.svg>")
  process.exit(1)
}

const [svg, tokens] = await Promise.all([
  readFile(sourcePath, "utf8"),
  readFile(
    new URL("../../browser/theme/src/tokens.css", import.meta.url),
    "utf8",
  ),
])

const themeLink =
  /^<\?xml-stylesheet\s+type="text\/css"\s+href="(?:\.\.\/)+browser\/theme\/src\/(?:tokens|fraunces)\.css"\?>\r?\n/gm
const svgRoot = /^<svg\b(?:[^<>"']|"[^"]*"|'[^']*')*>/m
if (!svgRoot.test(svg)) {
  throw new Error(`Expected an SVG root in ${sourcePath}`)
}

const defaults = tokens
  .replace(/\/\*[\s\S]*?\*\//g, "")
  .match(/:root\s*\{([^}]+)\}/)?.[1]
if (!defaults) {
  throw new Error("Expected theme variables in :root")
}
const values = new Map(
  Array.from(
    defaults.matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g),
    ([, name, value]) => [name, value.trim().replace(/\s+/g, " ")],
  ),
)

function references(text) {
  return Array.from(text.matchAll(/var\(\s*(--[\w-]+)/g), ([, name]) => name)
}

const required = new Set(references(svg))
for (const name of required) {
  for (const dependency of references(values.get(name) ?? "")) {
    required.add(dependency)
  }
}

let css = Array.from(values)
  .filter(([name]) => required.has(name))
  .map(([name, value]) => `    ${name}: ${value};`)
  .join("\n")
css = `  :root {\n${css}\n  }`

if (required.has("--font-serif")) {
  const fontStylesheet = new URL(
    "../../browser/theme/src/fraunces.css",
    import.meta.url,
  )
  const fontCss = await readFile(fontStylesheet, "utf8")
  const fontSource = /url\("([^"]+)"\)/.exec(fontCss)
  if (!fontSource) {
    throw new Error("Expected a font URL in fraunces.css")
  }
  const [font, license] = await Promise.all([
    readFile(new URL(fontSource[1], fontStylesheet)),
    readFile(new URL("./fonts/OFL.txt", fontStylesheet), "utf8"),
  ])
  const embeddedFont = fontCss.replace(
    fontSource[0],
    `url("data:font/woff2;base64,${font.toString("base64")}")`,
  )
  css = `/*\n${license.trimEnd()}\n*/\n${embeddedFont.trimEnd()}\n${css}`
}

css = css
  .replaceAll("&", "&amp;")
  .replaceAll("<", "&lt;")
  .replaceAll(">", "&gt;")

const result = svg
  .replace(themeLink, "")
  .replace(
    svgRoot,
    (openingTag) => `${openingTag}\n  <style>\n${css}\n  </style>`,
  )
process.stdout.write(result)
