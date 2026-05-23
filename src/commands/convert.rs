use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use crate::api::{
    CommandOutput, CommandSummary, ConvertArgs, ConvertGffArgs, OutputFile, OutputKind,
};
use crate::errors::{Result, ViewBsError};

pub fn convert_bsseeker(args: ConvertArgs) -> Result<CommandOutput> {
    let mut records_read = 0_u64;
    let mut records_used = 0_u64;
    with_output_parent(&args.output)?;

    let input = File::open(&args.input).map_err(|source| ViewBsError::io(&args.input, source))?;
    let mut output = BufWriter::new(
        File::create(&args.output).map_err(|source| ViewBsError::io(&args.output, source))?,
    );

    for (index, line) in BufReader::new(input).lines().enumerate() {
        let line_no = index as u64 + 1;
        let line = line.map_err(|source| ViewBsError::io(&args.input, source))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        records_read += 1;
        let columns = trimmed.split_whitespace().collect::<Vec<_>>();
        if columns.len() < 8 {
            return Err(ViewBsError::parse_error(
                &args.input,
                Some(line_no),
                "BSseeker2 CGmap rows must contain at least 8 columns",
            ));
        }

        let depth = parse_u64(&args.input, line_no, "depth", columns[7])?;
        if depth < args.min_depth {
            continue;
        }
        let methylated = parse_u64(&args.input, line_no, "methylated count", columns[6])?;
        if methylated > depth {
            return Err(ViewBsError::parse_error(
                &args.input,
                Some(line_no),
                "methylated count cannot exceed depth",
            ));
        }
        let unmethylated = depth - methylated;
        let strand = if columns[1] == "C" { "+" } else { "-" };
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            columns[0], columns[2], strand, methylated, unmethylated, columns[3], columns[4]
        )
        .map_err(|source| ViewBsError::io(&args.output, source))?;
        records_used += 1;
    }

    output
        .flush()
        .map_err(|source| ViewBsError::io(&args.output, source))?;
    Ok(converter_output(
        "ConvertBsseeker",
        args.output,
        records_read,
        records_used,
    ))
}

pub fn convert_brat(args: ConvertArgs) -> Result<CommandOutput> {
    let mut records_read = 0_u64;
    let mut records_used = 0_u64;
    with_output_parent(&args.output)?;

    let input = File::open(&args.input).map_err(|source| ViewBsError::io(&args.input, source))?;
    let mut output = BufWriter::new(
        File::create(&args.output).map_err(|source| ViewBsError::io(&args.output, source))?,
    );

    for (index, line) in BufReader::new(input).lines().enumerate() {
        let line_no = index as u64 + 1;
        let line = line.map_err(|source| ViewBsError::io(&args.input, source))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        records_read += 1;
        let columns = trimmed.split('\t').collect::<Vec<_>>();
        if columns.len() < 6 {
            return Err(ViewBsError::parse_error(
                &args.input,
                Some(line_no),
                "BRAT methylation rows must contain at least 6 tab-delimited columns",
            ));
        }

        let (context, depth) = columns[3].split_once(':').ok_or_else(|| {
            ViewBsError::parse_error(
                &args.input,
                Some(line_no),
                "BRAT total column must be formatted as CONTEXT:DEPTH",
            )
        })?;
        let context = context.replace("CpG", "CG");
        let depth = parse_u64(&args.input, line_no, "depth", depth)?;
        if depth < args.min_depth {
            continue;
        }
        let level = parse_f64(&args.input, line_no, "methylation level", columns[4])?;
        let methylated = (depth as f64 * level + 0.5).floor() as u64;
        if methylated > depth {
            return Err(ViewBsError::parse_error(
                &args.input,
                Some(line_no),
                "rounded methylated count cannot exceed depth",
            ));
        }
        let unmethylated = depth - methylated;
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            columns[0], columns[1], columns[5], methylated, unmethylated, context, context
        )
        .map_err(|source| ViewBsError::io(&args.output, source))?;
        records_used += 1;
    }

    output
        .flush()
        .map_err(|source| ViewBsError::io(&args.output, source))?;
    Ok(converter_output(
        "ConvertBrat",
        args.output,
        records_read,
        records_used,
    ))
}

pub fn convert_gff(args: ConvertGffArgs) -> Result<CommandOutput> {
    let mut records_read = 0_u64;
    let mut records_used = 0_u64;
    with_output_parent(&args.output)?;

    let input = File::open(&args.input).map_err(|source| ViewBsError::io(&args.input, source))?;
    let mut output = BufWriter::new(
        File::create(&args.output).map_err(|source| ViewBsError::io(&args.output, source))?,
    );
    let features = args
        .features
        .iter()
        .map(|feature| feature.as_str())
        .collect::<Vec<_>>();

    for (index, line) in BufReader::new(input).lines().enumerate() {
        let line_no = index as u64 + 1;
        let line = line.map_err(|source| ViewBsError::io(&args.input, source))?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        records_read += 1;
        let columns = trimmed.split('\t').collect::<Vec<_>>();
        if columns.len() < 9 {
            return Err(ViewBsError::parse_error(
                &args.input,
                Some(line_no),
                "GFF/GTF rows must contain 9 tab-delimited columns",
            ));
        }
        if !features.is_empty() && !features.contains(&columns[2]) {
            continue;
        }
        let start = parse_u64(&args.input, line_no, "start", columns[3])?;
        let end = parse_u64(&args.input, line_no, "end", columns[4])?;
        if end < start {
            return Err(ViewBsError::parse_error(
                &args.input,
                Some(line_no),
                "feature end cannot be smaller than start",
            ));
        }
        let attributes = parse_attributes(columns[8]);
        let id = attributes
            .get(args.id_attribute.as_str())
            .or_else(|| attributes.get("ID"))
            .or_else(|| attributes.get("Name"))
            .or_else(|| attributes.get("gene_id"))
            .or_else(|| attributes.get("transcript_id"))
            .cloned()
            .unwrap_or_else(|| format!("{}_{}_{}", columns[0], start, end));
        let strand = match columns[6] {
            "+" | "-" => columns[6],
            _ => ".",
        };
        writeln!(
            output,
            "{}\t{}\t{}\t{}\t{}",
            columns[0], start, end, id, strand
        )
        .map_err(|source| ViewBsError::io(&args.output, source))?;
        records_used += 1;
    }

    output
        .flush()
        .map_err(|source| ViewBsError::io(&args.output, source))?;
    Ok(converter_output(
        "ConvertGff",
        args.output,
        records_read,
        records_used,
    ))
}

fn converter_output(
    command: &str,
    output: std::path::PathBuf,
    records_read: u64,
    records_used: u64,
) -> CommandOutput {
    CommandOutput {
        tables: vec![OutputFile {
            kind: OutputKind::Table,
            path: output,
            format: "tab".to_string(),
        }],
        plots: Vec::new(),
        logs: Vec::new(),
        summary: CommandSummary {
            command: command.to_string(),
            samples: 0,
            records_read,
            records_used,
        },
    }
}

fn with_output_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| ViewBsError::io(parent, source))?;
    }
    Ok(())
}

fn parse_u64(path: &Path, line: u64, label: &str, value: &str) -> Result<u64> {
    value.parse::<u64>().map_err(|source| {
        ViewBsError::parse_error(
            path,
            Some(line),
            format!("invalid {label} `{value}`: {source}"),
        )
    })
}

fn parse_f64(path: &Path, line: u64, label: &str, value: &str) -> Result<f64> {
    value.parse::<f64>().map_err(|source| {
        ViewBsError::parse_error(
            path,
            Some(line),
            format!("invalid {label} `{value}`: {source}"),
        )
    })
}

fn parse_attributes(value: &str) -> std::collections::BTreeMap<&str, String> {
    let mut attributes = std::collections::BTreeMap::new();
    for raw_part in value.split(';') {
        let part = raw_part.trim();
        if part.is_empty() {
            continue;
        }
        let (key, raw_value) = if let Some((key, raw_value)) = part.split_once('=') {
            (key.trim(), raw_value.trim())
        } else if let Some((key, raw_value)) = part.split_once(char::is_whitespace) {
            (key.trim(), raw_value.trim())
        } else {
            continue;
        };
        if key.is_empty() {
            continue;
        }
        attributes.insert(key, unquote(raw_value));
    }
    attributes
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_string()
}
