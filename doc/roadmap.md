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

Mirrored from `hdmi-hal`: each item lands there first and here in the same release, with
the same shape. See `hdmi-hal`'s
[`doc/roadmap.md`](https://github.com/DracoWhitefire/hdmi-hal/blob/main/doc/roadmap.md)
for the details.

- **Further PHY configuration** — lane mapping, TMDS pre-emphasis and swing,
  spread-spectrum clocking and power states, each decided when the layer that needs it is
  built. As trait methods with default implementations they are not breaking changes; as
  required methods they would be.
- **CEC line trait** — a single-wire CEC bus access primitive, when CEC work begins.
