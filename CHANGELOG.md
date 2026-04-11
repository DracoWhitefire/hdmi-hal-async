# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **SLSA Build Level 2 provenance** — release artifacts are attested via
  `actions/attest-build-provenance` and verified with
  `gh attestation verify <file> --repo DracoWhitefire/hdmi-hal-async`.

## [0.1.0] - 2026-04-03

### Added

- `scdc::ScdcTransport` — async mirror of `hdmi_hal::scdc::ScdcTransport`. Methods:
  `async fn read(reg: u8) -> Result<u8, Self::Error>` and
  `async fn write(reg: u8, value: u8) -> Result<(), Self::Error>`.
- `phy::HdmiPhy` — async mirror of `hdmi_hal::phy::HdmiPhy`. Methods:
  `async fn set_frl_rate`, `async fn send_ltp`, `async fn adjust_equalization`,
  `async fn set_scrambling`.
- Re-exports of `LtpPattern`, `EqParams`, and `LaneEqParams` at the crate root,
  mirroring the position of those types in `hdmi-hal`.

**Project infrastructure**

- `#![no_std]`, `#![forbid(unsafe_code)]`
- `#![allow(async_fn_in_trait)]` — targets single-threaded embedded executors (e.g.
  Embassy) where `Send` bounds on futures are not required, following the same
  decision made by `embedded-hal-async`.
- Full rustdoc coverage enforced via `cargo rustdoc -- -D missing_docs`.
- Unit tests with mock implementors in each trait module, using `pollster` as the
  test executor.

## [0.0.1] - 2026-04-03

- Name claimed in advance of implementation. No traits are implemented yet.
  See [`doc/roadmap.md`](doc/roadmap.md) for the planned 0.1.0 scope.
