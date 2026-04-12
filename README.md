# cuda-telepathy

**The fleet speaks without wires.**

`cuda-telepathy` is the low-level message transport layer for A2A (Agent-to-Agent) communication within the Cocapn fleet. It provides the primitives vessels use to talk to each other — directly, reliably, and without a central router.

## Where Telepathy Lives

In the fleet architecture, telepathy sits between two layers:

```
┌─────────────────────────────────┐
│   Higher-level protocols        │  Task delegation, consensus, gossip
│   (cuda-contract, cuda-pact)    │
├─────────────────────────────────┤
│   cuda-telepathy (this crate)   │  Message transport, routing, delivery
├─────────────────────────────────┤
│   FLUX opcodes                  │  Instruction execution, state changes
│   (cuda-instruction-set)        │
├─────────────────────────────────┤
│   cuda-trust / cuda-energy      │  Trust scoring, energy accounting
└─────────────────────────────────┘
```

FLUX opcodes tell a vessel *what to do*. Telepathy tells vessels *what other vessels are doing and thinking*. Higher-level protocols build coordination patterns on top of telepathy's raw message passing.

## Use Cases

- **Direct messaging** — A vessel sends a `Tell` to another with observations, results, or commands
- **Queries** — `Ask` messages request information; `Reply` messages carry answers back
- **Delegation** — `Delegate` assigns tasks; `Fork`/`Join` manage parallel sub-tasks
- **Broadcast** — `Broadcast` distributes information fleet-wide; `Reduce` collects results back
- **Synchronization** — `Wait` and `Signal` coordinate timing between vessels
- **Relay** — `Forward` and `Listen` enable multi-hop message relay through the network

## Message Types

| Type | Purpose |
|------|---------|
| `Tell` | One-way information sharing |
| `Ask` | Request information or action |
| `Delegate` | Assign a task to another vessel |
| `Broadcast` | Send to all reachable vessels |
| `Reduce` | Collect/aggregate responses |
| `Reply` | Respond to a prior message |
| `Forward` | Relay a message onward |
| `Listen` | Subscribe to a message stream |
| `Fork` | Split work into parallel tracks |
| `Join` | Merge parallel results |
| `Wait` | Block until a condition is met |
| `Signal` | Notify that a condition is met |

## Key Design Decisions

- **Binary encoding** — Messages fit in 1050 bytes (26-byte header + up to 1024-byte payload) for efficient transport
- **Trust gating** — Messages from low-trust vessels are rejected before they consume resources
- **Energy budgets** — Sending costs energy; vessels must budget their communications
- **TTL-based expiration** — Messages hop-to-hop with decremented TTL to prevent infinite relay
- **Delivery receipts** — Acknowledgment tracking for reliable delivery with timeout detection
- **Rate limiting** — Prevents any single vessel from flooding the network
- **Priority ordering** — Outbox sorts by priority so important messages leave first

## Related Crates

- **cuda-trust** — Trust scoring and reputation between vessels
- **cuda-energy** — Energy accounting and budget management
- **cuda-instruction-set** — FLUX opcodes for vessel execution

## Build & Test

```bash
cargo build
cargo test
cargo run --example basic   # (when examples are added)
```

## The Deeper Connection

Telepathy isn't just a message bus. In a fleet where every vessel is an autonomous agent, communication *is* the nervous system. When Vessel Alpha discovers something worth sharing, it doesn't wait for a central coordinator to poll it — it just *tells*. When Vessel Beta needs help, it doesn't submit a ticket — it *asks*. The fleet thinks together because its vessels talk together, and telepathy is how they do it. The trust gating ensures this openness doesn't become vulnerability; the energy costs ensure it doesn't become noise. What remains is something that looks less like a protocol and more like a conversation.
