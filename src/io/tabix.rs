use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};

use noodles_bgzf as bgzf;
use noodles_csi as csi;
use noodles_tabix as tabix;

use crate::errors::{Result, ViewBsError};
use crate::io::methyl_report::parse_methyl_record;
use crate::meth::MethylRecord;

pub struct IndexedMethylReader {
    path: PathBuf,
    reader: IndexedMethylReaderInner,
}

enum IndexedMethylReaderInner {
    Tabix(csi::io::IndexedReader<bgzf::io::Reader<File>, tabix::Index>),
    Csi(csi::io::IndexedReader<bgzf::io::Reader<File>, csi::Index>),
}

impl IndexedMethylReader {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let reader = if associated_index_path(&path, "tbi").exists() {
            let index = tabix::fs::read(associated_index_path(&path, "tbi"))
                .map_err(|error| indexed_query_error(&path, "<index>", error))?;
            let file =
                File::open(&path).map_err(|error| indexed_query_error(&path, "<index>", error))?;
            IndexedMethylReaderInner::Tabix(csi::io::IndexedReader::new(file, index))
        } else if associated_index_path(&path, "csi").exists() {
            let index = csi::fs::read(associated_index_path(&path, "csi"))
                .map_err(|error| indexed_query_error(&path, "<index>", error))?;
            let file =
                File::open(&path).map_err(|error| indexed_query_error(&path, "<index>", error))?;
            IndexedMethylReaderInner::Csi(csi::io::IndexedReader::new(file, index))
        } else {
            return Err(indexed_query_error(
                &path,
                "<index>",
                format!(
                    "missing associated index; expected {} or {}",
                    associated_index_path(&path, "tbi").display(),
                    associated_index_path(&path, "csi").display()
                ),
            ));
        };
        Ok(Self { path, reader })
    }

    pub fn query(&mut self, region: &str) -> Result<Vec<MethylRecord>> {
        let parsed_region = region
            .parse()
            .map_err(|error| indexed_query_error(&self.path, region, error))?;

        match &mut self.reader {
            IndexedMethylReaderInner::Tabix(reader) => {
                let query = reader
                    .query(&parsed_region)
                    .map_err(|error| indexed_query_error(&self.path, region, error))?;
                collect_query_records(&self.path, region, query)
            }
            IndexedMethylReaderInner::Csi(reader) => {
                let query = reader
                    .query(&parsed_region)
                    .map_err(|error| indexed_query_error(&self.path, region, error))?;
                collect_query_records(&self.path, region, query)
            }
        }
    }
}

pub fn query_methyl_records(path: &Path, region: &str) -> Result<Vec<MethylRecord>> {
    let mut reader = IndexedMethylReader::from_path(path)?;
    reader.query(region)
}

fn indexed_query_error(path: &Path, region: &str, error: impl std::fmt::Display) -> ViewBsError {
    ViewBsError::IndexedQueryError {
        path: path.to_path_buf(),
        region: region.to_string(),
        message: error.to_string(),
    }
}

fn collect_query_records<I>(path: &Path, region: &str, query: I) -> Result<Vec<MethylRecord>>
where
    I: Iterator<Item = std::io::Result<csi::io::indexed_records::Record>>,
{
    let mut records = Vec::new();
    for (index, result) in query.enumerate() {
        let record = result.map_err(|error| indexed_query_error(path, region, error))?;
        records.push(parse_methyl_record(
            path,
            index as u64 + 1,
            record.as_ref(),
        )?);
    }
    Ok(records)
}

fn associated_index_path(path: &Path, extension: &str) -> PathBuf {
    let mut value = OsString::from(path.as_os_str());
    value.push(".");
    value.push(extension);
    PathBuf::from(value)
}
