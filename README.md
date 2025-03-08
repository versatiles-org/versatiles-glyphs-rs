[![Crates.io](https://img.shields.io/crates/v/versatiles_glyphs?label=crates.io)](https://crates.io/crates/versatiles_glyphs)
[![Crates.io](https://img.shields.io/crates/d/versatiles_glyphs?label=downloads)](https://crates.io/crates/versatiles_glyphs)
[![Code Coverage](https://codecov.io/gh/versatiles-org/versatiles-glyphs-rs/branch/main/graph/badge.svg?token=2eUtj8ick2)](https://codecov.io/gh/versatiles-org/versatiles-glyphs-rs)
[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/versatiles-org/versatiles-glyphs-rs/ci.yml)](https://github.com/versatiles-org/versatiles-glyphs-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Unlicense-green)](https://unlicense.org/)

---

# VersaTiles Glyphs

**VersaTiles Glyphs** is a Rust-based tool and library for generating smooth SDF (Signed Distance Field) glyphs from TrueType fonts.

You can **see and test** the results for _Noto Sans_ (in several languages) here:  
[versatiles.org/versatiles-glyphs-rs](https://versatiles.org/versatiles-glyphs-rs/)

## Why?

There are many alternative projects related to glyph rendering like [font-maker](https://github.com/maplibre/font-maker), [fontnik](https://github.com/mapbox/fontnik), [node-fontnik](https://github.com/mapbox/node-fontnik), [sdf_font_tools](https://github.com/stadiamaps/sdf_font_tools), [sdf-glyph-foundry](https://github.com/mapbox/sdf-glyph-foundry) or[TinySDF](https://github.com/mapbox/tiny-sdf). But all of them violates at least one of the following criteria:
- uses precise SDF calculation using vector data directly
- render bezier curves with high precision
- code does not have major bugs
- it's an active project, that is still maintained and accepts pull requests
- it's not a wrapper, around a wrapper, around a wrapper, around a side branch of an unmaintained repository

## Installation

### Via Script

Download and install the latest precompiled binary in one step:

```bash
curl -Ls "https://github.com/versatiles-org/versatiles-glyphs-rs/raw/refs/heads/main/scripts/install.sh" | sh
```

### Via Cargo Install

To compile and install from crates.io, ensure you have Rust installed, then run:

```bash
cargo install versatiles_glyphs --features="cli"
```

### Building from Source

If you want to build the latest (potentially unreleased) version directly from GitHub:

```bash
git clone https://github.com/versatiles-org/versatiles-glyphs-rs.git
cd versatiles-glyphs-rs
cargo build --release
```

The compiled binary will be located at target/release/versatiles_glyphs.

## Usage

`versatiles_glyphs` has two subcommands `recurse` and `merge`:

### `versatiles_glyphs recurse`

Scans one or more files and directories recursively for font files and convert them into multiple directories of glyphs.

```shell
versatiles_glyphs recurse ./font/
```

If a input directory contains a `fonts.json` (like [this one](https://github.com/versatiles-org/versatiles-fonts/blob/main/fonts/Noto%20Sans/fonts.json)) it uses the files listed in there instead of scanning recursively.

The results are written to an output directory that follows the [frontend specification](https://docs.versatiles.org/compendium/specification_frontend.html#folder-assets-glyphs);
<pre>
📂 glyphs/
├── 📂 {font_id}/
│   └── 📄 {start}-{end}.pbf
├── 📄 font_families.json
└── 📄 index.json
</pre>

You can specify an output directory with the flag `-o` or `--output-directory`:
```shell
versatiles_glyphs recurse ./font/ -o glyphs
```

Alternatively you can write the resulting files as a TAR to stdout with `-t` or `--tar`:
```shell
versatiles_glyphs recurse ./font/ -t | gzip -9 > glyphs.tar.gz
```

### `versatiles_glyphs merge`

Merges one or more font files and converts them into a single directory of glyphs. Example

```shell
versatiles_glyphs merge font
```

You can also use the arguments `--output-directory` and `--tar`.

## Develop

### Font metrics

Since I could not find an official documentation on SDF glyphs - especially on how to use font metrics correctly - i heavily relied on the code of [sdf-glyph-foundry](https://github.com/mapbox/sdf-glyph-foundry/blob/master/include/mapbox/glyph_foundry_impl.hpp), [fontnik](https://github.com/mapbox/fontnik/blob/master/lib/sdf.js), [tiny-sdf](https://github.com/mapbox/tiny-sdf) and [maplibre-gl-js](https://github.com/maplibre/maplibre-gl-js/blob/main/src/render/glyph_manager.ts).

### Testing results

For every release [versatiles.org/versatiles-glyphs-rs](https://versatiles.org/versatiles-glyphs-rs/) is updated to show the resulting glyphs.

If you want to improve or expand the languages/characters being tested, you can add new strings [here](https://github.com/versatiles-org/versatiles-glyphs-rs/blob/main/pages/web/index.html#L26).

### Local Testing (Web Pages)

1. Run the build script: `./pages/build.sh`
2. Serve the folder `./pages/web/` locally (for example, using `npx http-server -sc0`, `python3 -m http.server` or `cargo install basic-http-server`)
3. Open the served page in your browser to see the changes.

## Contributing

Contributions, issues, and feature requests are very welcome!
Feel free to open an issue or pull request if you’d like to contribute, report a bug, or suggest new features.

## License

This project is distributed under the [Unlicense](https://unlicense.org/). Essentially, you can do whatever you want with the code—no attribution required.
