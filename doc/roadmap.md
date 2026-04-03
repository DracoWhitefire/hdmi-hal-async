# Roadmap

## Released

### 0.1.0

- `scdc::ScdcTransport` — async mirror of `hdmi_hal::scdc::ScdcTransport`
- `phy::HdmiPhy` — async mirror of `hdmi_hal::phy::HdmiPhy`
- Re-exports of `LtpPattern`, `EqParams`, and `LaneEqParams` at the crate root
- `#![no_std]`, `#![forbid(unsafe_code)]`, `#![allow(async_fn_in_trait)]`
- Full rustdoc coverage

### 0.0.1

- Name claimed in advance of implementation.

## Planned

### culvert-async

An async companion to `culvert` that wraps `hdmi_hal_async::scdc::ScdcTransport` and
provides typed SCDC register wrappers with `async fn` methods. Mirrors `culvert`'s API
with `async fn` replacing `fn` throughout.

### plumbob-async

An async companion to `plumbob` that implements the four-phase FRL link training state
machine as a cooperative async state machine. Uses `hdmi_hal_async::phy::HdmiPhy` and its
own async `ScdcClient` trait (parallel to `plumbob::ScdcClient`). The polling loops in
`plumbob`'s phases become cooperative `await` points rather than spin loops with a
bounded iteration counter.
