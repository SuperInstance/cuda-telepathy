use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VesselId(pub String);

impl VesselId {
    pub fn new(s: impl Into<String>) -> Self {
        VesselId(s.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessageType {
    Tell = 0,
    Ask = 1,
    Delegate = 2,
    Broadcast = 3,
    Reduce = 4,
    Reply = 5,
    Forward = 6,
    Listen = 7,
    Fork = 8,
    Join = 9,
    Wait = 10,
    Signal = 11,
}

impl MessageType {
    pub(crate) fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(MessageType::Tell),
            1 => Some(MessageType::Ask),
            2 => Some(MessageType::Delegate),
            3 => Some(MessageType::Broadcast),
            4 => Some(MessageType::Reduce),
            5 => Some(MessageType::Reply),
            6 => Some(MessageType::Forward),
            7 => Some(MessageType::Listen),
            8 => Some(MessageType::Fork),
            9 => Some(MessageType::Join),
            10 => Some(MessageType::Wait),
            11 => Some(MessageType::Signal),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AMessage {
    pub id: u64,
    pub from: VesselId,
    pub to: VesselId,
    pub msg_type: MessageType,
    pub payload: Vec<u8>,
    pub confidence: Option<f64>,
    pub energy_cost: u16,
    pub ttl: u8,
    pub timestamp: u64,
    pub reply_to: Option<u64>,
    pub in_reply_to: Option<u64>,
    pub priority: u8,
}

impl A2AMessage {
    pub fn new(from: VesselId, to: VesselId, msg_type: MessageType) -> Self {
        Self {
            id: 0,
            from,
            to,
            msg_type,
            payload: Vec::new(),
            confidence: None,
            energy_cost: 1,
            ttl: 10,
            timestamp: 0,
            reply_to: None,
            in_reply_to: None,
            priority: 5,
        }
    }

    pub fn with_id(mut self, id: u64) -> Self {
        self.id = id;
        self
    }

    pub fn with_payload(mut self, payload: Vec<u8>) -> Self {
        self.payload = payload;
        self
    }

    pub fn with_timestamp(mut self, ts: u64) -> Self {
        self.timestamp = ts;
        self
    }

    pub fn with_priority(mut self, p: u8) -> Self {
        self.priority = p.min(9);
        self
    }

    pub fn with_energy_cost(mut self, cost: u16) -> Self {
        self.energy_cost = cost;
        self
    }

    pub fn with_ttl(mut self, ttl: u8) -> Self {
        self.ttl = ttl;
        self
    }
}
