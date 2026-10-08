use display_types::cea861::hdmi_forum::HdmiForumFrl;
use hdmi_hal::phy::{EqParams, LtpPattern};

/// Async PHY lane configuration for an HDMI 2.1 transmitter or receiver.
///
/// This is the async counterpart to `hdmi_hal::phy::HdmiPhy`. The trait contract is
/// identical; only the method signatures gain `async`.
///
/// Abstracts the register sequences required to configure an HDMI 2.1 PHY: lane
/// mapping, pre-emphasis, equalization, scrambling, and FRL rate selection.
/// Vendor-specific register sequences are an implementation detail of each backend.
pub trait HdmiPhy {
    /// Error type returned by PHY operations.
    type Error;

    /// Select the FRL rate (or TMDS). Triggers the required lane reconfiguration sequence.
    async fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error>;

    /// Drive the given link training pattern on the physical lanes.
    async fn send_ltp(&mut self, pattern: LtpPattern) -> Result<(), Self::Error>;

    /// Adjust equalization parameters after link training feedback.
    async fn adjust_equalization(&mut self, params: EqParams) -> Result<(), Self::Error>;

    /// Enable or disable scrambling on the PHY.
    async fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPhy {
        frl_rate: Option<HdmiForumFrl>,
        scrambling: Option<bool>,
        eq_calls: u32,
        last_ltp: Option<LtpPattern>,
    }

    impl MockPhy {
        fn new() -> Self {
            Self {
                frl_rate: None,
                scrambling: None,
                eq_calls: 0,
                last_ltp: None,
            }
        }
    }

    impl HdmiPhy for MockPhy {
        type Error = core::convert::Infallible;

        async fn set_frl_rate(&mut self, rate: HdmiForumFrl) -> Result<(), Self::Error> {
            self.frl_rate = Some(rate);
            Ok(())
        }

        async fn send_ltp(&mut self, pattern: LtpPattern) -> Result<(), Self::Error> {
            self.last_ltp = Some(pattern);
            Ok(())
        }

        async fn adjust_equalization(&mut self, _params: EqParams) -> Result<(), Self::Error> {
            self.eq_calls += 1;
            Ok(())
        }

        async fn set_scrambling(&mut self, enabled: bool) -> Result<(), Self::Error> {
            self.scrambling = Some(enabled);
            Ok(())
        }
    }

    #[pollster::test]
    async fn send_ltp_records_pattern() {
        let mut phy = MockPhy::new();
        phy.send_ltp(LtpPattern::Lfsr0).await.unwrap();
        assert_eq!(phy.last_ltp, Some(LtpPattern::Lfsr0));
    }

    #[pollster::test]
    async fn send_ltp_updates_on_each_call() {
        let mut phy = MockPhy::new();
        phy.send_ltp(LtpPattern::Lfsr0).await.unwrap();
        phy.send_ltp(LtpPattern::Lfsr2).await.unwrap();
        assert_eq!(phy.last_ltp, Some(LtpPattern::Lfsr2));
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
    async fn set_scrambling_enable() {
        let mut phy = MockPhy::new();
        phy.set_scrambling(true).await.unwrap();
        assert_eq!(phy.scrambling, Some(true));
    }

    #[pollster::test]
    async fn set_scrambling_disable() {
        let mut phy = MockPhy::new();
        phy.set_scrambling(false).await.unwrap();
        assert_eq!(phy.scrambling, Some(false));
    }

    #[pollster::test]
    async fn set_scrambling_can_be_toggled() {
        let mut phy = MockPhy::new();
        phy.set_scrambling(true).await.unwrap();
        phy.set_scrambling(false).await.unwrap();
        assert_eq!(phy.scrambling, Some(false));
    }

    #[pollster::test]
    async fn adjust_equalization_tracks_call_count() {
        let mut phy = MockPhy::new();
        phy.adjust_equalization(EqParams::new()).await.unwrap();
        phy.adjust_equalization(EqParams::new()).await.unwrap();
        assert_eq!(phy.eq_calls, 2);
    }
}
