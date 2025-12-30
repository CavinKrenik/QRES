use tokenizers::Tokenizer;
use std::sync::OnceLock;

// Helper to embed and lazy-load
static SEMANTIC_TOKENIZER: OnceLock<Tokenizer> = OnceLock::new();

const TOKENIZER_JSON: &[u8] = include_bytes!("../assets/tokenizer.json");

pub struct SemanticEngine;

impl SemanticEngine {
    fn get() -> &'static Tokenizer {
        SEMANTIC_TOKENIZER.get_or_init(|| {
            Tokenizer::from_bytes(TOKENIZER_JSON).expect("Failed to load embedded tokenizer")
        })
    }

    pub fn encode(text: &str) -> Vec<u32> {
        let tokenizer = Self::get();
        // Encodes to IDs (u32)
        if let Ok(encoding) = tokenizer.encode(text, false) {
             encoding.get_ids().to_vec()
        } else {
             Vec::new() // Fail gracefully
        }
    }

    pub fn decode(ids: &[u32]) -> String {
        let tokenizer = Self::get();
        tokenizer.decode(ids, false).unwrap_or_else(|_| String::new())
    }
}
