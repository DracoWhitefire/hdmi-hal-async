# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking changes

- **`LtpPattern` is now a `#[non_exhaustive]` enum with the HDMI 2.1 pattern values**,
  following `hdmi-hal`. The re-exported type previously documented 1–4 as LFSR 0–3, which
  was wrong: the values are 1 = all ones, 2 = all zeros, 3 = Nyquist clock, 4 = DDE
  compliance and 5–8 = LFSR 0–3. `LtpPattern::new(u8)` is removed; use the variants
  (`LtpPattern::Lfsr0` etc.).
- **`HdmiPhy::send_ltp` takes the full per-lane pattern set**, mirroring `hdmi-hal`: the
  signature is now `async fn send_ltp(&mut self, patterns: LanePatterns)`. `None` on a
  lane means no training pattern; `None` on every lane stops the training patterns.
- **`HdmiPhy::set_frl_output` is a new required method**, mirroring `hdmi-hal`: existing
  implementations must add
  `async fn set_frl_output(&mut self, output: FrlOutput) -> Result<(), Self::Error>`. It
  selects gap-only output during link training and active output once training has
  succeeded.
- **`ScdcTransport::read` now takes `&self` instead of `&mut self`.** Mirrors the
  same change in `hdmi-hal`. Implementations must change `async fn read(&mut self, reg: u8)`
  to `async fn read(&self, reg: u8)` and introduce interior mutability for any state
  mutated during reads.

### Added

- `LanePatterns` re-exported from `hdmi-hal` at the crate root.
- `FrlOutput` re-exported from `hdmi-hal` at the crate root.
- `TxFfeLevel` re-exported from `hdmi-hal` at the crate root. `LaneEqParams` (from
  `hdmi-hal`) now carries the lane's TxFFE level as `tx_ffe_level`, applied through
  `HdmiPhy::adjust_equalization`.
- `ScdcTransport::read_block(reg, buf)`, mirroring `hdmi-hal` — reads consecutive
  registers into `buf`. The default implementation calls `read` once per byte, so existing
  transports need no changes; transports that can burst-read should override it.
- **SLSA Build Level 2 provenance** — release artifacts are attested via
  `actions/attest-build-provenance` and verified with
  `gh attestation verify <file> --repo DracoWhitefire/hdmi-hal-async`.

### Changed

- `HdmiPhy`'s documentation now describes it as covering the transmitter's link-level
  FRL and TMDS behaviour as well as analog lane configuration, matching `hdmi-hal`.
- **`hdmi-hal` updated to 0.5** — the release with the per-lane `send_ltp`,
  `set_frl_output` and the HDMI 2.1 `LtpPattern` values mirrored above. It uses
  display-types 0.4, like this crate.
- **`display-types` updated to 0.4** — tracks DisplayID 2.x support added in `piaf` 0.4.1.

### Internal

- **CI builds for a `no_std` target** — the `Build (no_std)` step now builds for
  `thumbv7em-none-eabi`. It previously built for the host, where `std` is always
  available, so it could not catch a dependency that enables `std` (as hdmi-hal did
  through `display-types`).
  The publish workflow runs the same build steps.

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
