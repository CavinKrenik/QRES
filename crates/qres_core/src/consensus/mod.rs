//! Consensus algorithms for Byzantine Fault Tolerant distributed learning.
//!
//! This module provides deterministic aggregation algorithms using fixed-point
//! arithmetic (I16F16) to guarantee bit-perfect consensus across heterogeneous
//! microcontroller architectures.

pub mod krum;

pub use krum::aggregate_krum;
