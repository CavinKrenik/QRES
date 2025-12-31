use candle_core::{Device, DType, IndexOp, Result, Tensor, D};
use candle_nn::{linear, linear_no_bias, Embedding, LayerNorm, LayerNormConfig, Module, ops, VarBuilder};
use lazy_static::lazy_static;

const EMBED_DIM: usize = 128;
const HIDDEN_DIM: usize = 256;
const HEADS: usize = 4;
const LAYERS: usize = 2;
const SEQ_LEN: usize = 256;

struct TransformerBlock {
    ln1: LayerNorm,
    ln2: LayerNorm,
    // For attn: Load PyTorch-packed weights
    attn_in_proj: candle_nn::Linear,  // [embed*3, embed] no bias
    attn_out_proj: candle_nn::Linear, // [embed, embed]
    ff1: candle_nn::Linear,
    ff2: candle_nn::Linear,
}

impl TransformerBlock {
    fn new(vs: VarBuilder) -> Result<Self> {
        let ln_cfg = LayerNormConfig { eps: 1e-5, ..Default::default() };
        Ok(Self {
            ln1: candle_nn::layer_norm(EMBED_DIM, ln_cfg, vs.pp("ln1"))?,
            ln2: candle_nn::layer_norm(EMBED_DIM, ln_cfg, vs.pp("ln2"))?,
            attn_in_proj: linear_no_bias(EMBED_DIM, EMBED_DIM * 3, vs.pp("attn.in_proj"))?,
            attn_out_proj: linear(EMBED_DIM, EMBED_DIM, vs.pp("attn.out_proj"))?,
            ff1: linear(EMBED_DIM, HIDDEN_DIM, vs.pp("ff.0"))?,
            ff2: linear(HIDDEN_DIM, EMBED_DIM, vs.pp("ff.2"))?,
        })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let x_norm = self.ln1.forward(x)?;
        
        // Project and split for MHA (mimic PyTorch)
        let qkv = self.attn_in_proj.forward(&x_norm)?;
        let chunk_dim = qkv.dim(D::Minus1)? / 3;
        let q = qkv.narrow(D::Minus1, 0, chunk_dim)?;
        let k = qkv.narrow(D::Minus1, chunk_dim, chunk_dim)?;
        let v = qkv.narrow(D::Minus1, 2 * chunk_dim, chunk_dim)?;
        
        // Reshape for heads (batch, seq, embed) -> (batch, heads, seq, embed/heads)
        let head_dim = EMBED_DIM / HEADS;
        let q = q.reshape(((), (), HEADS, head_dim))?.transpose(1, 2)?.contiguous()?;
        let k = k.reshape(((), (), HEADS, head_dim))?.transpose(1, 2)?.contiguous()?;
        let v = v.reshape(((), (), HEADS, head_dim))?.transpose(1, 2)?.contiguous()?;
        
        // Scaled dot-product attention
        let attn_scores = q.matmul(&k.transpose(2, 3)?)? / (head_dim as f64).sqrt();
        let attn_probs = ops::softmax(&attn_scores, D::Minus1)?;
        let attn_out = attn_probs.matmul(&v)?;
        
        // Concat heads and project out
        let attn_out = attn_out.transpose(1, 2)?.contiguous()?.reshape(x.shape())?;
        let attn_out = self.attn_out_proj.forward(&attn_out)?;
        
        let x = x.add(&attn_out)?;
        
        let x_norm = self.ln2.forward(&x)?;
        let ff_out = self.ff2.forward(&self.ff1.forward(&x_norm)?.relu()?)?;
        x.add(&ff_out)
    }
}

struct MetaTransformer {
    embed: Embedding,
    blocks: Vec<TransformerBlock>,
    head: candle_nn::Linear,
}

impl MetaTransformer {
    fn new(vs: VarBuilder) -> Result<Self> {
        let blocks = (0..LAYERS).map(|i| TransformerBlock::new(vs.pp(&format!("blocks.{}", i)))).collect::<Result<Vec<_>>>()?;
        Ok(Self {
            embed: candle_nn::embedding(256, EMBED_DIM, vs.pp("embed"))?,
            blocks,
            head: linear(EMBED_DIM, 4, vs.pp("head"))?,
        })
    }

    fn forward(&self, input: &Tensor) -> Result<Tensor> {
        let mut x = self.embed.forward(input)?;
        for block in &self.blocks {
            x = block.forward(&x)?;
        }
        let x = x.mean(1)?;
        self.head.forward(&x)
    }
}

// Lazy static as before
lazy_static! {
    static ref MODEL: MetaTransformer = {
        let device = Device::Cpu;
        let path = "assets/meta_brain.safetensors";
        let vs = unsafe { VarBuilder::from_mmaped_safetensors(&[path], DType::F32, &device).expect("Load failed") };
        MetaTransformer::new(vs).expect("Build failed")
    };
}

pub fn predict(input: &[u8]) -> (u8, &'static str) {
    // Same as your code, but add error handling
    let device = Device::Cpu;
    let len = std::cmp::min(input.len(), SEQ_LEN);
    let mut input_vec = input[..len].to_vec();
    input_vec.resize(SEQ_LEN, 0); // Pad with 0
    let input_u32: Vec<u32> = input_vec.iter().map(|&b| b as u32).collect();
    
    match (|| -> Result<u8> {
        let input_tensor = Tensor::from_slice(&input_u32, (1, SEQ_LEN), &device)?;
        let logits = MODEL.forward(&input_tensor)?;
        let probs = ops::softmax(&logits, D::Minus1)?;
        Ok(probs.argmax(D::Minus1)?.to_scalar::<u32>()? as u8)
    })() {
        Ok(idx) => match idx {
            0 => (1, "Neural (Linear)"),
            1 => (5, "Neural (iPEPS)"),
            2 => (6, "Neural (Zstd)"),
            3 => (7, "Neural (Text)"),
            _ => (6, "Fallback"),
        },
        Err(e) => {
            eprintln!("Predict error: {:?}", e);
            (6, "Fallback")
        }
    }
}
