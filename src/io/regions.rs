use std::str::FromStr;
use std::{fs::File, io::BufRead, io::BufReader, path::Path};

use crate::errors::{Result, ViewBsError};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GenomicRegion {
    pub chrom: String,
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RegionRecord {
    pub chrom: String,
    pub start: u64,
    pub end: u64,
    pub name: Option<String>,
    pub strand: char,
}

impl RegionRecord {
    pub fn to_genomic_region(&self) -> GenomicRegion {
        GenomicRegion {
            chrom: self.chrom.clone(),
            start: self.start,
            end: self.end,
        }
    }

    pub fn len(&self) -> u64 {
        self.end - self.start + 1
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl GenomicRegion {
    pub fn with_flank(&self, flank: u64) -> Self {
        Self {
            chrom: self.chrom.clone(),
            start: self.start.saturating_sub(flank).max(1),
            end: self.end.saturating_add(flank),
        }
    }

    pub fn to_tabix_region(&self) -> String {
        format!("{}:{}-{}", self.chrom, self.start, self.end)
    }
}

impl FromStr for GenomicRegion {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        let (chrom, range) = value
            .split_once(':')
            .ok_or_else(|| "expected region format chr:start-end".to_string())?;
        let (start, end) = range
            .split_once('-')
            .ok_or_else(|| "expected region format chr:start-end".to_string())?;
        if chrom.is_empty() {
            return Err("chromosome cannot be empty".to_string());
        }
        let start = start
            .parse::<u64>()
            .map_err(|source| format!("invalid region start `{start}`: {source}"))?;
        let end = end
            .parse::<u64>()
            .map_err(|source| format!("invalid region end `{end}`: {source}"))?;
        if start == 0 {
            return Err("region start must be >= 1".to_string());
        }
        if end < start {
            return Err("region end must be >= start".to_string());
        }
        Ok(Self {
            chrom: chrom.to_string(),
            start,
            end,
        })
    }
}

pub fn read_region_file(path: &Path) -> Result<Vec<GenomicRegion>> {
    read_region_records(path).map(|regions| {
        regions
            .into_iter()
            .map(|region| region.to_genomic_region())
            .collect()
    })
}

pub fn read_region_records(path: &Path) -> Result<Vec<RegionRecord>> {
    let file = File::open(path).map_err(|source| ViewBsError::io(path, source))?;
    let reader = BufReader::new(file);
    let mut regions = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line = line.map_err(|source| ViewBsError::io(path, source))?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let columns = line.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 3 {
            return Err(ViewBsError::parse_error(
                path,
                Some(index as u64 + 1),
                "expected at least chromosome, start, and end columns",
            ));
        }
        let start = columns[1].parse::<u64>().map_err(|source| {
            ViewBsError::parse_error(
                path,
                Some(index as u64 + 1),
                format!("invalid region start `{}`: {source}", columns[1]),
            )
        })?;
        let end = columns[2].parse::<u64>().map_err(|source| {
            ViewBsError::parse_error(
                path,
                Some(index as u64 + 1),
                format!("invalid region end `{}`: {source}", columns[2]),
            )
        })?;
        if start == 0 {
            return Err(ViewBsError::parse_error(
                path,
                Some(index as u64 + 1),
                "region start must be >= 1",
            ));
        }
        if end < start {
            return Err(ViewBsError::parse_error(
                path,
                Some(index as u64 + 1),
                "region end must be >= start",
            ));
        }
        let name = columns.get(3).map(|value| (*value).to_string());
        let strand = columns
            .get(4)
            .and_then(|value| value.chars().next())
            .unwrap_or('+');
        regions.push(RegionRecord {
            chrom: columns[0].to_string(),
            start,
            end,
            name,
            strand,
        });
    }

    Ok(regions)
}
