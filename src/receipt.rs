use std::collections::{HashMap, HashSet};

use crate::message::VesselId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReceiptStatus {
    Delivered,
    Read,
    Rejected,
    Expired,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub msg_id: u64,
    pub from: VesselId,
    pub to: VesselId,
    pub status: ReceiptStatus,
    pub timestamp: u64,
}

pub struct ReceiptTracker {
    pending: HashMap<u64, u64>,  // msg_id -> send_cycle
    delivered: HashSet<u64>,
    pub timeout_cycles: u64,
}

impl ReceiptTracker {
    pub fn new(timeout_cycles: u64) -> Self {
        Self {
            pending: HashMap::new(),
            delivered: HashSet::new(),
            timeout_cycles,
        }
    }

    /// Register a message as pending delivery.
    pub fn send(&mut self, msg_id: u64, cycle: u64) {
        self.pending.insert(msg_id, cycle);
    }

    /// Record a receipt.
    pub fn acknowledge(&mut self, receipt: &DeliveryReceipt) {
        self.pending.remove(&receipt.msg_id);
        if receipt.status == ReceiptStatus::Delivered || receipt.status == ReceiptStatus::Read {
            self.delivered.insert(receipt.msg_id);
        }
    }

    /// Check for timed-out messages. Returns expired message IDs.
    pub fn check_timeouts(&mut self, current_cycle: u64) -> Vec<u64> {
        let expired: Vec<u64> = self.pending
            .iter()
            .filter(|&(_, &sent)| current_cycle.saturating_sub(sent) > self.timeout_cycles)
            .map(|(&id, _)| id)
            .collect();
        for &id in &expired {
            self.pending.remove(&id);
        }
        expired
    }

    pub fn is_delivered(&self, msg_id: u64) -> bool {
        self.delivered.contains(&msg_id)
    }

    pub fn is_pending(&self, msg_id: u64) -> bool {
        self.pending.contains_key(&msg_id)
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_receipt_tracking() {
        let mut tracker = ReceiptTracker::new(100);
        tracker.send(1, 10);
        tracker.send(2, 10);

        assert!(tracker.is_pending(1));
        assert!(!tracker.is_delivered(1));

        let receipt = DeliveryReceipt {
            msg_id: 1,
            from: VesselId::new("a"),
            to: VesselId::new("b"),
            status: ReceiptStatus::Delivered,
            timestamp: 20,
        };
        tracker.acknowledge(&receipt);

        assert!(!tracker.is_pending(1));
        assert!(tracker.is_delivered(1));
        assert!(tracker.is_pending(2));
        assert_eq!(tracker.pending_count(), 1);
    }

    #[test]
    fn test_timeout() {
        let mut tracker = ReceiptTracker::new(50);
        tracker.send(1, 10);

        let expired = tracker.check_timeouts(70);
        assert_eq!(expired, vec![1]);
        assert!(!tracker.is_pending(1));
    }
}
