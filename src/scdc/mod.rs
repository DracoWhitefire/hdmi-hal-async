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
    async fn read(&self, reg: u8) -> Result<u8, Self::Error>;

    /// Write a single byte to the given SCDC register address.
    async fn write(&mut self, reg: u8, value: u8) -> Result<(), Self::Error>;

    /// Read consecutive SCDC registers, starting at `reg`, into `buf`.
    ///
    /// The default implementation calls [`read`](Self::read) once per byte, stopping at
    /// the first error. Transports that can read several bytes in one DDC/I²C
    /// transaction should override it, so that related registers (such as the status
    /// flags or the character error counters) are read together rather than one at a
    /// time while the sink may be updating them. A read past register `0xFF` is not
    /// meaningful; the default implementation wraps the address to `0x00`.
    async fn read_block(&self, reg: u8, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut addr = reg;
        for byte in buf {
            *byte = self.read(addr).await?;
            addr = addr.wrapping_add(1);
        }
        Ok(())
    }
}

/// [`ScdcTransport`] with `Send` futures, for code that spawns link training on a
/// multi-threaded executor.
///
/// `ScdcTransport`'s `async fn`s promise nothing about their futures, so code generic over
/// `T: ScdcTransport` cannot spawn a future that calls them, however `T` is bounded. This
/// trait has the same methods and contract, each returning a future that is `Send`, and
/// errors that are `Send` too: a spawned task returns them. Implementations still write
/// `async fn`; the compiler checks that each future is `Send`.
///
/// The transport is `Sync` as well as `Send`: [`read`](Self::read) takes `&self`, and a
/// future holding `&self` is only `Send` when `Self` is `Sync`. A transport that keeps
/// interior state for reads uses a `Sync` type for it (an atomic or a lock rather than a
/// `Cell`).
///
/// Every `SendScdcTransport` is an `ScdcTransport`, through a blanket implementation, so
/// it works with everything written against `ScdcTransport` as well (a `read_block`
/// override included); a type implements one of the two (implementing both conflicts with
/// the blanket implementation). A `SendScdcTransport` type has both traits' methods, so
/// with both traits in scope a method call on it is ambiguous: import only the trait the
/// code is bounded by, or call `SendScdcTransport::read(…)`.
///
/// Bound generic code by `SendScdcTransport` alone. Adding `+ ScdcTransport`, which it
/// implies anyway, makes the compiler use that bound instead of the blanket
/// implementation, and the futures are no longer known to be `Send`.
pub trait SendScdcTransport: Send + Sync {
    /// Error type returned by transport operations.
    type Error: Send;

    /// See [`ScdcTransport::read`].
    fn read(&self, reg: u8) -> impl Future<Output = Result<u8, Self::Error>> + Send;

    /// See [`ScdcTransport::write`].
    fn write(&mut self, reg: u8, value: u8)
    -> impl Future<Output = Result<(), Self::Error>> + Send;

    /// See [`ScdcTransport::read_block`]; the default implementation is the same, one
    /// [`read`](Self::read) per byte.
    fn read_block(
        &self,
        reg: u8,
        buf: &mut [u8],
    ) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async move {
            let mut addr = reg;
            for byte in buf {
                *byte = self.read(addr).await?;
                addr = addr.wrapping_add(1);
            }
            Ok(())
        }
    }
}

impl<T: SendScdcTransport> ScdcTransport for T {
    type Error = <T as SendScdcTransport>::Error;

    fn read(&self, reg: u8) -> impl Future<Output = Result<u8, Self::Error>> {
        <T as SendScdcTransport>::read(self, reg)
    }

    fn write(&mut self, reg: u8, value: u8) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendScdcTransport>::write(self, reg, value)
    }

    fn read_block(&self, reg: u8, buf: &mut [u8]) -> impl Future<Output = Result<(), Self::Error>> {
        <T as SendScdcTransport>::read_block(self, reg, buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;
    use core::sync::atomic::{AtomicU32, Ordering};

    struct MockTransport {
        registers: [u8; 256],
        reads: Cell<u32>,
        fail_at: Option<u8>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                registers: [0u8; 256],
                reads: Cell::new(0),
                fail_at: None,
            }
        }

        /// A transport whose every register holds its own address.
        fn filled() -> Self {
            let mut t = Self::new();
            for (i, r) in t.registers.iter_mut().enumerate() {
                *r = i as u8;
            }
            t
        }
    }

    impl ScdcTransport for MockTransport {
        type Error = u8;

        async fn read(&self, reg: u8) -> Result<u8, Self::Error> {
            if self.fail_at == Some(reg) {
                return Err(reg);
            }
            self.reads.set(self.reads.get() + 1);
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
        let t = MockTransport::new();
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

    #[pollster::test]
    async fn read_block_reads_consecutive_registers() {
        let t = MockTransport::filled();
        let mut buf = [0u8; 3];
        t.read_block(0x40, &mut buf).await.unwrap();
        assert_eq!(buf, [0x40, 0x41, 0x42]);
        assert_eq!(t.reads.get(), 3);
    }

    #[pollster::test]
    async fn read_block_empty_buffer_reads_nothing() {
        let t = MockTransport::filled();
        t.read_block(0x40, &mut []).await.unwrap();
        assert_eq!(t.reads.get(), 0);
    }

    #[pollster::test]
    async fn read_block_stops_at_first_error() {
        let mut t = MockTransport::filled();
        t.fail_at = Some(0x41);
        let mut buf = [0u8; 3];
        assert_eq!(t.read_block(0x40, &mut buf).await, Err(0x41));
        assert_eq!(buf, [0x40, 0x00, 0x00]);
        assert_eq!(t.reads.get(), 1);
    }

    #[pollster::test]
    async fn read_block_wraps_past_0xff() {
        let t = MockTransport::filled();
        let mut buf = [0u8; 2];
        t.read_block(0xFF, &mut buf).await.unwrap();
        assert_eq!(buf, [0xFF, 0x00]);
    }

    #[pollster::test]
    async fn write_then_read_block() {
        let mut t = MockTransport::new();
        t.write(0x20, 0xAB).await.unwrap();
        let mut buf = [0u8; 1];
        t.read_block(0x20, &mut buf).await.unwrap();
        assert_eq!(buf, [0xAB]);
    }

    /// Implements only the `Send` twin, with the default `read_block`. Counts reads in an
    /// atomic, since the twin needs `Sync`.
    struct SendMockTransport {
        registers: [u8; 256],
        reads: AtomicU32,
        fail_at: Option<u8>,
    }

    impl SendMockTransport {
        fn filled() -> Self {
            let mut registers = [0u8; 256];
            for (i, r) in registers.iter_mut().enumerate() {
                *r = i as u8;
            }
            Self {
                registers,
                reads: AtomicU32::new(0),
                fail_at: None,
            }
        }

        fn reads(&self) -> u32 {
            self.reads.load(Ordering::Relaxed)
        }
    }

    impl SendScdcTransport for SendMockTransport {
        type Error = u8;

        async fn read(&self, reg: u8) -> Result<u8, Self::Error> {
            if self.fail_at == Some(reg) {
                return Err(reg);
            }
            self.reads.fetch_add(1, Ordering::Relaxed);
            Ok(self.registers[reg as usize])
        }

        async fn write(&mut self, reg: u8, value: u8) -> Result<(), Self::Error> {
            self.registers[reg as usize] = value;
            Ok(())
        }
    }

    /// Implements only the `Send` twin and overrides `read_block`, as a transport that
    /// reads a block in one transaction does.
    struct BlockReadingTransport;

    impl SendScdcTransport for BlockReadingTransport {
        type Error = ();

        async fn read(&self, _reg: u8) -> Result<u8, Self::Error> {
            Err(())
        }

        async fn write(&mut self, _reg: u8, _value: u8) -> Result<(), Self::Error> {
            Err(())
        }

        async fn read_block(&self, reg: u8, buf: &mut [u8]) -> Result<(), Self::Error> {
            buf.fill(reg);
            Ok(())
        }
    }

    /// Writes, then reads back one register and a block, through `ScdcTransport`, as code
    /// written against the plain trait does.
    async fn round_trip_plain<T: ScdcTransport>(
        t: &mut T,
        buf: &mut [u8],
    ) -> (
        Result<(), T::Error>,
        Result<u8, T::Error>,
        Result<(), T::Error>,
    ) {
        let written = t.write(0x20, 0xAB).await;
        let read = t.read(0x20).await;
        let block = t.read_block(0x20, buf).await;
        (written, read, block)
    }

    /// The compiler proves this `Send` bound for every `T: SendScdcTransport`, not only
    /// for the mock; with `ScdcTransport` in its place it would not compile.
    fn spawnable<'a, T: SendScdcTransport>(
        t: &'a T,
        buf: &'a mut [u8],
    ) -> impl Future<Output = Result<(), T::Error>> + Send + 'a {
        SendScdcTransport::read_block(t, 0x40, buf)
    }

    fn assert_send<T: Send>(value: T) -> T {
        value
    }

    #[pollster::test]
    async fn send_twin_is_usable_as_scdc_transport() {
        let mut t = SendMockTransport::filled();
        let mut buf = [0u8; 2];
        let (written, read, block) = round_trip_plain(&mut t, &mut buf).await;
        assert_eq!((written, read, block), (Ok(()), Ok(0xAB), Ok(())));
        assert_eq!(buf, [0xAB, 0x21]);
        assert_eq!(t.reads(), 3);
    }

    #[pollster::test]
    async fn send_twin_futures_are_send_in_generic_code() {
        let t = SendMockTransport::filled();
        let mut buf = [0u8; 3];
        assert_send(spawnable(&t, &mut buf)).await.unwrap();
        assert_eq!(buf, [0x40, 0x41, 0x42]);
    }

    #[pollster::test]
    async fn send_twin_default_read_block_stops_at_first_error() {
        let mut t = SendMockTransport::filled();
        t.fail_at = Some(0x41);
        let mut buf = [0u8; 3];
        assert_eq!(
            SendScdcTransport::read_block(&t, 0x40, &mut buf).await,
            Err(0x41)
        );
        assert_eq!(buf, [0x40, 0x00, 0x00]);
        assert_eq!(t.reads(), 1);
    }

    #[pollster::test]
    async fn send_twin_default_read_block_wraps_past_0xff() {
        let t = SendMockTransport::filled();
        let mut buf = [0u8; 2];
        SendScdcTransport::read_block(&t, 0xFF, &mut buf)
            .await
            .unwrap();
        assert_eq!(buf, [0xFF, 0x00]);
    }

    #[pollster::test]
    async fn send_twin_read_block_override_is_used_through_scdc_transport() {
        let mut t = BlockReadingTransport;
        let mut buf = [0u8; 2];
        let (written, read, block) = round_trip_plain(&mut t, &mut buf).await;
        assert_eq!((written, read, block), (Err(()), Err(()), Ok(())));
        assert_eq!(buf, [0x20, 0x20]);
    }
}
