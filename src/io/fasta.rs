use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::errors::{Result, ViewBsError};
use crate::meth::Context;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct ReferenceContextCounts {
    pub c_or_g: u64,
    pub cg: u64,
    pub chg: u64,
    pub chh: u64,
}

impl ReferenceContextCounts {
    pub fn as_context_map(self) -> HashMap<Context, u64> {
        HashMap::from([
            (Context::Cg, self.cg),
            (Context::Chg, self.chg),
            (Context::Chh, self.chh),
        ])
    }

    fn add_assign(&mut self, other: Self) {
        self.c_or_g += other.c_or_g;
        self.cg += other.cg;
        self.chg += other.chg;
        self.chh += other.chh;
    }
}

pub fn count_reference_contexts(path: impl AsRef<Path>) -> Result<ReferenceContextCounts> {
    let path = path.as_ref().to_path_buf();
    let file = File::open(&path).map_err(|source| ViewBsError::io(&path, source))?;
    let reader = BufReader::new(file);
    let mut counts = ReferenceContextCounts::default();
    let mut current = String::new();

    for line in reader.lines() {
        let line = line.map_err(|source| ViewBsError::io(&path, source))?;
        if line.starts_with('>') {
            add_sequence_counts(&mut counts, &current);
            current.clear();
        } else {
            current.push_str(line.trim());
        }
    }
    add_sequence_counts(&mut counts, &current);

    Ok(counts)
}

fn add_sequence_counts(total: &mut ReferenceContextCounts, sequence: &str) {
    if sequence.is_empty() {
        return;
    }
    total.add_assign(count_sequence_contexts(sequence));
}

pub fn count_sequence_contexts(sequence: &str) -> ReferenceContextCounts {
    let seq = sequence.to_ascii_uppercase();
    let bytes = seq.as_bytes();
    let c_or_g = bytes
        .iter()
        .filter(|base| matches!(base, b'C' | b'G'))
        .count() as u64;
    let cg_sites = count_pattern(bytes, b"CG") * 2;
    let chg_sites = 2
        * (count_pattern(bytes, b"CAG")
            + count_pattern(bytes, b"CTG")
            + count_pattern(bytes, b"CCG"));
    let chh_sites = c_or_g.saturating_sub(cg_sites + chg_sites);

    ReferenceContextCounts {
        c_or_g,
        cg: cg_sites,
        chg: chg_sites,
        chh: chh_sites,
    }
}

fn count_pattern(sequence: &[u8], pattern: &[u8]) -> u64 {
    if pattern.is_empty() || sequence.len() < pattern.len() {
        return 0;
    }
    sequence
        .windows(pattern.len())
        .filter(|window| *window == pattern)
        .count() as u64
}

pub fn ensure_reference_exists(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        Ok(path.to_path_buf())
    } else {
        Err(ViewBsError::invalid_input(
            path,
            "reference FASTA does not exist",
        ))
    }
}
