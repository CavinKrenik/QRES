// Probe Constriction API
use constriction::stream::stack::DefaultAnsCoder;
use constriction::stream::model::DefaultLeakyQuantizer; 
// Try other paths if this fails:
// use constriction::symbol::DefaultLeakyQuantizer;
// use constriction::models::DefaultLeakyQuantizer;

fn main() {
    let _coder = DefaultAnsCoder::new();
    let _quant = DefaultLeakyQuantizer::new(-128..=127, 24, 2);
}
