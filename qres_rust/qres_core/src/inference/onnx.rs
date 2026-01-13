use anyhow::{Context, Result};
use std::path::Path;

/// Neural predictor using ONNX runtime via tract.
///
/// This provides inference for the TinyPredictor model exported from PyTorch.
/// The model expects a fixed window of 32 float values and outputs a single prediction.
pub struct NeuralPredictor {
    #[allow(clippy::type_complexity)]
    model: tract_onnx::prelude::SimplePlan<
        tract_onnx::prelude::TypedFact,
        Box<dyn tract_onnx::prelude::TypedOp>,
        tract_onnx::prelude::Graph<
            tract_onnx::prelude::TypedFact,
            Box<dyn tract_onnx::prelude::TypedOp>,
        >,
    >,
    window_size: usize,
}

impl NeuralPredictor {
    /// The expected input window size for the model.
    pub const WINDOW_SIZE: usize = 32;

    /// Loads an ONNX model from the specified path.
    ///
    /// Args:
    ///     path: Path to the ONNX model file.
    ///
    /// Returns:
    ///     A new NeuralPredictor instance.
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        use tract_onnx::prelude::*;

        let model = tract_onnx::onnx()
            .model_for_path(path.as_ref())
            .context("Failed to load ONNX model")?
            .with_input_fact(0, f32::fact([1, Self::WINDOW_SIZE]).into())
            .context("Failed to set input fact")?
            .into_optimized()
            .context("Failed to optimize model")?
            .into_runnable()
            .context("Failed to create runnable model")?;

        Ok(Self {
            model,
            window_size: Self::WINDOW_SIZE,
        })
    }

    /// Predicts the next value given a window of previous values.
    ///
    /// Args:
    ///     window: A slice of exactly `WINDOW_SIZE` float values.
    ///
    /// Returns:
    ///     The predicted next value.
    pub fn predict(&self, window: &[f32]) -> Result<f32> {
        use tract_onnx::prelude::*;

        if window.len() != self.window_size {
            anyhow::bail!(
                "Expected window of size {}, got {}",
                self.window_size,
                window.len()
            );
        }

        // Create input tensor
        let input = tract_ndarray::Array2::from_shape_vec((1, self.window_size), window.to_vec())
            .context("Failed to create input array")?;

        let input_tensor: Tensor = input.into();

        // Run inference
        let result = self
            .model
            .run(tvec![input_tensor.into()])
            .context("Failed to run inference")?;

        // Extract output
        let output = result[0]
            .to_array_view::<f32>()
            .context("Failed to extract output")?;

        Ok(output[[0, 0]])
    }

    /// Returns the expected window size.
    pub fn window_size(&self) -> usize {
        self.window_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_size() {
        assert_eq!(NeuralPredictor::WINDOW_SIZE, 32);
    }

    #[test]
    fn test_inference() -> Result<()> {
        use std::path::PathBuf;

        // Locate asset relative to crate root
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("assets/predictor_v2.onnx");

        if !d.exists() {
            println!("Skipping inference test: model not found at {:?}", d);
            return Ok(());
        }

        let predictor = NeuralPredictor::load(&d)?;

        // Create a simple sine wave input
        let mut window = [0.0f32; 32];
        for (i, val) in window.iter_mut().enumerate() {
            *val = (i as f32 * 0.2).sin();
        }

        let prediction = predictor.predict(&window)?;
        println!("Prediction: {}", prediction);

        // Simple sanity check: prediction should be finite
        assert!(prediction.is_finite());

        Ok(())
    }
}
