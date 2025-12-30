use serde::{Serialize, Deserialize};

// FALLBACK: Using Bincode until Constriction API is aligned.
// This allows verification of the Mixer/iPEPS logic.
// PERFORMANCE WARNING: Ratios will be > 1.0.

pub struct AnsWriter {
    residuals: Vec<i8>,
}

impl AnsWriter {
    pub fn new() -> Self {
        AnsWriter { residuals: Vec::with_capacity(1024) }
    }

    pub fn write_residual(&mut self, residual: i8) {
        self.residuals.push(residual);
    }

    pub fn finish(self) -> Vec<u8> {
        // Serialize
        bincode::serialize(&self.residuals).unwrap()
    }
}

pub struct AnsReader<'a> {
    residuals: std::vec::IntoIter<i8>,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> AnsReader<'a> {
    pub fn new(data: &[u8]) -> Self {
        let residuals: Vec<i8> = bincode::deserialize(data).unwrap_or_default();
        AnsReader {
            residuals: residuals.into_iter(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn read_residual(&mut self) -> i8 {
        self.residuals.next().unwrap_or(0)
    }
}