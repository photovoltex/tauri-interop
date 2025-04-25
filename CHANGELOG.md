# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) since v0.2.0.

## [Unreleased] - YYYY-MM-DD

### Changed

- Update the `README` to provide short samples at the very beginning, so that the usage is right away visible

### Added

- Added `ManagedEmit` as derive macro (feature: `event` and `initial_value` required) for easier implementation

### Removed

- Removed the mandatory inclusion of third party crates by re-exporting them (breaking)
- Excluded `/.github`, `publish.sh` and `CHANGELOG.md` from the published crate

## [2.2.1](https://github.com/photovoltex/tauri-interop/releases/tag/v2.2.1) - 2025-04-23

### Fixed

- fixed the compilation errors on [`docs.rs`](https://docs.rs) by adding `docsrs` as `cfg` flag to the compiler

## [2.2.0](https://github.com/photovoltex/tauri-interop/releases/tag/v2.2.0) - 2025-04-23

### Changed

- Updated `tauri` to v2 (breaking)
- Updated `leptos` to 0.7 (breaking)
- Update the CI workflow, so that some files don't trigger a pipe run anymore
- Update the `README` to provide a "How to get started" section 

### Added

- Added `publish.sh` to simplify the local publishing process

### Removed

- Removed all `Cargo.lock` files to allow users to always use the latest available version
