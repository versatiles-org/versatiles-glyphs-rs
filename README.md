[![Crates.io](https://img.shields.io/crates/v/versatiles_glyphs?label=crates.io)](https://crates.io/crates/versatiles_glyphs)
[![Crates.io](https://img.shields.io/crates/d/versatiles_glyphs?label=downloads)](https://crates.io/crates/versatiles_glyphs)
[![Code Coverage](https://codecov.io/gh/versatiles-org/versatiles-glyphs-rs/branch/main/graph/badge.svg?token=2eUtj8ick2)](https://codecov.io/gh/versatiles-org/versatiles-glyphs-rs)
[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/versatiles-org/versatiles-glyphs-rs/ci.yml)](https://github.com/versatiles-org/versatiles-glyphs-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Unlicense-green)](https://unlicense.org/)

---

# VersaTiles Glyphs

**VersaTiles Glyphs** is a Rust tool and library for generating signed distance field (SDF) glyphs from TrueType fonts. It aims for high-precision rendering by working directly with font vector outlines—no additional C++ libraries or wrappers required.

**See the results** for _Noto Sans_ (in several languages) here:  
[versatiles.org/versatiles-glyphs-rs](https://versatiles.org/versatiles-glyphs-rs/)

---

## Why Another Glyph Tool?

There are numerous glyph rendering projects — e.g., [font-maker](https://github.com/maplibre/font-maker), [fontnik](https://github.com/mapbox/fontnik), [node-fontnik](https://github.com/mapbox/node-fontnik), [sdf_font_tools](https://github.com/stadiamaps/sdf_font_tools), [sdf-glyph-foundry](https://github.com/mapbox/sdf-glyph-foundry), and [TinySDF](https://github.com/mapbox/tiny-sdf). However, many have tradeoffs such as low rendering precision, unmaintained code, or "unfavourable architecture".

**VersaTiles Glyphs**:
- Renders SDF with maximum precision directly from the raw vector data.
- Renders also bezier curves with high precision.
- No external wrappers or complicated build steps.
- Actively maintained, welcomes contributions.

## Installation

### 1. Via Installation Script

Use a single shell command to download and install the latest precompiled binary:

```bash
curl -Ls "https://github.com/versatiles-org/versatiles-glyphs-rs/raw/refs/heads/main/scripts/install.sh" | sh
```

### 2. Via Cargo

Install from crates.io using Rust’s package manager:

```bash
cargo install versatiles_glyphs
```

### 3. From Source

To build the latest (potentially unreleased) version:

```bash
git clone https://github.com/versatiles-org/versatiles-glyphs-rs.git
cd versatiles-glyphs-rs
cargo build --release
```

The compiled binary will be located at target/release/versatiles_glyphs.

## Usage

`versatiles_glyphs` provides two main subcommands: `recurse` and `merge`.

### Subcommand: `recurse`

Recursively scans fonts from one or more directories or files, converting them into glyph sets:

```bash
versatiles_glyphs recurse ./font/
```

If a directory contains a `fonts.json` (like [this example](https://github.com/versatiles-org/versatiles-fonts/blob/main/fonts/Noto%20Sans/fonts.json)), it uses the files from that JSON instead of a raw file scan.

Output follows the [frontend specification](https://docs.versatiles.org/compendium/specification_frontend.html#folder-assets-glyphs):

<pre>
📂 glyphs/
├── 📂 {font_id}/
│   └── 📄 {start}-{end}.pbf
├── 📄 font_families.json
└── 📄 index.json
</pre>

Specify an output directory with `-o` or `--output-directory`:

```bash
versatiles_glyphs recurse ./font/ -o glyphs
```

Generate a TAR archive instead of directories, with `-t` or `--tar`:

```bash
versatiles_glyphs recurse ./font/ --tar | gzip -9 > glyphs.tar.gz
```

### Subcommand: `merge`

Merges one or more font files into a single directory of glyphs:

```bash
versatiles_glyphs merge ./font/
```

It supports the same `--output-directory` and `--tar` options.

## Development Notes

### Documentation

You can find the latest documentation at [docs.rs/versatiles_glyphs](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/).

### Quick Overview

- Font files are added to a [`FontManager`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/font/struct.FontManager.html), which scans their [metadata](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/font/struct.FontMetadata.html) and [parses the font name](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/font/fn.parse_font_name.html) to guess the font family, style, weight, width …
- Font files of the same font (e.g. when a font is split into multiple files, each for a different language) are combined in a [`FontWrapper`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/font/struct.FontWrapper.html).
- The [`FontManager`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/font/struct.FontManager.html) can [render all glyphs and write them](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/font/struct.FontManager.html#method.render_glyphs) to one of two [`Writer`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/writer/trait.Writer.html)s: [`FileWriter`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/writer/struct.FileWriter.html) or [`TarWriter`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/writer/struct.TarWriter.html)
- Glyphs are rendered serially per [`GlyphBlock`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/font/struct.GlyphBlock.html). Each block contains a maximum of 256 glyphs. The blocks are rendered in parallel.
- A single glyph is rendered with [`render_glyph`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/render/fn.render_glyph.html) using [`RendererPrecise`](https://docs.rs/versatiles_glyphs/latest/versatiles_glyphs/render/struct.RendererPrecise.html).

### Font Metrics & Precision

Since no official SDF-glyph spec for all metrics could be found, most references come from:
- [sdf-glyph-foundry](https://github.com/mapbox/sdf-glyph-foundry/blob/master/include/mapbox/glyph_foundry_impl.hpp)
- [fontnik](https://github.com/mapbox/fontnik/blob/master/lib/sdf.js)
- [tiny-sdf](https://github.com/mapbox/tiny-sdf)
- [maplibre-gl-js](https://github.com/maplibre/maplibre-gl-js/blob/main/src/render/glyph_manager.ts).

### Testing Online

Every new release is showcased at [versatiles.org/versatiles-glyphs-rs](https://versatiles.org/versatiles-glyphs-rs/).
If you’d like to expand or alter characters tested, [edit these lines](https://github.com/versatiles-org/versatiles-glyphs-rs/blob/main/pages/web/index.html#L27).

### Local Web Testing
1.	Run the build script: `./pages/build.sh`
2.	Serve the `./pages/web/` directory (e.g., using `npx http-server -sc0`, `python3 -m http.server` or `cargo install basic-http-server`)
3.	Visit it in your browser to check changes.

## Contributing

Issues and pull requests are always welcome. Join the community by reporting bugs, improving documentation, or adding new features!

## License

This project is distributed under the [Unlicense](https://unlicense.org/). Essentially, you can do whatever you want with the code—no attribution required.
