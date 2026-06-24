# TSCP Event Algebra V1

Status: Frozen

This document freezes the runtime event semantics implemented by TSCP Anchor v0.1.

No new event types are introduced here.

The compatibility boundary consists of:

- EventEnvelope
- TransitionKind
- TransitionReceipt
- ReplayEngine

The payload hash remains opaque at this layer.

Genesis state:
0000000000000000000000000000000000000000000000000000000000000000

Receipt domain separation:
TSCP_TRANSITION_RECEIPT_V1
