use alloc::string::String;
use serde::{Deserialize, Serialize};

#[cfg(feature = "cli")]
use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum PredictorType {
    Heuristic,
    Neural,
    Hybrid,
    Zero,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum CoderType {
    Huffman,
    Arithmetic,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(ValueEnum))]
pub enum CompressionMode {
    Lossless,
    Lossy, // Simple lossy mode
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "cli", derive(Args))]
pub struct QresConfig {
    /// Predictor Strategy (Heuristic, Neural, Hybrid, Zero)
    #[cfg_attr(feature = "cli", arg(long, value_enum, default_value_t = PredictorType::Hybrid))]
    pub predictor: PredictorType,

    /// Entropy Coder (Huffman, Arithmetic)
    #[cfg_attr(feature = "cli", arg(long, value_enum, default_value_t = CoderType::Arithmetic))]
    pub coder: CoderType,

    /// Compression Mode (Lossless, Lossy, Adaptive)
    #[cfg_attr(feature = "cli", arg(long, value_enum, default_value_t = CompressionMode::Adaptive))]
    pub mode: CompressionMode,

    /// Variance threshold for Hybrid/Adaptive switching
    #[cfg_attr(feature = "cli", arg(long, default_value = "0.01"))]
    pub threshold: f32,

    /// Window size for prediction history
    #[cfg_attr(feature = "cli", arg(long, default_value_t = 32))]
    pub window_size: usize,

    /// Path to ONNX model file (optional)
    #[cfg_attr(feature = "cli", arg(long))]
    pub model_path: Option<String>,
}

impl Default for QresConfig {
    fn default() -> Self {
        Self {
            predictor: PredictorType::Hybrid,
            coder: CoderType::Arithmetic,
            mode: CompressionMode::Adaptive,
            threshold: 0.01,
            window_size: 32,
            model_path: None,
        }
    }
}

impl QresConfig {
    #[cfg(feature = "std")]
    pub fn create_predictor(&self) -> alloc::boxed::Box<dyn crate::predictors::Predictor> {
        match self.predictor {
            PredictorType::Zero => alloc::boxed::Box::new(crate::predictors::ZeroPredictor::new()),
            PredictorType::Heuristic => {
                alloc::boxed::Box::new(crate::predictors::SimplePredictor::new())
            }
            PredictorType::Neural => {
                // Neural predictor uses GraphPredictor (learning predictor) until
                // byte-level ONNX wrapper is implemented
                eprintln!("⚠️ INFO: Neural mode using GraphPredictor (learning)");
                alloc::boxed::Box::new(crate::predictors::GraphPredictor::new())
            }
            PredictorType::Hybrid => {
                // Hybrid uses LzMatchPredictor for pattern matching
                alloc::boxed::Box::new(crate::predictors::LzMatchPredictor::new())
            }
        }
    }
}
