//! Async hardware abstraction traits for the HDMI stack.
//!
//! `hdmi-hal-async` is the async companion to `hdmi-hal`, following the same split as
//! `embedded-hal` / `embedded-hal-async`. It mirrors each trait in `hdmi-hal` with
//! `async fn` methods and otherwise identical signatures. It is a traits-only crate:
//! no implementations live here.

#![no_std]
#![forbid(unsafe_code)]

/// PHY lane configuration traits and associated types.
pub mod phy;

/// SCDC register transport trait.
pub mod scdc;
