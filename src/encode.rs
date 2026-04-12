use crate::message::{A2AMessage, MessageType, VesselId};
use thiserror::Error;

const MAX_PAYLOAD: usize = 1024;
const HEADER_SIZE: usize = 26;

#[derive(Debug, Error, PartialEq)]
pub enum DecodeError {
    #[error("buffer too short: expected {0} bytes, got {1}")]
    TooShort(usize, usize),
    #[error("payload too large: {0} bytes exceeds max {1}")]
    PayloadTooLarge(usize, usize),
    #[error("unknown message type: {0}")]
    UnknownMessageType(u8),
    #[error("invalid vessel id")]
    InvalidVesselId,
}

/// Encode a VesselId as 8 bytes (truncate or pad with zeros).
fn encode_vessel_id(vid: &VesselId, buf: &mut [u8; 8]) {
    let bytes = vid.0.as_bytes();
    let len = bytes.len().min(8);
    buf[..len].copy_from_slice(&bytes[..len]);
    if len < 8 {
        buf[len..].fill(0);
    }
}

/// Decode a VesselId from 8 bytes.
fn decode_vessel_id(buf: &[u8; 8]) -> Result<VesselId, DecodeError> {
    let end = buf.iter().rposition(|&b| b != 0).map(|i| i + 1).unwrap_or(0);
    let s = std::str::from_utf8(&buf[..end]).map_err(|_| DecodeError::InvalidVesselId)?;
    Ok(VesselId::new(s.to_owned()))
}

/// Encode an A2AMessage into a compact binary format (max 1050 bytes).
pub fn encode(msg: &A2AMessage) -> Result<Vec<u8>, EncodeError> {
    if msg.payload.len() > MAX_PAYLOAD {
        return Err(EncodeError::PayloadTooLarge(msg.payload.len(), MAX_PAYLOAD));
    }

    let payload_len = msg.payload.len() as u16;
    let total = HEADER_SIZE + msg.payload.len();
    let mut buf = Vec::with_capacity(total);

    // from (8 bytes)
    let mut from_buf = [0u8; 8];
    encode_vessel_id(&msg.from, &mut from_buf);
    buf.extend_from_slice(&from_buf);

    // to (8 bytes)
    let mut to_buf = [0u8; 8];
    encode_vessel_id(&msg.to, &mut to_buf);
    buf.extend_from_slice(&to_buf);

    // msg_type (1 byte)
    buf.push(msg.msg_type as u8);

    // ttl (1 byte)
    buf.push(msg.ttl);

    // energy_cost (2 bytes, big-endian)
    buf.extend_from_slice(&msg.energy_cost.to_be_bytes());

    // timestamp (4 bytes, big-endian)
    buf.extend_from_slice(&(msg.timestamp as u32).to_be_bytes());

    // payload_len (2 bytes, big-endian)
    buf.extend_from_slice(&payload_len.to_be_bytes());

    // payload (variable)
    buf.extend_from_slice(&msg.payload);

    Ok(buf)
}

#[derive(Debug, Error, PartialEq)]
pub enum EncodeError {
    #[error("payload too large: {0} bytes exceeds max {1}")]
    PayloadTooLarge(usize, usize),
}

/// Decode an A2AMessage from binary format.
pub fn decode(data: &[u8]) -> Result<A2AMessage, DecodeError> {
    if data.len() < HEADER_SIZE {
        return Err(DecodeError::TooShort(HEADER_SIZE, data.len()));
    }

    let mut from_buf = [0u8; 8];
    from_buf.copy_from_slice(&data[0..8]);
    let from = decode_vessel_id(&from_buf)?;

    let mut to_buf = [0u8; 8];
    to_buf.copy_from_slice(&data[8..16]);
    let to = decode_vessel_id(&to_buf)?;

    let msg_type = MessageType::from_u8(data[16])
        .ok_or(DecodeError::UnknownMessageType(data[16]))?;

    let ttl = data[17];

    let energy_cost = u16::from_be_bytes([data[18], data[19]]);

    let timestamp = u32::from_be_bytes([data[20], data[21], data[22], data[23]]) as u64;

    let payload_len = u16::from_be_bytes([data[24], data[25]]) as usize;

    if data.len() < HEADER_SIZE + payload_len {
        return Err(DecodeError::TooShort(HEADER_SIZE + payload_len, data.len()));
    }

    let payload = data[HEADER_SIZE..HEADER_SIZE + payload_len].to_vec();

    Ok(A2AMessage {
        id: 0,
        from,
        to,
        msg_type,
        payload,
        confidence: None,
        energy_cost,
        ttl,
        timestamp,
        reply_to: None,
        in_reply_to: None,
        priority: 5,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let msg = A2AMessage::new(
            VesselId::new("alpha"),
            VesselId::new("beta"),
            MessageType::Tell,
        )
        .with_id(42)
        .with_payload(b"hello world".to_vec())
        .with_timestamp(12345)
        .with_energy_cost(3)
        .with_ttl(7);

        let encoded = encode(&msg).unwrap();
        let decoded = decode(&encoded).unwrap();

        assert_eq!(decoded.from, VesselId::new("alpha"));
        assert_eq!(decoded.to, VesselId::new("beta"));
        assert_eq!(decoded.msg_type, MessageType::Tell);
        assert_eq!(decoded.payload, b"hello world");
        assert_eq!(decoded.energy_cost, 3);
        assert_eq!(decoded.ttl, 7);
        assert_eq!(decoded.timestamp, 12345);
    }

    #[test]
    fn test_all_message_types() {
        for mt in [
            MessageType::Tell, MessageType::Ask, MessageType::Delegate,
            MessageType::Broadcast, MessageType::Reduce, MessageType::Reply,
            MessageType::Forward, MessageType::Listen, MessageType::Fork,
            MessageType::Join, MessageType::Wait, MessageType::Signal,
        ] {
            let msg = A2AMessage::new(VesselId::new("a"), VesselId::new("b"), mt);
            let enc = encode(&msg).unwrap();
            let dec = decode(&enc).unwrap();
            assert_eq!(dec.msg_type, mt);
        }
    }

    #[test]
    fn test_payload_too_large() {
        let msg = A2AMessage::new(VesselId::new("a"), VesselId::new("b"), MessageType::Tell)
            .with_payload(vec![0u8; 1025]);
        assert!(encode(&msg).is_err());
    }

    #[test]
    fn test_decode_too_short() {
        assert!(decode(&[0; 10]).is_err());
    }
}
