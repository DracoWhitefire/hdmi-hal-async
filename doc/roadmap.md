# Roadmap

What each release contains is in the [changelog](../CHANGELOG.md); this page is about
what comes next.

## Planned

Mirrored from `hdmi-hal`: each item lands there first and here in the same release, with
the same shape. See `hdmi-hal`'s
[`doc/roadmap.md`](https://github.com/ashlar-display/hdmi-hal/blob/main/doc/roadmap.md)
for the details.

- **PHY traits by link mode (breaking)** — `HdmiPhy` and `SendHdmiPhy` renamed `FrlPhy` and
  `SendFrlPhy`, with no alias; a new `TmdsPhy` with its `Send` twin `SendTmdsPhy`; the power
  trait; the clock and delay traits, where the async delay gets a hand-written `Send` twin
  as the other traits have (generic code bounded by embedded-hal-async's `DelayNs` cannot
  produce a `Send` future, and that crate has no `Send` twin); `DdcBus` as an adapter
  type; a cancellation clause for the PHY traits' futures, matching `ScdcTransport`'s.
- **Further PHY configuration** — lane mapping, TMDS pre-emphasis and swing,
  spread-spectrum clocking and power states beyond off and on, each decided when the layer
  that needs it is built. As trait methods with default implementations they are not
  breaking changes; as required methods they would be.
- **CEC traits** — message-level and bit-level, when CEC work begins.
