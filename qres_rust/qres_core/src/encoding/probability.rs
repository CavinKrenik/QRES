/// Adaptive probability model for arithmetic coding.
///
/// This model tracks symbol frequencies and provides cumulative probability ranges
/// needed for arithmetic encoding. Uses Laplace smoothing (starting count = 1 for all symbols)
/// to prevent zero-probability crashes.
pub struct AdaptiveModel {
    /// Frequency counts for each byte (0-255).
    counts: [u32; 256],
    /// Total count of all symbols.
    total: u32,
}

impl Default for AdaptiveModel {
    fn default() -> Self {
        Self::new()
    }
}

impl AdaptiveModel {
    /// Maximum total count before rescaling (prevents overflow).
    const MAX_TOTAL: u32 = 1 << 14; // 16384

    /// Creates a new model with Laplace smoothing (count=1 for all symbols).
    pub fn new() -> Self {
        Self {
            counts: [1; 256],
            total: 256,
        }
    }

    /// Updates the model by incrementing the count for the given symbol.
    ///
    /// Args:
    ///     symbol: The byte value to update.
    pub fn update(&mut self, symbol: u8) {
        self.counts[symbol as usize] += 1;
        self.total += 1;

        // Rescale if counts are getting too large
        if self.total >= Self::MAX_TOTAL {
            self.rescale();
        }
    }

    /// Rescales all counts by dividing by 2, maintaining minimum count of 1.
    fn rescale(&mut self) {
        self.total = 0;
        for count in &mut self.counts {
            *count = (*count / 2).max(1);
            self.total += *count;
        }
    }

    /// Returns the probability range for a symbol.
    ///
    /// Args:
    ///     symbol: The byte value to query.
    ///
    /// Returns:
    ///     (cumulative_start, symbol_count, total_count): The range for arithmetic coding.
    pub fn get_probability(&self, symbol: u8) -> (u32, u32, u32) {
        let mut cumulative = 0u32;
        for i in 0..(symbol as usize) {
            cumulative += self.counts[i];
        }
        let count = self.counts[symbol as usize];
        (cumulative, count, self.total)
    }

    /// Returns the symbol for a given cumulative count (for decoding).
    ///
    /// Args:
    ///     target: The cumulative count to find.
    ///
    /// Returns:
    ///     The symbol whose cumulative range contains the target.
    pub fn symbol_from_count(&self, target: u32) -> u8 {
        let mut cumulative = 0u32;
        for (i, &count) in self.counts.iter().enumerate() {
            cumulative += count;
            if cumulative > target {
                return i as u8;
            }
        }
        255
    }

    /// Returns the total count.
    pub fn total(&self) -> u32 {
        self.total
    }
}
