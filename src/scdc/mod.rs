/// Async raw register access to the SCDC register map over DDC/I²C.
///
/// This is the async counterpart to `hdmi_hal::scdc::ScdcTransport`. The trait
/// contract is identical; only the method signatures gain `async`.
///
/// SCDC is a bidirectional channel used for FRL link training initiation, scrambling
/// control, and CED (Character Error Detection) reporting. Any code that needs to touch
/// the SCDC register map does so through this trait.
///
/// The trait operates at the raw register level: a one-byte address and a one-byte
/// value. Typed register wrappers, named constants, and multi-register sequences belong
/// in the protocol crate, not here.
pub trait ScdcTransport {
    /// Error type returned by transport operations.
    type Error;

    /// Read a single byte from the given SCDC register address.
    async fn read(&mut self, reg: u8) -> Result<u8, Self::Error>;

    /// Write a single byte to the given SCDC register address.
    async fn write(&mut self, reg: u8, value: u8) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockTransport {
        registers: [u8; 256],
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                registers: [0u8; 256],
            }
        }
    }

    impl ScdcTransport for MockTransport {
        type Error = core::convert::Infallible;

        async fn read(&mut self, reg: u8) -> Result<u8, Self::Error> {
            Ok(self.registers[reg as usize])
        }

        async fn write(&mut self, reg: u8, value: u8) -> Result<(), Self::Error> {
            self.registers[reg as usize] = value;
            Ok(())
        }
    }

    #[pollster::test]
    async fn write_then_read_returns_written_value() {
        let mut t = MockTransport::new();
        t.write(0x20, 0xAB).await.unwrap();
        assert_eq!(t.read(0x20).await.unwrap(), 0xAB);
    }

    #[pollster::test]
    async fn unwritten_register_reads_zero() {
        let mut t = MockTransport::new();
        assert_eq!(t.read(0x31).await.unwrap(), 0x00);
    }

    #[pollster::test]
    async fn write_is_register_specific() {
        let mut t = MockTransport::new();
        t.write(0x01, 0xFF).await.unwrap();
        assert_eq!(t.read(0x00).await.unwrap(), 0x00);
        assert_eq!(t.read(0x02).await.unwrap(), 0x00);
    }

    #[pollster::test]
    async fn write_overwrites_previous_value() {
        let mut t = MockTransport::new();
        t.write(0x20, 0x01).await.unwrap();
        t.write(0x20, 0x02).await.unwrap();
        assert_eq!(t.read(0x20).await.unwrap(), 0x02);
    }
}
