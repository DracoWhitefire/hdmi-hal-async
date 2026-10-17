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

/// PHY control trait.
pub mod phy;

/// SCDC register transport trait.
pub mod scdc;

pub use hdmi_hal::phy::{
    EqParams, FrlOutput, LaneEqParams, LanePatterns, LtpPattern, TxFfeLevel, TxFfeRestrictions,
};

/// The FRL rate, as `HdmiPhy::set_frl_rate` takes it; re-exported so that implementations
/// name the same type this crate does.
pub use hdmi_hal::HdmiForumFrl;
