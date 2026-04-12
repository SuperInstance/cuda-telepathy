use std::collections::HashMap;

use crate::message::{A2AMessage, MessageType, VesselId};
use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum RouteDecision {
    Allow,
    RateLimited,
    TrustTooLow,
    NoEnergy,
    Expired,
}

#[derive(Debug, Error, PartialEq)]
pub enum RouteError {
    #[error("expired: TTL is 0")]
    Expired,
    #[error("trust too low")]
    TrustTooLow,
}

pub struct MessageRouter {
    pub trust_store: HashMap<VesselId, f64>,
    pub energy_cost_base: u16,
    pub max_ttl: u8,
    pub rate_limit: HashMap<VesselId, u64>,
    rate_limit_window: u64,
}

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            trust_store: HashMap::new(),
            energy_cost_base: 1,
            max_ttl: 10,
            rate_limit: HashMap::new(),
            rate_limit_window: 100,
        }
    }

    pub fn with_energy_cost_base(mut self, cost: u16) -> Self {
        self.energy_cost_base = cost;
        self
    }

    pub fn with_max_ttl(mut self, ttl: u8) -> Self {
        self.max_ttl = ttl;
        self
    }

    pub fn with_rate_limit_window(mut self, window: u64) -> Self {
        self.rate_limit_window = window;
        self
    }

    pub fn route(&self, msg: &A2AMessage, sender_energy: u32) -> RouteDecision {
        if msg.ttl == 0 {
            return RouteDecision::Expired;
        }
        let trust = self.trust_store.get(&msg.from).copied().unwrap_or(0.0);
        if trust < 0.3 {
            return RouteDecision::TrustTooLow;
        }
        if msg.energy_cost as u32 > sender_energy {
            return RouteDecision::NoEnergy;
        }
        if let Some(&last) = self.rate_limit.get(&msg.from) {
            if msg.timestamp.saturating_sub(last) < self.rate_limit_window {
                return RouteDecision::RateLimited;
            }
        }
        RouteDecision::Allow
    }

    pub fn forward(&self, msg: &mut A2AMessage, _next_hop: &VesselId) -> Result<(), RouteError> {
        if msg.ttl == 0 {
            return Err(RouteError::Expired);
        }
        msg.ttl -= 1;
        msg.msg_type = MessageType::Forward;
        Ok(())
    }

    pub fn update_trust(&mut self, vessel: &VesselId, trust: f64) {
        self.trust_store.insert(vessel.clone(), trust.max(0.0).min(1.0));
    }

    pub fn check_rate_limit(&self, vessel: &VesselId, current_cycle: u64) -> bool {
        if let Some(&last) = self.rate_limit.get(vessel) {
            current_cycle.saturating_sub(last) < self.rate_limit_window
        } else {
            false
        }
    }

    pub fn record_send(&mut self, vessel: &VesselId, cycle: u64) {
        self.rate_limit.insert(vessel.clone(), cycle);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg_from(from: &str, ttl: u8) -> A2AMessage {
        A2AMessage::new(VesselId::new(from), VesselId::new("dest"), MessageType::Tell)
            .with_ttl(ttl)
            .with_timestamp(1000)
    }

    #[test]
    fn test_router_allows_trusted_messages() {
        let mut router = MessageRouter::new();
        router.update_trust(&VesselId::new("alice"), 0.8);
        let msg = msg_from("alice", 5);
        assert_eq!(router.route(&msg, 100), RouteDecision::Allow);
    }

    #[test]
    fn test_router_blocks_low_trust() {
        let mut router = MessageRouter::new();
        router.update_trust(&VesselId::new("eve"), 0.1);
        let msg = msg_from("eve", 5);
        assert_eq!(router.route(&msg, 100), RouteDecision::TrustTooLow);
    }

    #[test]
    fn test_rate_limiting() {
        let mut router = MessageRouter::new().with_rate_limit_window(50);
        router.update_trust(&VesselId::new("spammer"), 0.8);
        router.record_send(&VesselId::new("spammer"), 1000);

        // Too soon
        let msg = msg_from("spammer", 5).with_timestamp(1040);
        assert_eq!(router.route(&msg, 100), RouteDecision::RateLimited);

        // After window
        let msg = msg_from("spammer", 5).with_timestamp(1100);
        assert_eq!(router.route(&msg, 100), RouteDecision::Allow);
    }

    #[test]
    fn test_ttl_decrement_on_forward() {
        let router = MessageRouter::new();
        let mut msg = msg_from("alice", 5);
        router.forward(&mut msg, &VesselId::new("bob")).unwrap();
        assert_eq!(msg.ttl, 4);
        assert_eq!(msg.msg_type, MessageType::Forward);
    }

    #[test]
    fn test_expired_messages_rejected() {
        let router = MessageRouter::new();
        let mut msg = msg_from("alice", 0);
        assert_eq!(router.forward(&mut msg, &VesselId::new("bob")), Err(RouteError::Expired));
        assert_eq!(router.route(&msg, 100), RouteDecision::Expired);
    }
}
