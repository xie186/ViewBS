use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use flate2::read::MultiGzDecoder;

use crate::errors::{Result, ViewBsError};
use crate::meth::{Context, MethylRecord};

pub struct MethylReportReader {
    path: PathBuf,
    line_no: u64,
    reader: Box<dyn BufRead>,
}

impl MethylReportReader {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = File::open(&path).map_err(|source| ViewBsError::io(&path, source))?;
        let reader: Box<dyn Read> = if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("gz"))
        {
            Box::new(MultiGzDecoder::new(file))
        } else {
            Box::new(file)
        };

        Ok(Self {
            path,
            line_no: 0,
            reader: Box::new(BufReader::new(reader)),
        })
    }
}

impl Iterator for MethylReportReader {
    type Item = Result<MethylRecord>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        match self.reader.read_line(&mut line) {
            Ok(0) => None,
            Ok(_) => {
                self.line_no += 1;
                Some(parse_methyl_record(&self.path, self.line_no, &line))
            }
            Err(source) => Some(Err(ViewBsError::io(&self.path, source))),
        }
    }
}

pub fn parse_methyl_record(path: &Path, line_no: u64, line: &str) -> Result<MethylRecord> {
    let columns: Vec<&str> = line.trim_end().split('\t').collect();
    if columns.len() < 7 {
        return Err(ViewBsError::parse_error(
            path,
            Some(line_no),
            format!(
                "expected at least 7 tab-delimited columns, found {}",
                columns.len()
            ),
        ));
    }

    let pos = parse_u64(path, line_no, columns[1], "position")?;
    let methylated = parse_u64(path, line_no, columns[3], "count methylated")?;
    let unmethylated = parse_u64(path, line_no, columns[4], "count unmethylated")?;
    let strand = columns[2]
        .chars()
        .next()
        .ok_or_else(|| ViewBsError::parse_error(path, Some(line_no), "empty strand column"))?;

    Ok(MethylRecord {
        chrom: columns[0].to_string(),
        pos,
        strand,
        methylated,
        unmethylated,
        context: Context::from(columns[5]),
        trinuc: columns[6].to_string(),
    })
}

fn parse_u64(path: &Path, line_no: u64, value: &str, label: &str) -> Result<u64> {
    value.parse::<u64>().map_err(|source| {
        ViewBsError::parse_error(
            path,
            Some(line_no),
            format!("invalid {label} value `{value}`: {source}"),
        )
    })
}
