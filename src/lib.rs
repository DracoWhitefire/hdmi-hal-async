//! Async hardware abstraction traits for the HDMI stack.
//!
//! `hdmi-hal-async` is the async companion to `hdmi-hal`, following the same split as
//! `embedded-hal` / `embedded-hal-async`. It mirrors each trait in `hdmi-hal` with
//! `async fn` methods and otherwise identical signatures. It is a traits-only crate:
//! no implementations live here.

#![no_std]
#![forbid(unsafe_code)]
// async fn in traits does not guarantee Send on the returned futures. This is intentional:
// the crate targets single-threaded embedded executors (e.g. Embassy) where Send is not
// required. This follows the same decision made by embedded-hal-async.
#![allow(async_fn_in_trait)]

/// PHY lane configuration traits and associated types.
pub mod phy;

/// SCDC register transport trait.
pub mod scdc;

pub use hdmi_hal::phy::{EqParams, LaneEqParams, LtpPattern};
