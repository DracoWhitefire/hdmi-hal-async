# hdmi-hal-async

[![crates.io](https://img.shields.io/crates/v/hdmi-hal-async.svg)](https://crates.io/crates/hdmi-hal-async)
[![License: MPL-2.0](https://img.shields.io/badge/license-MPL--2.0-blue.svg)](LICENSE)

**This crate is a planned stub.** No traits are implemented yet. The name is claimed
in advance of the implementation.

---

`hdmi-hal-async` will be the async companion to [`hdmi-hal`](https://crates.io/crates/hdmi-hal),
following the same split as `embedded-hal` / `embedded-hal-async`.

It will mirror each trait in `hdmi-hal` with `async fn` methods and otherwise identical
signatures — `ScdcTransport` and `HdmiPhy` for async platform backends (Embassy, async
kernel drivers). Data types such as `EqParams` are defined in `hdmi-hal` and shared;
this crate re-exports them rather than duplicating them.

Implementors targeting async environments implement the traits here. Sync consumers
continue to use `hdmi-hal` directly. The two crates impose no executor dependency on
each other.

## Status

Implementation will begin when an async consumer — an Embassy platform backend or an
async link training crate — is being built. Until then this crate exists to hold the
name.

## License

Licensed under the [Mozilla Public License 2.0](LICENSE).
