use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Cursor;

/// A compressor that skips values if they can be predicted within a specific error bound.
///
/// This demonstrates the core concept of "Perceptual Compression" for IoT:
/// if the sensor hasn't changed enough to matter, don't send the data.
pub struct ErrorBoundedCompressor {
    /// The maximum allowed difference between actual and predicted value.
    pub error_bound: f32,
}

impl ErrorBoundedCompressor {
    /// Creates a new compressor with the specified error tolerance.
    pub fn new(error_bound: f32) -> Self {
        Self { error_bound }
    }

    /// Compresses a stream of floating point values.
    ///
    /// Algorithm:
    /// 1. Predict next value (using Last-Value predictor).
    /// 2. If abs(actual - predicted) <= bound, increment skip count.
    /// 3. If diff > bound, write any pending skips, then write the new actual value.
    ///
    /// Output Format (Binary):
    /// - Tag (u8): 0x01 = Literal Value, 0x02 = Skip Run
    /// - Payload: f32 (for Literal) or u32 (for Run Length)
    pub fn compress(&self, data: &[f32]) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        if data.is_empty() {
            return Ok(buffer);
        }

        // Always write the first value as a reference
        let mut last_value = data[0];
        cursor.write_u8(0x01)?; // Tag: Literal
        cursor.write_f32::<LittleEndian>(last_value)?;

        let mut skip_count: u32 = 0;

        for &value in data.iter().skip(1) {
            let prediction = last_value;
            let diff = (value - prediction).abs();

            if diff <= self.error_bound {
                // Value is predictable; skip it
                skip_count += 1;
            } else {
                // Value deviated; flush skips and write new value
                if skip_count > 0 {
                    cursor.write_u8(0x02)?; // Tag: Skip Run
                    cursor.write_u32::<LittleEndian>(skip_count)?;
                    skip_count = 0;
                }

                cursor.write_u8(0x01)?; // Tag: Literal
                cursor.write_f32::<LittleEndian>(value)?;
                last_value = value;
            }
        }

        // Flush any trailing skips
        if skip_count > 0 {
            cursor.write_u8(0x02)?;
            cursor.write_u32::<LittleEndian>(skip_count)?;
        }

        Ok(buffer)
    }
}
