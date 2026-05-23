use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::api::SampleSpec;
use crate::errors::{Result, ViewBsError};

pub fn parse_sample_args(values: &[String]) -> Result<Vec<SampleSpec>> {
    if values.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            "at least one --sample value is required",
        ));
    }

    if values.iter().any(|value| value.starts_with("file:")) {
        if values.len() != 1 {
            return Err(ViewBsError::invalid_input(
                "<sample>",
                "only one --sample value is allowed when using file:<sample-list>",
            ));
        }
        let list_path = values[0].trim_start_matches("file:");
        return parse_sample_file(list_path);
    }

    values
        .iter()
        .map(|value| parse_inline_sample(value))
        .collect()
}

fn parse_inline_sample(value: &str) -> Result<SampleSpec> {
    let (path, name) = value.split_once(',').ok_or_else(|| {
        ViewBsError::invalid_input(
            "<sample>",
            format!("sample `{value}` must be formatted as methylation_file,sample_name"),
        )
    })?;

    if path.is_empty() || name.is_empty() {
        return Err(ViewBsError::invalid_input(
            "<sample>",
            format!("sample `{value}` must include both path and name"),
        ));
    }

    Ok(SampleSpec {
        path: PathBuf::from(path),
        name: name.to_string(),
        region_path: None,
    })
}

fn parse_sample_file(path: impl AsRef<Path>) -> Result<Vec<SampleSpec>> {
    let path = path.as_ref().to_path_buf();
    let file = File::open(&path).map_err(|source| ViewBsError::io(&path, source))?;
    let reader = BufReader::new(file);
    let mut samples = Vec::new();

    for (index, line) in reader.lines().enumerate() {
        let line_no = index as u64 + 1;
        let line = line.map_err(|source| ViewBsError::io(&path, source))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let columns: Vec<&str> = trimmed.split_whitespace().collect();
        if columns.len() < 2 {
            return Err(ViewBsError::parse_error(
                &path,
                Some(line_no),
                "sample list rows must contain at least methylation_file and sample_name",
            ));
        }
        samples.push(SampleSpec {
            path: PathBuf::from(columns[0]),
            name: columns[1].to_string(),
            region_path: columns.get(2).map(|value| PathBuf::from(*value)),
        });
    }

    if samples.is_empty() {
        return Err(ViewBsError::invalid_input(
            path,
            "sample list did not contain any usable samples",
        ));
    }

    Ok(samples)
}
