// Probe Constriction API Variants
use constriction::stream::{stack::DefaultAnsCoder, Decode, Encode};

fn main() {
    let data: Vec<u32> = vec![0, 1, 2];
    
    // Variant 1: from_binary 
    // let _coder1 = DefaultAnsCoder::from_binary(data.clone()).unwrap();
    
    // Variant 2: from_compressed_data 
    // let _coder2 = DefaultAnsCoder::from_compressed_data(data.clone()).unwrap();
    
    // Variant 3: from_reversed_compressed_data
    // let _coder3 = DefaultAnsCoder::from_reversed_compressed_data(data.clone()).unwrap();
    
    // I'll try just ONE that I suspect most: `from_compressed` maybe?
    // Or I'll use a `compile_error!` approach or just invalid call to list methods?
    // Let's try calling a non-existent method `list_methods()` to see "did you mean...?"
    let _coder = DefaultAnsCoder::list_methods();
}
