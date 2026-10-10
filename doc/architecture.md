# hdmi-hal-async — Architecture and Implementation Plan

## Overview

`hdmi-hal-async` is the async companion to `hdmi-hal`, following the same design split
as `embedded-hal` / `embedded-hal-async`. It defines async versions of the two traits
in `hdmi-hal`:

- `scdc::ScdcTransport` — raw byte-level SCDC register access
- `phy::HdmiPhy` — transmitter PHY control: FRL rate, link training patterns, FRL output
  mode, scrambling and equalization

Data types (`LtpPattern`, `LanePatterns`, `FrlOutput`, `EqParams`, `LaneEqParams`,
`TxFfeLevel`) are defined in `hdmi-hal` and re-exported here; they are not duplicated.

The crate is traits-only. No implementations live here.

---

## Language feature: async fn in traits

MSRV is `1.85`, which is well above the `1.75` stabilisation of `async fn` in traits
(AFIT). The `async-trait` proc-macro crate is not needed.

AFIT-generated futures are not object-safe, which is the same trade-off `embedded-hal-async`
accepts. Trait objects are not a goal for this HAL layer.

Nor do the traits promise that their futures are `Send`. With concrete types the compiler
sees the real futures, so a training future over `Send` implementations is `Send` and can
be spawned on a multi-threaded executor. Code generic over `HdmiPhy` or `ScdcTransport`
cannot spawn one, however the types are bounded; it runs on a single-threaded executor
(Embassy, Tokio's current-thread runtime or `LocalSet`) or uses concrete types. Because
`ScdcTransport::read` takes `&self`, a future that reads also needs the transport to be
`Sync` to be `Send`. `Send` variants of the traits are designed and proven in
the stack design document ("Sync and Async Companions", "`Send` futures for multi-threaded executors"), and deferred until a consumer needs them.

---

## Module structure

The module tree mirrors `hdmi-hal` exactly:

```
src/
  lib.rs          — #![no_std], module declarations, crate-level docs
  scdc/
    mod.rs        — async ScdcTransport trait
  phy/
    mod.rs        — async HdmiPhy trait
```

---

## Traits

### `scdc::ScdcTransport`

Async version of `hdmi_hal::scdc::ScdcTransport`. Identical semantics; only the
`fn` keywords become `async fn`.

```rust
pub trait ScdcTransport {
    type Error;

    async fn read(&self, reg: u8) -> Result<u8, Self::Error>;
    async fn write(&mut self, reg: u8, value: u8) -> Result<(), Self::Error>;

    /// Default: one `read` per byte. Override for a single burst transaction.
    async fn read_block(&self, reg: u8, buf: &mut [u8]) -> Result<(), Self::Error> { ... }
}
```

Doc comments should mirror the sync version verbatim, with a note that the trait is the
async counterpart.

### `phy::HdmiPhy`

Async version of `hdmi_hal::phy::HdmiPhy`. Same methods, same parameter types,
same semantics.

```rust
pub trait HdmiPhy {
    type Error;

    async fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error>;
    async fn send_ltp(&mut self, patterns: LanePatterns) -> Result<(), Self::Error>;
    async fn set_frl_output(&mut self, output: FrlOutput) -> Result<(), Self::Error>;
    async fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error>;
    async fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error>;
}
```

`HdmiForumFrl` comes from `display-types` via re-export in `hdmi-hal`.
`LanePatterns`, `LtpPattern`, `FrlOutput` and `EqParams` are re-exported from `hdmi-hal`
(see below).

---

## Re-exports

Data types that live in `hdmi-hal` and are used in trait signatures must be re-exported
so callers have a single import path. The following should appear in `lib.rs`:

```rust
pub use hdmi_hal::phy::{EqParams, FrlOutput, LaneEqParams, LanePatterns, LtpPattern, TxFfeLevel};
```

`HdmiForumFrl` is already accessible through `display-types`; if it appears as a trait
bound consumers will import it from there or from `hdmi-hal` directly. It does not need
a re-export here.

---

## Cargo.toml

`hdmi-hal` (for the shared data types) and `display-types` (for `HdmiForumFrl` in the
`HdmiPhy` signature) are the only dependencies. No other dependencies are required. The
crate must remain `#![no_std]`: dependencies are declared with `default-features = false`,
and CI builds the crate for `thumbv7em-none-eabi`, a target without `std`.

---

## What this crate does not do

- **No implementations.** Platform backends (Embassy drivers, async kernel adaptors)
  implement the traits here; implementations do not live in this crate.
- **No sync-to-async adapters.** A wrapper that makes a sync `ScdcTransport` implement
  the async trait (by blocking) would belong in a compatibility shim crate, not here.
  The same split exists between `embedded-hal` and `embedded-hal-async`.
- **No executor dependency.** The traits impose no constraint on which async runtime is
  used; that is entirely the implementor's concern.
- **No plumbob or culvert dependency.** Those are higher-level protocol crates. The
  dependency only flows downward: protocol crates depend on HAL crates, not the reverse.

---

## Testing

Unit tests for a traits-only crate are minimal but still useful:

- A compile-time test that a concrete mock type implementing each trait compiles correctly
  verifies that the trait signatures are well-formed and that `async fn` in traits works
  end-to-end.
- Each trait module should have a `#[cfg(test)]` block with a minimal mock implementor
  (mirroring the pattern in `hdmi-hal`) and one or two smoke-test cases.

Tests must compile and pass in `no_std` with a `#[tokio::test]` or similar executor
appropriate for the test harness. Because the crate itself is `no_std`, tests can use
`tokio` or `embassy-executor` in dev-dependencies without pulling them into the library.

---

## Release integrity

Every release is published through a GitHub Actions workflow that signs the `.crate`
package with [SLSA Build Level 2](https://slsa.dev) provenance. Verify with:

```sh
gh attestation verify hdmi-hal-async-X.Y.Z.crate --repo DracoWhitefire/hdmi-hal-async
```

---

## Future consumers

Once `hdmi-hal-async` is published, the expected downstream crates are:

| Crate | Async analogue of | Uses |
|---|---|---|
| `culvert-async` | `culvert` | `hdmi_hal_async::scdc::ScdcTransport` |
| `plumbob-async` | `plumbob` | `hdmi_hal_async::scdc::ScdcTransport` (via its own `ScdcClient`) and `hdmi_hal_async::phy::HdmiPhy` |

The `plumbob-async` state machine will own the largest design challenge: converting the
four-phase polling loop into a cooperative async state machine that yields between poll
iterations instead of spinning.
