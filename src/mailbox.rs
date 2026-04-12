use std::collections::VecDeque;

use crate::message::A2AMessage;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum MailboxError {
    #[error("inbox full")]
    InboxFull,
    #[error("insufficient energy: need {0}, have {1}")]
    InsufficientEnergy(u32, u32),
    #[error("trust too low: need {0}, have {1}")]
    TrustTooLow(f64, f64),
}

pub struct VesselMailbox {
    pub vessel_id: crate::message::VesselId,
    inbox: VecDeque<A2AMessage>,
    outbox: VecDeque<A2AMessage>,
    pub sent_log: Vec<u64>,
    pub max_inbox_size: usize,
    pub trust_required_to_send: f64,
    pub energy_budget: u32,
}

impl VesselMailbox {
    pub fn new(vessel_id: crate::message::VesselId) -> Self {
        Self {
            vessel_id,
            inbox: VecDeque::new(),
            outbox: VecDeque::new(),
            sent_log: Vec::new(),
            max_inbox_size: 256,
            trust_required_to_send: 0.5,
            energy_budget: 1000,
        }
    }

    pub fn with_max_inbox_size(mut self, size: usize) -> Self {
        self.max_inbox_size = size;
        self
    }

    pub fn with_trust_required(mut self, trust: f64) -> Self {
        self.trust_required_to_send = trust;
        self
    }

    pub fn with_energy_budget(mut self, budget: u32) -> Self {
        self.energy_budget = budget;
        self
    }

    /// Receive a message into the inbox.
    pub fn receive(&mut self, msg: A2AMessage) -> Result<(), MailboxError> {
        if self.inbox.len() >= self.max_inbox_size {
            return Err(MailboxError::InboxFull);
        }
        self.inbox.push_back(msg);
        Ok(())
    }

    /// Send a message: enqueues to outbox, deducts energy, logs message ID.
    /// Trust check must be done externally (by router) or caller can check trust_required_to_send.
    pub fn send(&mut self, msg: A2AMessage, sender_trust: f64) -> Result<(), MailboxError> {
        if sender_trust < self.trust_required_to_send {
            return Err(MailboxError::TrustTooLow(self.trust_required_to_send, sender_trust));
        }
        let cost = msg.energy_cost as u32;
        if self.energy_budget < cost {
            return Err(MailboxError::InsufficientEnergy(cost, self.energy_budget));
        }
        self.energy_budget -= cost;
        self.sent_log.push(msg.id);
        // Priority ordering: higher priority first, so insert in sorted position
        let pos = self.outbox.iter().position(|m| m.priority < msg.priority).unwrap_or(self.outbox.len());
        self.outbox.insert(pos, msg);
        Ok(())
    }

    pub fn next_incoming(&mut self) -> Option<A2AMessage> {
        self.inbox.pop_front()
    }

    pub fn next_outgoing(&mut self) -> Option<A2AMessage> {
        self.outbox.pop_front()
    }

    pub fn drain_outbox(&mut self, limit: usize) -> Vec<A2AMessage> {
        let drain_count = limit.min(self.outbox.len());
        self.outbox.drain(..drain_count).collect()
    }

    pub fn is_full(&self) -> bool {
        self.inbox.len() >= self.max_inbox_size
    }

    pub fn unread_count(&self) -> usize {
        self.inbox.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::{MessageType, VesselId};

    fn make_msg(id: u64, priority: u8) -> A2AMessage {
        A2AMessage::new(VesselId::new("sender"), VesselId::new("receiver"), MessageType::Tell)
            .with_id(id)
            .with_priority(priority)
    }

    #[test]
    fn test_receive_and_retrieve() {
        let mut mb = VesselMailbox::new(VesselId::new("me"));
        let msg = make_msg(1, 5);
        mb.receive(msg).unwrap();
        assert_eq!(mb.unread_count(), 1);
        let got = mb.next_incoming().unwrap();
        assert_eq!(got.id, 1);
        assert_eq!(mb.unread_count(), 0);
    }

    #[test]
    fn test_mailbox_full_rejection() {
        let mut mb = VesselMailbox::new(VesselId::new("me")).with_max_inbox_size(2);
        mb.receive(make_msg(1, 5)).unwrap();
        mb.receive(make_msg(2, 5)).unwrap();
        assert!(mb.is_full());
        assert_eq!(mb.receive(make_msg(3, 5)), Err(MailboxError::InboxFull));
    }

    #[test]
    fn test_send_requires_trust() {
        let mut mb = VesselMailbox::new(VesselId::new("me")).with_trust_required(0.5);
        // Low trust
        assert_eq!(
            mb.send(make_msg(1, 5), 0.3),
            Err(MailboxError::TrustTooLow(0.5, 0.3))
        );
        // Sufficient trust
        assert!(mb.send(make_msg(2, 5), 0.7).is_ok());
    }

    #[test]
    fn test_send_requires_energy() {
        let mut mb = VesselMailbox::new(VesselId::new("me")).with_energy_budget(5);
        let msg = make_msg(1, 5).with_energy_cost(10);
        assert_eq!(
            mb.send(msg, 1.0),
            Err(MailboxError::InsufficientEnergy(10, 5))
        );
    }

    #[test]
    fn test_priority_ordering_in_outbox() {
        let mut mb = VesselMailbox::new(VesselId::new("me")).with_energy_budget(1000);
        mb.send(make_msg(1, 3), 1.0).unwrap(); // low priority
        mb.send(make_msg(2, 8), 1.0).unwrap(); // high priority
        mb.send(make_msg(3, 5), 1.0).unwrap(); // medium

        let out = mb.drain_outbox(10);
        assert_eq!(out[0].id, 2); // priority 8
        assert_eq!(out[1].id, 3); // priority 5
        assert_eq!(out[2].id, 1); // priority 3
    }
}
