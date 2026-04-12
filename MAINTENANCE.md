# MAINTENANCE.md

## Architecture Decisions

### Why Binary Encoding (Not JSON)

The fleet operates on constrained networks where bandwidth matters. A typical A2A message with a small payload is ~30 bytes in binary vs ~200+ bytes in JSON. At fleet scale with thousands of messages per cycle, this 6-7x reduction is significant.

The binary format is deliberately simple — no variable-length integers, no schema negotiation, no version negotiation headers. Every message is exactly `26 + payload_len` bytes. This makes parsing trivial and eliminates entire classes of deserialization bugs.

The tradeoff: binary is less debuggable than JSON. We mitigate this with clear test vectors and a `Debug` impl on `A2AMessage` that shows the human-readable form.

### 1050-Byte Limit Rationale

The 1050-byte maximum (26-byte header + 1024-byte payload) was chosen to:

1. **Fit in a single UDP packet** on most networks (standard MTU is 1500 bytes, leaving room for IP/UDP headers)
2. **Prevent memory exhaustion** — a malicious or buggy vessel can't send a multi-megabyte message
3. **Enforce conciseness** — vessels must think about what they're saying, not dump raw data

If a vessel needs to send more data, it should use the `Fork`/`Join` pattern to chunk and reassemble, or publish to a shared store and send a reference.

### Trust Gating on Send

Messages are trust-checked at two points:

1. **In the mailbox** — the sending vessel checks its own trust level against a configurable threshold before enqueuing
2. **In the router** — the receiving vessel's router checks the sender's trust before accepting

This double-check prevents both accidental spam (low-trust vessel trying to send) and deliberate flooding (a vessel that bypasses its own mailbox checks).

### Rate Limiting

Rate limiting uses a simple sliding window per vessel. The router tracks the last send timestamp and rejects messages sent within the window period. This prevents burst flooding while allowing steady communication.

The window is configurable and should be tuned based on fleet size and message volume.

### Receipt Tracking

Delivery receipts provide at-most-once delivery semantics with timeout detection:

1. When a message is sent, it's registered as "pending" with the current cycle
2. When a receipt arrives (Delivered/Read/Rejected), the pending entry is resolved
3. If no receipt arrives within `timeout_cycles`, the message is marked expired

This is intentionally simple — no retry logic, no exponential backoff. The sending vessel decides what to do with expired messages based on its own policies.
