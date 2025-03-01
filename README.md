[![Crates.io](https://img.shields.io/crates/v/versatiles_glyphs?label=crates.io)](https://crates.io/crates/versatiles_glyphs)
[![Crates.io](https://img.shields.io/crates/d/versatiles_glyphs?label=downloads)](https://crates.io/crates/versatiles_glyphs)
[![Code Coverage](https://codecov.io/gh/versatiles-org/versatiles-glyphs-rs/branch/main/graph/badge.svg?token=2eUtj8ick2)](https://codecov.io/gh/versatiles-org/versatiles-glyphs-rs)
[![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/versatiles-org/versatiles-glyphs-rs/ci.yml)](https://github.com/versatiles-org/versatiles-glyphs-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-green)](LICENSE)

---

# VersaTiles Glyphs

A Rust-based tool (and library) for generating SDF glyphs from TrueType fonts.

You can see and test the results for "Noto Sans" in several languages here:
https://versatiles.org/versatiles-glyphs-rs/

# Install

## via script

Download and install the latest precompiled binary via script:

```shell
curl -Ls "https://github.com/versatiles-org/versatiles-glyphs-rs/raw/refs/heads/main/scripts/install.sh" | sh
```

## `cargo install`

Compile and install latest release using `cargo install`.

```shell
cargo install versatiles_glyphs --features="cli"
```

## Building from Source

Clone the repository and build it.

```shell
git clone https://github.com/versatiles-org/versatiles-glyphs-rs.git
cd versatiles-glyphs-rs
cargo build --release
```

# Develop

If you want to improve the tested languages/characters you can add new string [here](https://github.com/versatiles-org/versatiles-glyphs-rs/blob/main/pages/web/index.html#L26).

To build and test it locally, run `./pages/build.sh`, serve the folder `./pages/web/` and open it in a browser.

# Contributing

Contributions, issues, and feature requests are very welcome!
Feel free to check the issues page if you’d like to contribute or report a bug.

# License

This project is distributed under the MIT License, unless otherwise noted. See the LICENSE file for more information.
