//! Async hardware abstraction traits for the HDMI stack.
//!
//! `hdmi-hal-async` is the async companion to `hdmi-hal`, following the same split as
//! `embedded-hal` / `embedded-hal-async`. It mirrors each trait in `hdmi-hal` with
//! `async fn` methods and otherwise identical signatures. It is a traits-only crate:
//! no implementations live here.
//!
//! Each trait also has a `Send` twin, [`phy::SendHdmiPhy`] and [`scdc::SendScdcTransport`],
//! for code that spawns link training on a multi-threaded executor: the same methods,
//! returning futures that are `Send`. Every twin implementation is an implementation of the
//! plain trait as well.

#![no_std]
#![forbid(unsafe_code)]
// async fn in traits does not guarantee Send on the returned futures. This is intentional
// for the plain traits: they serve single-threaded embedded executors (e.g. Embassy), where
// Send is not required, the same decision embedded-hal-async made. The Send twins
// (SendHdmiPhy, SendScdcTransport) are for multi-threaded executors.
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
