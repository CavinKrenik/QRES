//! Gene Storage Abstraction Layer
//!
//! Defines a trait for persisting evolved neural genes across sessions.
//! This enables "Lamarckian Evolution"—learned strategies survive reboots.

use alloc::vec::Vec;

/// Abstract interface for storing and retrieving evolved genes.
///
/// This trait allows neural nodes to "remember" learned bytecode strategies
/// even after the swarm restarts. Implementations can use disk, cloud, or
/// other persistence backends.
///
/// Constraint: Must be `no_std` compatible; implementations may use `std`.
pub trait GeneStorage {
    /// Save an evolved gene for a specific node.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the node
    /// * `gene` - The bytecode/genome to persist
    ///
    /// # Returns
    /// `true` if save succeeded, `false` otherwise
    fn save_gene(&mut self, id: u32, gene: &[u8]) -> bool;

    /// Load a previously saved gene for a node.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the node
    ///
    /// # Returns
    /// `Some(gene)` if a saved gene exists, `None` otherwise
    fn load_gene(&self, id: u32) -> Option<Vec<u8>>;
}
