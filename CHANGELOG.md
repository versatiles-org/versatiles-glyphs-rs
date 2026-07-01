# Changelog

All notable changes to this project are documented in this file.
It is generated from the commit history by [git-cliff](https://git-cliff.org).

## v0.9.1 - 2026-07-01

### 🐛 Bug Fixes
- Optimize get_blocks method and ensure to create all blocks
- Remove unnecessary reference in candidate segment query for improved clarity
- Update dependencies to latest versions for improved stability and performance

### 🧪 Testing
- Enhance FontWrapper tests to ensure all BMP ranges are emitted, including empty blocks

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.9.0...v0.9.1

## v0.9.0 - 2026-04-27

### 🐛 Bug Fixes
- Remove unnecessary dead code annotations and clarify comments in geometry and rendering modules
- Define constants for glyph size and SDF radius to improve clarity and maintainability
- Ensure idempotent finish behavior in Writer and improve drop finalization
- Improve is_empty method logic and add tests for edge cases in BBox
- Enhance script token normalization by adding comprehensive list for font family parsing
- Refactor Bezier curve approximation to use iterative stack-based approach for improved stability
- Update sorting method for font families to use cmp for improved clarity
- Optimize segment handling and improve distance calculation in renderer_precise function
- Update documentation and implementation of bitmap_as_digit_art function for clarity and accuracy
- Improve error handling in write_string function to prevent buffer overflow
- Handle potential None case for file extension in scan function
- Enhance documentation on SDF rendering and bbox rounding artifacts, close #8
- Improve error handling for reading glyph files in `run` function
- Remove unnecessary trailing whitespace in `run` function
- Add `is_empty` method to `GlyphBlock` for better empty check and update documentation in `FontMetadata`
- Remove unused test module from debug command
- Change panic strategy from unwind to abort for improved performance
- Update documentation references to use `Self` for improved clarity
- Enhance CI workflow by adding documentation build and cargo audit steps
- Change CI permissions from write to read for improved security
- Optimize regex initialization in name_to_id function for improved performance
- Update package metadata in Cargo.toml for improved clarity and documentation
- Enhance documentation and safety in FontFileEntry::new method
- Improve error handling in render_glyph method by using `?` operator for character conversion
- Refactor add_font_with_name method for improved clarity and error handling
- Improve error handling in render_glyphs method and update writer mutex usage
- Update dependencies to latest versions in Cargo.toml and Cargo.lock

### 🧪 Testing
- Add unit tests for FontWrapper, output directory preparation, and DummyWriter functionality
- Add end-to-end tests for debug::run functionality and error handling

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.8.0...v0.9.0

## v0.8.0 - 2026-02-22

### 🐛 Bug Fixes
- Update expected file count in tests for render_glyphs method
- Handle potential error in Writer drop implementation
- Move unused_imports allowance to the correct location in mod.rs
- Improve error handling for missing cmap table in FontMetadata
- Remove unused PlatformId import from metadata.rs
- Update get_metadata method in FontWrapper to return Result for error handling
- Optimize get_blocks method by removing unnecessary block initialization
- Reorder fields in FontFileEntry for clarity
- Simplify checksum field initialization in TarWriter
- Update badge formatting in README.md for consistency
- Add missing permissions for id-token and pages in workflows

### 🧹 Chores
- Update dependencies to latest versions

### 📦 Other
- Add funding.yml

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.7.2...v0.8.0

## v0.7.2 - 2025-08-15

### 🐛 Bug Fixes
- Clippy warnings
- Add permissions for write access in CI workflows

### 🧹 Chores
- Update dependencies to latest versions

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.7.1...v0.7.2

## v0.7.1 - 2025-08-01

### 🐛 Bug Fixes
- Clippy warnings
- Correctly set tar file mtime

### 🧹 Chores
- Update dependencies to latest versions

### 📦 Other
- Move sudo

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.7.0...v0.7.1

## v0.7.0 - 2025-03-11

### 🚀 Features
- Translate glyphs to compensate the rounding error of `advance`
- Add debug mode, close #6

### 🐛 Bug Fixes
- Clippy warnings und update test results
- Generate all glyph blocks, close #5

### 🚜 Refactoring
- Rename Glyph to PbfGlyph

### 📚 Documentation
- Update README
- Add hash

### 🧪 Testing
- Fix doc tests
- Update to reflect 256 glyph blocks

### ⚙️ Build & CI
- Upgrade dependencies
- Test doc

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.6.0...v0.7.0

## v0.6.0 - 2025-03-09

### 🚀 Features
- Allow to run single threaded
- Return codeblocks in font_families.json

### 🚜 Refactoring
- Writer
- Use crate directly
- Renderer

### 📚 Documentation
- Update
- Fix doc
- Missing functions

### 🧪 Testing
- Merge and recurse

### ⚙️ Build & CI
- Upgrade dependencies
- Add color
- Profile script

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.5.0...v0.6.0

## v0.5.0 - 2025-03-08

### 🚀 Features
- Allow one or more files or directories
- Increase bezier precision

### 🐛 Bug Fixes
- Arg help

### 🚜 Refactoring
- Rename to result.rs
- Rename module path
- Rename and move modules
- Remove unused code
- Use abstract renderer
- Abstract renderer
- Minor stuff
- Minor
- Many structs
- Font modules
- Font modules

### 📚 Documentation
- Update readme
- Utils/*
- Writer/*
- Render/*
- Protobuf/*
- Geometry/*
- Minor fixes
- Commands/* + font/*

### 🧪 Testing
- Output_directory
- File
- Scan
- Improve tests
- Font::manager
- Wrapper
- Index_file
- Protobuf

### ⚙️ Build & CI
- Fix selected features
- Upgrade graph
- Analyse bloat
- Improve check script
- Generate graph
- Upgrade some dependencies

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.4.0...v0.5.0

## v0.4.0 - 2025-03-04

### 🚀 Features
- Generate name from metadata
- Add index.json and font_family.json, close #2

### 🚜 Refactoring
- Add abstract writer

### 📚 Documentation
- Correct font name
- Add links

### 🧪 Testing
- Add second line

### 🎨 Styling
- Rename modules
- Rename

### ⚙️ Build & CI
- Upgrade dependencies
- Rename steps
- Trigger pages as reusable workflow

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.3.0...v0.4.0

## v0.3.0 - 2025-03-03

### 🚀 Features
- Stream output as tar #1
- Add font_manager
- Implement recurse

### 🐛 Bug Fixes
- Clippy warnings
- Font folder names

### 🚜 Refactoring
- Simplify sdf position
- Use standard progress bar
- Name_to_id
- Even more
- Reorganize code

### 📚 Documentation
- Vertical align: middle

### 🎨 Styling
- Cargo fmt

### ⚙️ Build & CI
- Generate pages only during release

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.2.1...v0.3.0

## v0.2.1 - 2025-03-01

### 📚 Documentation
- Update license and readme
- Update readme

### ⚙️ Build & CI
- Trigger release workflow via script

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.2.0...v0.2.1

## v0.2.0 - 2025-03-01

### 🚀 Features
- Speed up by parallelising glyph generation
- Add install script

### 🐛 Bug Fixes
- Url
- Glyph positions
- Use correct codepoint
- Space character
- Glyph height
- Progress bar
- Pbf

### 🚜 Refactoring
- Remove unused render_range
- Split code
- Remove unused code

### 📚 Documentation
- Add readme

### 🧪 Testing
- Fix tests
- Add more test data
- Render_sdf
- Rtree
- Font/metadata

### 🎨 Styling
- Add .prettierrc

### ⚙️ Build & CI
- Improve preview
- Upgrade maplibre
- Use correct path
- Build only pages on main branch
- Build and deploy pages
- Add font web demo
- Create releases as drafts

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.1.1...v0.2.0

## v0.1.1 - 2025-02-27

### 🐛 Bug Fixes
- Use correct tag
- Feature

### 🚜 Refactoring
- Reorganize libraries

### 📚 Documentation
- Fix badges

### ⚙️ Build & CI
- Update release when finished
- Use scripts/get_version.sh
- Release on mac
- Try cross
- Extract version
- Rewrite release workflow
- Disable github checks by codecov
- Remove unused code and flags
- Remove protobuf-compile
- Add protobuf-compiler
- Add cli feature
- Only release necessary files
- Improve release script

### 📦 Other
- Ignore .DS_Store

**Full Changelog**: https://github.com/versatiles-org/versatiles-glyphs-rs/compare/v0.1.0...v0.1.1

## v0.1.0 - 2025-02-27

### 🚀 Features
- Combine multiple font sources
- Add command "convert"
- Implement everything

### 🐛 Bug Fixes
- Clippy warnings

### 🚜 Refactoring
- Rename project
- Move scripts
- Reorganize code
- Remove unused code

### 📚 Documentation
- Add LICENSE
- Add readme

### 🧪 Testing
- Fix
- Scale
- Ring_builder
- Rings
- Ring
- Segment
- Points
- Bbox

### 🎨 Styling
- Cargo fmt
- Cargo fmt
- Fmt

### ⚙️ Build & CI
- Minor fixes before releasing
- Add release workflow
- Add release script
- Add script to upgrade dependencies
- Check unused dependencies
- Add check
- Add test script
- Generate docs
- Please don't make this sandwich joke
- Install protobuf-compiler
- Add workflows

### 📦 Other
- Use only unicode tables
- Scale glyphs
- Initial commit


