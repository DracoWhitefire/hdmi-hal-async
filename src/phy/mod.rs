use display_types::cea861::hdmi_forum::HdmiForumFrl;
/// The PHY value types, at the paths `hdmi-hal` gives them (`hdmi_hal::phy::*`). They are
/// re-exported at the crate root as well.
pub use hdmi_hal::phy::{
    EqParams, FrlOutput, LaneEqParams, LanePatterns, LtpPattern, TmdsClockRatio,
    TmdsPhyCapabilities, TmdsPhyConfig, TmdsVideo, TxFfeLevel, TxFfeRestrictions,
};

/// Async FRL control of an HDMI 2.1 transmitter PHY.
///
/// This is the async counterpart to `hdmi_hal::phy::FrlPhy`. The trait contract is
/// identical; only the method signatures gain `async`.
///
/// Covers what FRL link training needs from the transmitter: FRL rate selection, link
/// training patterns, FRL output mode, and each lane's TxFFE level (FRL's transmitter
/// equalization, a combination of pre-shoot and de-emphasis) through
/// [`adjust_equalization`](Self::adjust_equalization). TMDS is [`TmdsPhy`]'s, the source's
/// scrambler included. Further PHY configuration (lane mapping, spread-spectrum clocking and
/// power states) is not covered yet; it is planned for when the mode-setting layer needs it.
/// Vendor-specific register sequences are an implementation detail of each backend.
///
/// On a PHY that also implements [`TmdsPhy`], entering one mode leaves the other:
/// [`set_frl_rate`](Self::set_frl_rate) with a rate leaves TMDS.
///
/// # Cancellation
///
/// Any of these futures may be dropped before it completes, for example by a deadline in
/// the caller. What a dropped call has done is then unknown: the PHY may or may not have
/// applied the setting, or may be part-way through it.
///
/// Each method sets a whole state (the rate, the per-lane patterns, the output mode, the
/// equalization), so calling it again with the state wanted recovers from a dropped call.
/// An implementation for which that does not hold must say so in its documentation: what a
/// cancelled call leaves behind, and how to recover.
///
/// A caller that needs to stop early can avoid the question: check its deadline or cancel
/// signal before each call and stop there, rather than dropping a future mid-flight.
pub trait FrlPhy {
    /// Error type returned by PHY operations.
    type Error;

    /// Select the FRL rate, or turn FRL off. Triggers the required lane reconfiguration
    /// sequence.
    ///
    /// A rate enters FRL, leaving TMDS on a PHY that also does TMDS.
    /// [`HdmiForumFrl::NotSupported`] turns FRL off and enters no other mode: entering TMDS
    /// is [`TmdsPhy::configure`]'s.
    ///
    /// Returns once the PHY transmits at `rate`, after any bring-up its hardware needs to
    /// get there — the Xilinx HDMI 2.1 transmitter, for example, holds a Nyquist clock
    /// pattern until its link is up. Only the PHY knows when that is, so the sequence
    /// belongs here; the link training layer sends no pattern of its own around it.
    async fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error>;

    /// Drive the given link training patterns on the physical lanes, one per lane.
    async fn send_ltp(&mut self, patterns: LanePatterns) -> Result<(), Self::Error>;

    /// Select what the transmitter sends on the FRL lanes: gap characters only, or video,
    /// data islands and control periods.
    async fn set_frl_output(&mut self, output: FrlOutput) -> Result<(), Self::Error>;

    /// Adjust equalization parameters after link training feedback.
    async fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error>;
}

/// [`FrlPhy`] with `Send` futures, for code that spawns link training on a multi-threaded
/// executor.
///
/// `FrlPhy`'s `async fn`s promise nothing about their futures, so code generic over
/// `P: FrlPhy` cannot spawn a future that calls them, however `P` is bounded. This trait
/// has the same methods and contract, cancellation included, each returning a future that
/// is `Send`, and errors
/// that are `Send` too: a spawned task returns them. Implementations still write
/// `async fn`; the compiler checks that each future is `Send`.
///
/// Every `SendFrlPhy` is an `FrlPhy`, through a blanket implementation, so it works
/// with everything written against `FrlPhy` as well; a type implements one of the two
/// (implementing both conflicts with the blanket implementation). A `SendFrlPhy` type
/// has both traits' methods, so with both traits in scope a method call on it is
/// ambiguous: import only the trait the code is bounded by, or call
/// `SendFrlPhy::set_frl_rate(…)`.
///
/// Bound generic code by `SendFrlPhy` alone. Adding `+ FrlPhy`, which it implies
/// anyway, makes the compiler use that bound instead of the blanket implementation, and
/// the futures are no longer known to be `Send`.
pub trait SendFrlPhy: Send {
    /// Error type returned by PHY operations.
    type Error: Send;

    /// See [`FrlPhy::set_frl_rate`].
    fn set_frl_rate(
        &mut self,
        rate: HdmiForumFrl,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// See [`FrlPhy::send_ltp`].
    fn send_ltp(
        &mut self,
        patterns: LanePatterns,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// See [`FrlPhy::set_frl_output`].
    fn set_frl_output(
        &mut self,
        output: FrlOutput,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// See [`FrlPhy::adjust_equalization`].
    fn adjust_equalization(
        &mut self,
        params: EqParams,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<T: SendFrlPhy> FrlPhy for T {
    type Error = <T as SendFrlPhy>::Error;

    fn set_frl_rate(
        &mut self,
        rate: HdmiForumFrl,
    ) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendFrlPhy>::set_frl_rate(self, rate)
    }

    fn send_ltp(
        &mut self,
        patterns: LanePatterns,
    ) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendFrlPhy>::send_ltp(self, patterns)
    }

    fn set_frl_output(
        &mut self,
        output: FrlOutput,
    ) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendFrlPhy>::set_frl_output(self, output)
    }

    fn adjust_equalization(
        &mut self,
        params: EqParams,
    ) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendFrlPhy>::adjust_equalization(self, params)
    }
}

/// Async TMDS control of an HDMI transmitter PHY.
///
/// This is the async counterpart to `hdmi_hal::phy::TmdsPhy`. The trait contract is
/// identical; only the method signatures gain `async`, except
/// [`capabilities`](Self::capabilities), a plain report.
///
/// What TMDS bring-up and tear-down need from the transmitter, and nothing more: the
/// character rate and clock ratio, the source's scrambler, and starting and stopping the
/// output. A trait of its own, so TMDS-only hardware implements nothing of FRL.
///
/// Not covered: electrical settings (swing, pre-emphasis, lane mapping), which are the
/// implementation's; the video pipeline beyond what [`TmdsVideo`] gives the PHY; hotplug and
/// RX sense; and power.
///
/// On a PHY that also implements [`FrlPhy`], entering one mode leaves the other:
/// [`configure`](Self::configure) leaves FRL.
///
/// # Cancellation
///
/// Any of these futures may be dropped before it completes, for example by a deadline in
/// the caller. What a dropped call has done is then unknown: after a dropped
/// [`configure`](Self::configure) the rate and ratio are unspecified, after a dropped
/// [`set_scrambling`](Self::set_scrambling) the scrambler state, and after a dropped
/// [`start_output`](Self::start_output) or [`stop_output`](Self::stop_output) whether the
/// output runs.
///
/// [`stop_output`](Self::stop_output) and then [`configure`](Self::configure) again recover
/// from any of them. An implementation that cannot must say so in its documentation: what a
/// cancelled call leaves behind, and how to recover.
///
/// A caller that needs to stop early can avoid the question: check its deadline or cancel
/// signal before each call and stop there, rather than dropping a future mid-flight.
pub trait TmdsPhy {
    /// Error type returned by PHY operations.
    type Error: core::fmt::Debug;

    /// What the PHY can do. A plain report of what the implementation already knows: not
    /// `async`, not fallible.
    fn capabilities(&self) -> TmdsPhyCapabilities;

    /// Set the character rate and clock ratio, with the output stopped, and leave it
    /// stopped. Enters TMDS mode, leaving FRL on a PHY that also does FRL.
    ///
    /// An implementation that cannot reconfigure while transmitting may return an error
    /// when called with the output running.
    async fn configure(&mut self, config: TmdsPhyConfig) -> Result<(), Self::Error>;

    /// Turn the source's TMDS scrambler on or off, with the output running or stopped.
    ///
    /// `set_scrambling(false)` succeeds on a PHY without a scrambler
    /// ([`TmdsPhyCapabilities::scrambling`] false), since plain links call it.
    async fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error>;

    /// Start TMDS output at the configured rate.
    ///
    /// Completes once the PHY transmits, after whatever its hardware needs to get there (a
    /// PLL lock, a reset sequence): callers time from its completion, so completing means on
    /// the wire.
    async fn start_output(&mut self) -> Result<(), Self::Error>;

    /// Stop whatever the PHY transmits: TMDS clock and data or, on a PHY still in FRL, FRL
    /// transmission. Keeps the PHY powered and configured.
    ///
    /// Callable in any state, powered off included; stopping a stopped output is not an
    /// error.
    async fn stop_output(&mut self) -> Result<(), Self::Error>;
}

/// [`TmdsPhy`] with `Send` futures, for code that spawns TMDS bring-up on a multi-threaded
/// executor.
///
/// The same methods and contract, cancellation included, each returning a future that is
/// `Send`, and errors that are `Send` too. Every `SendTmdsPhy` is a `TmdsPhy`, through a
/// blanket implementation; a type implements one of the two. As with [`SendFrlPhy`], bound
/// generic code by `SendTmdsPhy` alone, and with both traits in scope call
/// `SendTmdsPhy::configure(…)` to avoid an ambiguous method call.
pub trait SendTmdsPhy: Send {
    /// Error type returned by PHY operations.
    type Error: core::fmt::Debug + Send;

    /// See [`TmdsPhy::capabilities`].
    fn capabilities(&self) -> TmdsPhyCapabilities;

    /// See [`TmdsPhy::configure`].
    fn configure(
        &mut self,
        config: TmdsPhyConfig,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// See [`TmdsPhy::set_scrambling`].
    fn set_scrambling(
        &mut self,
        enabled: bool,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// See [`TmdsPhy::start_output`].
    fn start_output(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// See [`TmdsPhy::stop_output`].
    fn stop_output(&mut self) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

impl<T: SendTmdsPhy> TmdsPhy for T {
    type Error = <T as SendTmdsPhy>::Error;

    fn capabilities(&self) -> TmdsPhyCapabilities {
        <T as SendTmdsPhy>::capabilities(self)
    }

    fn configure(
        &mut self,
        config: TmdsPhyConfig,
    ) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendTmdsPhy>::configure(self, config)
    }

    fn set_scrambling(&mut self, enabled: bool) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendTmdsPhy>::set_scrambling(self, enabled)
    }

    fn start_output(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendTmdsPhy>::start_output(self)
    }

    fn stop_output(&mut self) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendTmdsPhy>::stop_output(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal `FrlPhy` that records what it is given. The tests that drive it method by
    /// method check little beyond recording — they show that the trait can be implemented
    /// and used for each method, and they keep this implementation covered. Behaviour is
    /// tested where it lives: in plumbob, against its simulated PHY.
    struct MockPhy {
        frl_rate: Option<HdmiForumFrl>,
        eq_calls: u32,
        last_eq: Option<EqParams>,
        frl_output: Option<FrlOutput>,
        last_ltp: Option<LanePatterns>,
    }

    impl MockPhy {
        fn new() -> Self {
            Self {
                frl_rate: None,
                eq_calls: 0,
                last_eq: None,
                frl_output: None,
                last_ltp: None,
            }
        }
    }

    impl FrlPhy for MockPhy {
        type Error = core::convert::Infallible;

        async fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error> {
            self.frl_rate = Some(rate);
            Ok(())
        }

        async fn send_ltp(&mut self, patterns: LanePatterns) -> Result<(), Self::Error> {
            self.last_ltp = Some(patterns);
            Ok(())
        }

        async fn set_frl_output(&mut self, output: FrlOutput) -> Result<(), Self::Error> {
            self.frl_output = Some(output);
            Ok(())
        }

        async fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error> {
            self.eq_calls += 1;
            self.last_eq = Some(params);
            Ok(())
        }
    }

    #[pollster::test]
    async fn send_ltp_records_patterns() {
        let mut phy = MockPhy::new();
        let patterns = LanePatterns {
            lane0: Some(LtpPattern::Lfsr0),
            lane1: Some(LtpPattern::Lfsr1),
            lane2: Some(LtpPattern::Lfsr2),
            lane3: None,
        };
        phy.send_ltp(patterns).await.unwrap();
        assert_eq!(phy.last_ltp, Some(patterns));
    }

    #[pollster::test]
    async fn send_ltp_updates_on_each_call() {
        let mut phy = MockPhy::new();
        let first = LanePatterns {
            lane0: Some(LtpPattern::Lfsr0),
            ..LanePatterns::default()
        };
        let second = LanePatterns {
            lane0: Some(LtpPattern::NyquistClock),
            ..first
        };
        phy.send_ltp(first).await.unwrap();
        phy.send_ltp(second).await.unwrap();
        assert_eq!(phy.last_ltp, Some(second));
    }

    #[pollster::test]
    async fn set_frl_rate_records_rate() {
        let mut phy = MockPhy::new();
        phy.set_frl_rate(HdmiForumFrl::Rate6Gbps4Lanes)
            .await
            .unwrap();
        assert_eq!(phy.frl_rate, Some(HdmiForumFrl::Rate6Gbps4Lanes));
    }

    #[pollster::test]
    async fn set_frl_rate_not_supported_is_valid() {
        let mut phy = MockPhy::new();
        phy.set_frl_rate(HdmiForumFrl::NotSupported).await.unwrap();
        assert_eq!(phy.frl_rate, Some(HdmiForumFrl::NotSupported));
    }

    #[pollster::test]
    async fn set_frl_rate_can_be_updated() {
        let mut phy = MockPhy::new();
        phy.set_frl_rate(HdmiForumFrl::Rate3Gbps3Lanes)
            .await
            .unwrap();
        phy.set_frl_rate(HdmiForumFrl::Rate12Gbps4Lanes)
            .await
            .unwrap();
        assert_eq!(phy.frl_rate, Some(HdmiForumFrl::Rate12Gbps4Lanes));
    }

    #[pollster::test]
    async fn set_frl_output_records_mode() {
        let mut phy = MockPhy::new();
        phy.set_frl_output(FrlOutput::GapOnly).await.unwrap();
        assert_eq!(phy.frl_output, Some(FrlOutput::GapOnly));
    }

    #[pollster::test]
    async fn set_frl_output_can_be_switched() {
        let mut phy = MockPhy::new();
        phy.set_frl_output(FrlOutput::GapOnly).await.unwrap();
        phy.set_frl_output(FrlOutput::Active).await.unwrap();
        assert_eq!(phy.frl_output, Some(FrlOutput::Active));
    }

    #[pollster::test]
    async fn adjust_equalization_records_per_lane_levels() {
        let mut phy = MockPhy::new();
        let mut params = EqParams::new();
        params.lane1.tx_ffe_level = TxFfeLevel::MAX;
        phy.adjust_equalization(params).await.unwrap();
        let last = phy.last_eq.unwrap();
        assert_eq!(last, params);
        assert_eq!(last.lane0.tx_ffe_level.value(), 0);
        assert_eq!(last.lane1.tx_ffe_level.value(), 7);
    }

    #[pollster::test]
    async fn adjust_equalization_tracks_call_count() {
        let mut phy = MockPhy::new();
        phy.adjust_equalization(EqParams::new()).await.unwrap();
        phy.adjust_equalization(EqParams::new()).await.unwrap();
        assert_eq!(phy.eq_calls, 2);
    }

    /// Implements only the `Send` twin; it records the calls in order.
    #[derive(Default)]
    struct SendMockPhy {
        calls: [Option<&'static str>; 4],
        n: usize,
        last_eq: Option<EqParams>,
    }

    impl SendMockPhy {
        fn record(&mut self, call: &'static str) {
            self.calls[self.n] = Some(call);
            self.n += 1;
        }
    }

    impl SendFrlPhy for SendMockPhy {
        type Error = core::convert::Infallible;

        async fn set_frl_rate(&mut self, _rate: HdmiForumFrl) -> Result<(), Self::Error> {
            self.record("set_frl_rate");
            Ok(())
        }

        async fn send_ltp(&mut self, _patterns: LanePatterns) -> Result<(), Self::Error> {
            self.record("send_ltp");
            Ok(())
        }

        async fn set_frl_output(&mut self, _output: FrlOutput) -> Result<(), Self::Error> {
            self.record("set_frl_output");
            Ok(())
        }

        async fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error> {
            self.record("adjust_equalization");
            self.last_eq = Some(params);
            Ok(())
        }
    }

    /// The result of each method call, in call order.
    type Results<E> = [Result<(), E>; 4];

    /// Calls every method through `FrlPhy`, as code written against the plain trait does.
    async fn drive_plain<P: FrlPhy>(phy: &mut P, eq: EqParams) -> Results<P::Error> {
        [
            phy.set_frl_rate(HdmiForumFrl::Rate6Gbps4Lanes).await,
            phy.send_ltp(LanePatterns::default()).await,
            phy.set_frl_output(FrlOutput::GapOnly).await,
            phy.adjust_equalization(eq).await,
        ]
    }

    /// Calls every method through `SendFrlPhy`, as code bounded by the twin does.
    async fn drive_send<P: SendFrlPhy>(phy: &mut P) -> Results<P::Error> {
        [
            SendFrlPhy::set_frl_rate(phy, HdmiForumFrl::Rate12Gbps4Lanes).await,
            SendFrlPhy::send_ltp(phy, LanePatterns::default()).await,
            SendFrlPhy::set_frl_output(phy, FrlOutput::Active).await,
            SendFrlPhy::adjust_equalization(phy, EqParams::new()).await,
        ]
    }

    /// The compiler proves this `Send` bound for every `P: SendFrlPhy`, not only for the
    /// mock; with `FrlPhy` in its place it would not compile.
    fn spawnable<P: SendFrlPhy>(
        phy: &mut P,
    ) -> impl Future<Output = Results<P::Error>> + Send + '_ {
        drive_send(phy)
    }

    fn assert_send<T: Send>(value: T) -> T {
        value
    }

    const ALL_CALLS: [Option<&str>; 4] = [
        Some("set_frl_rate"),
        Some("send_ltp"),
        Some("set_frl_output"),
        Some("adjust_equalization"),
    ];

    #[pollster::test]
    async fn send_twin_is_usable_as_hdmi_phy() {
        let mut phy = SendMockPhy::default();
        let mut eq = EqParams::new();
        eq.lane2.tx_ffe_level = TxFfeLevel::MAX;
        assert_eq!(drive_plain(&mut phy, eq).await, [Ok(()); 4]);
        assert_eq!(phy.calls, ALL_CALLS);
        assert_eq!(phy.last_eq, Some(eq));
    }

    #[pollster::test]
    async fn send_twin_futures_are_send_in_generic_code() {
        let mut phy = SendMockPhy::default();
        assert_eq!(assert_send(spawnable(&mut phy)).await, [Ok(()); 4]);
        assert_eq!(phy.calls, ALL_CALLS);
    }

    #[test]
    fn value_types_are_hdmi_hals_at_both_paths() {
        // `phy::` mirrors `hdmi_hal::phy::`; the crate root has the same types.
        let pattern: crate::phy::LtpPattern = hdmi_hal::phy::LtpPattern::Lfsr0;
        let _: crate::LtpPattern = pattern;
        let output: crate::phy::FrlOutput = hdmi_hal::phy::FrlOutput::GapOnly;
        let _: crate::FrlOutput = output;
        let level: crate::phy::TxFfeLevel = crate::TxFfeLevel::MAX;
        assert_eq!(level.value(), 7);
        let _: crate::phy::LanePatterns = crate::LanePatterns::default();
        let _: crate::phy::EqParams = crate::EqParams::default();
        let _: crate::phy::LaneEqParams = crate::LaneEqParams::default();
        let restrictions: crate::phy::TxFfeRestrictions =
            hdmi_hal::phy::TxFfeRestrictions::default();
        let _: crate::TxFfeRestrictions = restrictions;
    }

    const VIDEO_4K60: TmdsVideo = TmdsVideo::new(
        594_000,
        display_types::ColorBitDepth::Depth8,
        display_types::ColorFormat::Rgb444,
    );

    const CONFIG_4K60: TmdsPhyConfig =
        TmdsPhyConfig::new(594_000, TmdsClockRatio::Div40, VIDEO_4K60);

    /// A minimal `TmdsPhy` that records what it is given, as `MockPhy` does for `FrlPhy`.
    #[derive(Default)]
    struct MockTmdsPhy {
        config: Option<TmdsPhyConfig>,
        scrambling: Option<bool>,
        output: bool,
    }

    impl TmdsPhy for MockTmdsPhy {
        type Error = core::convert::Infallible;

        fn capabilities(&self) -> TmdsPhyCapabilities {
            TmdsPhyCapabilities::new(600_000).with_scrambling(true)
        }

        async fn configure(&mut self, config: TmdsPhyConfig) -> Result<(), Self::Error> {
            self.config = Some(config);
            Ok(())
        }

        async fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error> {
            self.scrambling = Some(enabled);
            Ok(())
        }

        async fn start_output(&mut self) -> Result<(), Self::Error> {
            self.output = true;
            Ok(())
        }

        async fn stop_output(&mut self) -> Result<(), Self::Error> {
            self.output = false;
            Ok(())
        }
    }

    #[pollster::test]
    async fn tmds_phy_records_configure_scrambling_and_output() {
        let mut phy = MockTmdsPhy::default();
        assert!(phy.capabilities().scrambling);
        phy.stop_output().await.unwrap();
        phy.configure(CONFIG_4K60).await.unwrap();
        phy.set_scrambling(true).await.unwrap();
        phy.start_output().await.unwrap();
        assert_eq!(phy.config, Some(CONFIG_4K60));
        assert_eq!(phy.scrambling, Some(true));
        assert!(phy.output);
        phy.set_scrambling(false).await.unwrap();
        phy.stop_output().await.unwrap();
        assert_eq!(phy.scrambling, Some(false));
        assert!(!phy.output);
    }

    /// Implements only the `Send` twin; it records the calls in order.
    #[derive(Default)]
    struct SendMockTmdsPhy {
        calls: [Option<&'static str>; 4],
        n: usize,
        config: Option<TmdsPhyConfig>,
    }

    impl SendMockTmdsPhy {
        fn record(&mut self, call: &'static str) {
            self.calls[self.n] = Some(call);
            self.n += 1;
        }
    }

    impl SendTmdsPhy for SendMockTmdsPhy {
        type Error = core::convert::Infallible;

        fn capabilities(&self) -> TmdsPhyCapabilities {
            TmdsPhyCapabilities::new(340_000)
        }

        async fn configure(&mut self, config: TmdsPhyConfig) -> Result<(), Self::Error> {
            self.record("configure");
            self.config = Some(config);
            Ok(())
        }

        async fn set_scrambling(&mut self, _enabled: bool) -> Result<(), Self::Error> {
            self.record("set_scrambling");
            Ok(())
        }

        async fn start_output(&mut self) -> Result<(), Self::Error> {
            self.record("start_output");
            Ok(())
        }

        async fn stop_output(&mut self) -> Result<(), Self::Error> {
            self.record("stop_output");
            Ok(())
        }
    }

    type TmdsResults<E> = [Result<(), E>; 4];

    const TMDS_CALLS: [Option<&str>; 4] = [
        Some("stop_output"),
        Some("configure"),
        Some("set_scrambling"),
        Some("start_output"),
    ];

    /// Calls every method through `TmdsPhy`, as code written against the plain trait does.
    async fn bring_up_plain<P: TmdsPhy>(phy: &mut P) -> TmdsResults<P::Error> {
        let scrambling = phy.capabilities().scrambling;
        [
            phy.stop_output().await,
            phy.configure(CONFIG_4K60).await,
            phy.set_scrambling(scrambling).await,
            phy.start_output().await,
        ]
    }

    /// Calls every method through `SendTmdsPhy`, as code bounded by the twin does.
    async fn bring_up_send<P: SendTmdsPhy>(phy: &mut P) -> TmdsResults<P::Error> {
        let scrambling = SendTmdsPhy::capabilities(phy).scrambling;
        [
            SendTmdsPhy::stop_output(phy).await,
            SendTmdsPhy::configure(phy, CONFIG_4K60).await,
            SendTmdsPhy::set_scrambling(phy, scrambling).await,
            SendTmdsPhy::start_output(phy).await,
        ]
    }

    /// The compiler proves this `Send` bound for every `P: SendTmdsPhy`.
    fn spawnable_tmds<P: SendTmdsPhy>(
        phy: &mut P,
    ) -> impl Future<Output = TmdsResults<P::Error>> + Send + '_ {
        bring_up_send(phy)
    }

    #[pollster::test]
    async fn send_tmds_twin_is_usable_as_tmds_phy() {
        let mut phy = SendMockTmdsPhy::default();
        assert_eq!(bring_up_plain(&mut phy).await, [Ok(()); 4]);
        assert_eq!(phy.calls, TMDS_CALLS);
        assert_eq!(phy.config, Some(CONFIG_4K60));
    }

    #[pollster::test]
    async fn send_tmds_twin_futures_are_send_in_generic_code() {
        let mut phy = SendMockTmdsPhy::default();
        assert_eq!(assert_send(spawnable_tmds(&mut phy)).await, [Ok(()); 4]);
        assert_eq!(phy.calls, TMDS_CALLS);
    }

    #[test]
    fn tmds_value_types_are_hdmi_hals_at_both_paths() {
        let ratio: crate::phy::TmdsClockRatio = hdmi_hal::phy::TmdsClockRatio::Div10;
        let _: crate::TmdsClockRatio = ratio;
        let video: crate::phy::TmdsVideo = VIDEO_4K60;
        let _: crate::TmdsVideo = video;
        let config: crate::phy::TmdsPhyConfig = CONFIG_4K60;
        let _: crate::TmdsPhyConfig = config;
        let caps: crate::phy::TmdsPhyCapabilities = hdmi_hal::phy::TmdsPhyCapabilities::new(0);
        let _: crate::TmdsPhyCapabilities = caps;
    }
}
