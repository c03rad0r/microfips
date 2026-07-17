//! LR2021 framing layer — fragments byte streams into FLRC-sized packets
//! and reassembles them back into a byte stream.
//!
//! The `Transport` trait in `microfips-protocol` is stream-oriented
//! (`send(&[u8])` / `recv(&mut [u8]) -> usize`), but the LR2021 radio
//! transmits fixed-size packets (max 255 bytes FLRC payload).
//!
//! This module provides the coalescing/fragmentation logic that bridges
//! the two paradigms. It is hardware-agnostic and fully unit-testable.
//!
//! ## TX Path (byte stream → packets)
//!
//! [`TxFramer`] accumulates bytes from successive `send()` calls into an
//! internal buffer. When the buffer reaches `MAX_PACKET` bytes, it produces
//! a full packet. Remaining bytes stay buffered until the next call or
//! until [`TxFramer::flush_packet`] is called.
//!
//! ## RX Path (packets → byte stream)
//!
//! [`RxFramer`] holds a ring of received packet bytes. Each `recv()` call
//! drains up to `buf.len()` bytes from the buffer. When the buffer is
//! empty, the caller must supply a new packet via [`RxFramer::push_packet`].
//!
//! ## Wire Format
//!
//! Each FLRC packet carries raw bytes from the Transport stream — no
//! additional framing header is added at this layer. The `FrameWriter`
//! in `microfips-protocol` already wraps payloads with a 2-byte LE length
//! prefix, so the stream is self-delimiting. We just chunk it into
//! radio-sized pieces.

#![cfg_attr(not(test), no_std)]

use heapless::Vec;

/// Maximum FLRC payload per packet (proven baseline: 255 bytes at 2600 kbps).
pub const MAX_PACKET: usize = 255;

/// TX-side framer: accumulates bytes, produces MAX_PACKET-sized chunks.
#[derive(Debug)]
pub struct TxFramer {
    buf: Vec<u8, MAX_PACKET>,
}

impl TxFramer {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
        }
    }

    /// Push bytes into the framer. Returns number of bytes consumed from `data`.
    /// If buffer fills to MAX_PACKET, exactly MAX_PACKET bytes are consumed
    /// and the caller should call `take_packet()` to extract the full packet,
    /// then call `push()` again for remaining bytes.
    ///
    /// This design avoids borrow issues: push returns a count, not a slice.
    pub fn push(&mut self, data: &[u8]) -> usize {
        let space = MAX_PACKET - self.buf.len();
        let take = data.len().min(space);
        for &b in &data[..take] {
            let _ = self.buf.push(b);
        }
        take
    }

    /// Returns true if buffer is full (MAX_PACKET bytes ready to transmit).
    pub fn is_full(&self) -> bool {
        self.buf.len() >= MAX_PACKET
    }

    /// Flush buffered bytes as a packet (may be shorter than MAX_PACKET).
    /// Returns None if buffer is empty.
    pub fn flush_packet(&mut self) -> Option<&[u8]> {
        if self.buf.is_empty() {
            return None;
        }
        let slice = self.buf.as_slice();
        // Return reference to current buffer contents.
        // Caller must call .clear() after transmitting.
        Some(slice)
    }

    /// Take the buffered bytes as an owned packet, clearing the buffer.
    /// This is the preferred method when you need to transmit and clear
    /// in one step (avoids borrow issues).
    pub fn take_packet(&mut self, out: &mut [u8]) -> Option<usize> {
        if self.buf.is_empty() {
            return None;
        }
        let n = self.buf.len().min(out.len());
        out[..n].copy_from_slice(&self.buf[..n]);
        self.buf.clear();
        Some(n)
    }

    /// Clear the TX buffer after a packet has been transmitted.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Number of bytes currently buffered.
    pub fn pending(&self) -> usize {
        self.buf.len()
    }

    /// Check if buffer has data waiting to be flushed.
    pub fn has_pending(&self) -> bool {
        !self.buf.is_empty()
    }
}

impl Default for TxFramer {
    fn default() -> Self {
        Self::new()
    }
}

/// RX-side framer: buffers received packet bytes, drains on demand.
#[derive(Debug)]
pub struct RxFramer {
    buf: Vec<u8, { MAX_PACKET * 2 }>,
    read_pos: usize,
}

impl RxFramer {
    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
            read_pos: 0,
        }
    }

    /// Push a received packet's bytes into the framer.
    /// Returns false if the buffer would overflow (packet dropped).
    pub fn push_packet(&mut self, data: &[u8]) -> bool {
        // Compact if read_pos has advanced
        if self.read_pos > 0 {
            let remaining = self.buf.len() - self.read_pos;
            self.buf.copy_within(self.read_pos.., 0);
            // SAFETY: we just copied `remaining` bytes to the front
            unsafe {
                self.buf.set_len(remaining);
            }
            self.read_pos = 0;
        }
        // Check capacity
        if self.buf.len() + data.len() > self.buf.capacity() {
            return false;
        }
        for &b in data {
            if self.buf.push(b).is_err() {
                return false;
            }
        }
        true
    }

    /// Drain up to `buf.len()` bytes into `buf`. Returns number of bytes copied.
    /// Returns 0 if no bytes are available (caller should wait for next packet).
    pub fn drain(&mut self, buf: &mut [u8]) -> usize {
        let available = self.buf.len() - self.read_pos;
        if available == 0 {
            return 0;
        }
        let n = buf.len().min(available);
        buf[..n].copy_from_slice(&self.buf[self.read_pos..self.read_pos + n]);
        self.read_pos += n;

        // Compact if fully drained
        if self.read_pos >= self.buf.len() {
            self.buf.clear();
            self.read_pos = 0;
        }
        n
    }

    /// Number of bytes available to drain.
    pub fn available(&self) -> usize {
        self.buf.len() - self.read_pos
    }
}

impl Default for RxFramer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TxFramer tests ──────────────────────────────────────────────

    #[test]
    fn test_tx_small_chunk_no_flush() {
        let mut f = TxFramer::new();
        assert_eq!(f.push(b"hello"), 5);
        assert!(!f.is_full());
        assert_eq!(f.pending(), 5);
        assert!(f.has_pending());
    }

    #[test]
    fn test_tx_flush_partial() {
        let mut f = TxFramer::new();
        f.push(b"hello");
        let mut pkt = [0u8; MAX_PACKET];
        let n = f.take_packet(&mut pkt).unwrap();
        assert_eq!(n, 5);
        assert_eq!(&pkt[..n], b"hello");
        assert!(!f.has_pending());
    }

    #[test]
    fn test_tx_fill_exactly_max() {
        let mut f = TxFramer::new();
        let data = [0xABu8; MAX_PACKET];
        let consumed = f.push(&data);
        assert_eq!(consumed, MAX_PACKET);
        assert!(f.is_full());

        let mut pkt = [0u8; MAX_PACKET];
        let n = f.take_packet(&mut pkt).unwrap();
        assert_eq!(n, MAX_PACKET);
        assert!(!f.has_pending());
    }

    #[test]
    fn test_tx_push_more_than_space_truncates() {
        let mut f = TxFramer::new();
        f.push(b"hello"); // 5 bytes buffered
        let data = [0x42u8; MAX_PACKET]; // push MAX_PACKET more
        let consumed = f.push(&data);
        // Only MAX_PACKET - 5 = 250 bytes fit
        assert_eq!(consumed, MAX_PACKET - 5);
        assert!(f.is_full());
        assert_eq!(f.pending(), MAX_PACKET);

        // Take the packet
        let mut pkt = [0u8; MAX_PACKET];
        let n = f.take_packet(&mut pkt).unwrap();
        assert_eq!(n, MAX_PACKET);
        // First 5 bytes should be "hello"
        assert_eq!(&pkt[..5], b"hello");
        // Rest should be 0x42
        assert_eq!(&pkt[5..], &[0x42u8; MAX_PACKET - 5]);
    }

    #[test]
    fn test_tx_multi_packet_large_data() {
        let mut f = TxFramer::new();
        let data = [0xCDu8; 600];

        // First push: fills to MAX_PACKET (255 bytes consumed)
        let consumed1 = f.push(&data);
        assert_eq!(consumed1, MAX_PACKET);
        assert!(f.is_full());

        // Extract first packet
        let mut pkt = [0u8; MAX_PACKET];
        let n1 = f.take_packet(&mut pkt).unwrap();
        assert_eq!(n1, MAX_PACKET);
        assert_eq!(&pkt[..MAX_PACKET], &data[..MAX_PACKET]);

        // Push remaining 345 bytes — buffer is 255 capacity, so only 255 fit
        let consumed2 = f.push(&data[MAX_PACKET..]);
        assert_eq!(consumed2, MAX_PACKET); // 345 bytes in, 255 consumed
        assert!(f.is_full());

        // Extract second full packet
        let n2 = f.take_packet(&mut pkt).unwrap();
        assert_eq!(n2, MAX_PACKET);
        assert_eq!(&pkt[..MAX_PACKET], &data[MAX_PACKET..MAX_PACKET * 2]);

        // Push remaining 90 bytes
        let consumed3 = f.push(&data[MAX_PACKET * 2..]);
        assert_eq!(consumed3, 90);
        assert!(!f.is_full());

        // Flush final partial packet
        let n3 = f.take_packet(&mut pkt).unwrap();
        assert_eq!(n3, 90);
        assert_eq!(&pkt[..90], &data[MAX_PACKET * 2..]);
    }

    // ── RxFramer tests ──────────────────────────────────────────────

    #[test]
    fn test_rx_push_and_drain() {
        let mut f = RxFramer::new();
        f.push_packet(b"hello world");
        let mut buf = [0u8; 64];
        let n = f.drain(&mut buf);
        assert_eq!(n, 11);
        assert_eq!(&buf[..n], b"hello world");
        assert_eq!(f.available(), 0);
    }

    #[test]
    fn test_rx_partial_drain() {
        let mut f = RxFramer::new();
        f.push_packet(b"abcdefghij"); // 10 bytes
        let mut buf = [0u8; 4];
        assert_eq!(f.drain(&mut buf), 4);
        assert_eq!(&buf, b"abcd");
        assert_eq!(f.drain(&mut buf), 4);
        assert_eq!(&buf, b"efgh");
        assert_eq!(f.drain(&mut buf), 2);
        assert_eq!(&buf[..2], b"ij");
        assert_eq!(f.drain(&mut buf), 0);
    }

    #[test]
    fn test_rx_multiple_packets() {
        let mut f = RxFramer::new();
        f.push_packet(b"AAA");
        f.push_packet(b"BBB");
        let mut buf = [0u8; 64];
        let n = f.drain(&mut buf);
        assert_eq!(n, 6);
        assert_eq!(&buf[..n], b"AAABBB");
    }

    #[test]
    fn test_rx_drain_empty() {
        let mut f = RxFramer::new();
        let mut buf = [0u8; 8];
        assert_eq!(f.drain(&mut buf), 0);
    }

    #[test]
    fn test_rx_compact_after_partial_drain() {
        let mut f = RxFramer::new();
        f.push_packet(&[0u8; 200]);
        let mut buf = [0u8; 100];
        f.drain(&mut buf); // drain 100, read_pos=100
        assert_eq!(f.available(), 100);
        f.push_packet(b"more"); // should compact + append
        assert_eq!(f.available(), 104);
        let mut buf2 = [0u8; 200];
        let n = f.drain(&mut buf2);
        assert_eq!(n, 104);
    }

    // ── Round-trip: TX → radio → RX ─────────────────────────────────

    #[test]
    fn test_roundtrip_small_frame() {
        // Simulates FrameWriter sending header(2) + payload(20) over LR2021
        let mut tx = TxFramer::new();
        let mut rx = RxFramer::new();

        let header = 20u16.to_le_bytes();
        let payload = [0x55u8; 20];

        // FrameWriter calls send(header) then send(payload)
        tx.push(&header);
        tx.push(&payload);

        // Flush as a packet (simulates radio TX)
        let mut pkt = [0u8; MAX_PACKET];
        let n = tx.take_packet(&mut pkt).unwrap();

        // Radio RX receives the packet
        rx.push_packet(&pkt[..n]);

        // FrameReader calls recv() to get bytes
        let mut buf = [0u8; 64];
        let n = rx.drain(&mut buf);
        assert_eq!(n, 22); // 2 header + 20 payload
        assert_eq!(&buf[..2], &header);
        assert_eq!(&buf[2..22], &payload);
    }

    #[test]
    fn test_roundtrip_large_frame_fragments() {
        // Frame larger than MAX_PACKET — requires fragmentation
        let mut tx = TxFramer::new();
        let mut rx = RxFramer::new();

        let payload = [0x77u8; 500];
        let header = 500u16.to_le_bytes();

        tx.push(&header);
        // Push 500 bytes — only 253 fit (255 - 2 header already buffered)
        let consumed1 = tx.push(&payload);
        assert_eq!(consumed1, 253);
        assert!(tx.is_full());

        // Extract first packet (255 bytes: 2 header + 253 payload)
        let mut pkt1 = [0u8; MAX_PACKET];
        let pkt1_len = tx.take_packet(&mut pkt1).unwrap();
        assert_eq!(pkt1_len, 255);

        // Radio TX/RX first packet
        rx.push_packet(&pkt1[..pkt1_len]);

        // Push remaining 247 bytes of payload
        let consumed2 = tx.push(&payload[consumed1..]);
        assert_eq!(consumed2, 247);

        // Extract second packet
        let mut pkt2 = [0u8; MAX_PACKET];
        let pkt2_len = tx.take_packet(&mut pkt2).unwrap();
        assert_eq!(pkt2_len, 247);

        // Radio TX/RX second packet
        rx.push_packet(&pkt2[..pkt2_len]);

        // Drain all bytes from RX
        let mut buf = [0u8; 600];
        let total = rx.drain(&mut buf);
        // Total: 255 + 247 = 502 (2 header + 500 payload)
        assert_eq!(total, 502);
        assert_eq!(&buf[..2], &header);
        assert_eq!(buf[2..502], [0x77u8; 500]);
    }

    #[test]
    fn test_roundtrip_multiple_small_frames() {
        // Two small frames in one packet (coalescing)
        let mut tx = TxFramer::new();
        let mut rx = RxFramer::new();

        // Frame 1: 2-byte header + 5-byte payload = 7 bytes
        let hdr1 = 5u16.to_le_bytes();
        let pl1 = b"hello";

        // Frame 2: 2-byte header + 5-byte payload = 7 bytes
        let hdr2 = 5u16.to_le_bytes();
        let pl2 = b"world";

        tx.push(&hdr1);
        tx.push(pl1);
        tx.push(&hdr2);
        tx.push(pl2);

        // All 14 bytes fit in one packet
        let mut pkt = [0u8; MAX_PACKET];
        let n = tx.take_packet(&mut pkt).unwrap();
        assert_eq!(n, 14);

        rx.push_packet(&pkt[..n]);

        let mut buf = [0u8; 64];
        let total = rx.drain(&mut buf);
        assert_eq!(total, 14);
        assert_eq!(&buf[..7], &[5, 0, b'h', b'e', b'l', b'l', b'o']);
        assert_eq!(&buf[7..14], &[5, 0, b'w', b'o', b'r', b'l', b'd']);
    }
}
