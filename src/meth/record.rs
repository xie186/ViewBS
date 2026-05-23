#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::Context;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct MethylRecord {
    pub chrom: String,
    pub pos: u64,
    pub strand: char,
    pub methylated: u64,
    pub unmethylated: u64,
    pub context: Context,
    pub trinuc: String,
}

impl MethylRecord {
    pub fn depth(&self) -> u64 {
        self.methylated + self.unmethylated
    }
}
