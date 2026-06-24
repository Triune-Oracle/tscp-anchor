//! TSCP Event Algebra V1
//!
//! Frozen compatibility layer.
//! This module does not redefine runtime semantics.

pub use crate::event::EventEnvelope;

pub type EventId = [u8; 16];
pub type StateHash = [u8; 32];

pub const GENESIS_STATE: StateHash = [0u8; 32];
