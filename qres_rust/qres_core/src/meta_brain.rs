use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use serde::Deserialize;

const NUM_INPUTS: usize = 4;
const NUM_OUTPUTS: usize = 6;

#[derive(Deserialize, Debug)]
struct MetaBrainWeights {
    scaler_mean: Vec<f32>,
    scaler_scale: Vec<f32>,
    layer_0_weights: Vec<Vec<f32>>,
    layer_0_bias: Vec<f32>,
    layer_1_weights: Vec<Vec<f32>>,
    layer_1_bias: Vec<f32>,
    layer_2_weights: Vec<Vec<f32>>,
    layer_2_bias: Vec<f32>,
}

pub struct MetaBrain {
    weights: MetaBrainWeights,
}

impl MetaBrain {
    pub fn new() -> Option<Self> {
        // Embed weights for portability (and WASM support)
        let json = include_str!("../assets/meta_brain_v2.json");
        let weights: MetaBrainWeights = serde_json::from_str(json).ok()?;
        Some(MetaBrain { weights })
    }

    fn dense(input: &[f32], weights: &[Vec<f32>], bias: &[f32]) -> Vec<f32> {
        let out_dim = bias.len();
        let in_dim = input.len();
        let mut output = vec![0.0; out_dim];

        for i in 0..out_dim {
            let mut sum = 0.0;
            for j in 0..in_dim {
                // Weights shape: [in_dim][out_dim] usually in sklearn: coefs_ is [n_features, n_units]
                // My export was: coefs_[0].tolist().
                // Weights from sklearn are (n_in, n_out).
                // So weights[j][i].
                // Let's verify export format: `layer_0_weights` is list of lists.
                // Outer list is input dim? Inner list is output dim?
                // Sklearn: coefs_ is list of shape (n_in, n_out).
                // So weights[j] is the vector of weights for input feature j contributing to all outputs.
                // Correct.
                sum += input[j] * weights[j][i];
            }
            output[i] = sum + bias[i];
        }
        output
    }

    fn relu(x: &mut [f32]) {
        for v in x.iter_mut() {
            if *v < 0.0 {
                *v = 0.0;
            }
        }
    }

    pub fn forward(&self, features: &[f32]) -> [f32; NUM_OUTPUTS] {
        // 1. Scale
        let mut x = vec![0.0; NUM_INPUTS];
        for i in 0..NUM_INPUTS {
            x[i] = (features[i] - self.weights.scaler_mean[i]) / self.weights.scaler_scale[i];
        }

        // 2. L0
        let mut h0 = Self::dense(
            &x,
            &self.weights.layer_0_weights,
            &self.weights.layer_0_bias,
        );
        Self::relu(&mut h0);

        // 3. L1
        let mut h1 = Self::dense(
            &h0,
            &self.weights.layer_1_weights,
            &self.weights.layer_1_bias,
        );
        Self::relu(&mut h1);

        // 4. L2
        let out = Self::dense(
            &h1,
            &self.weights.layer_2_weights,
            &self.weights.layer_2_bias,
        );

        let mut result = [0.0; NUM_OUTPUTS];
        // Copy available weights from network (v2 has 5 outputs)
        let n = out.len().min(NUM_OUTPUTS);
        result[..n].copy_from_slice(&out[..n]);
        result
    }
}

pub fn calculate_features(data: &[u8]) -> [f32; 4] {
    if data.is_empty() {
        return [0.0; 4];
    }

    let mut counts = [0usize; 256];
    let mut sum = 0.0;
    let mut sum_sq = 0.0;

    for &b in data {
        counts[b as usize] += 1;
        sum += b as f32;
        let val = b as f32;
        sum_sq += val * val;
    }

    let n = data.len() as f32;
    let mean = sum / n;
    let variance = sum_sq / n - mean * mean;

    let mut entropy = 0.0;
    for &c in &counts {
        if c > 0 {
            let p = c as f32 / n;
            entropy -= p * libm::log2f(p);
        }
    }

    // Autocorrelation Lag 1
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..data.len() - 1 {
        let diff1 = data[i] as f32 - mean;
        let diff2 = data[i + 1] as f32 - mean;
        num += diff1 * diff2;
        den += diff1 * diff1;
    }
    let autocorr_1 = if den != 0.0 { num / den } else { 0.0 };

    [entropy, mean, variance, autocorr_1]
}

lazy_static! {
    static ref META_BRAIN: Option<MetaBrain> = MetaBrain::new();
}

pub fn predict_init_weights(chunk: &[u8]) -> Option<[f32; NUM_OUTPUTS]> {
    if let Some(brain) = &*META_BRAIN {
        // Use first 512 bytes for speed
        let header_len = chunk.len().min(512);
        let features = calculate_features(&chunk[0..header_len]);
        let weights = brain.forward(&features);

        // Normalize (Softmax or just simple abs norm? Our training data was approx normalized)
        // Let's apply Softmax to enforce distribution
        let mut max_w = -f32::INFINITY;
        for &w in &weights {
            if w > max_w {
                max_w = w;
            }
        }

        let mut sum = 0.0;
        let mut exp_w = [0.0; NUM_OUTPUTS];
        for (i, &w) in weights.iter().enumerate() {
            exp_w[i] = libm::expf(w - max_w);
            sum += exp_w[i];
        }

        for w in &mut exp_w {
            *w /= sum;
        }

        Some(exp_w)
    } else {
        None
    }
}
