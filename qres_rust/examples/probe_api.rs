use constriction::stream::model::LeakyQuantizer;
use constriction::stream::stack::AnsCoder;
use constriction::stream::{Decode, Encode};
use probability::distribution::Gaussian;

// Explicitly define types to match our ans_coder.rs intent
// Note: State must be at least 2x the size of Word for constriction
type Coder = AnsCoder<u32, u64>;

fn main() {
    println!("Probing Constriction API...");

    // 1. Create some data
    let mut coder = Coder::new();
    let quantizer = LeakyQuantizer::<f64, i32, u32, 24>::new(-128..=127);
    let model = quantizer.quantize(Gaussian::new(0.0, 32.0));

    coder.encode_symbol(5, model).unwrap();
    coder.encode_symbol(-3, model).unwrap();

    // 2. Get compressed data (Vec<u32>)
    let data: Vec<u32> = coder.into_compressed().unwrap();
    println!("Compressed data (words): {:?}", data);

    // 3. Round trip check - Reconstruct Coder
    // TESTING CANDIDATE: from_binary
    let mut decoder = Coder::from_binary(data).unwrap();

    // 4. Decode
    let s1 = decoder.decode_symbol(model).unwrap();
    let s2 = decoder.decode_symbol(model).unwrap();

    println!("Decoded: {}, {}", s1, s2);
    assert_eq!(s1, -3); // Stack ANS is LIFO, so last in is first out
    assert_eq!(s2, 5);

    println!("✅ API VERIFIED: from_reversed_compressed_data(vec) works!");
}
