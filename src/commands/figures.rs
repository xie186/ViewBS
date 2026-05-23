#[cfg(feature = "plots")]
use std::fs;
#[cfg(feature = "plots")]
use std::path::Path;

use crate::api::{CommandOutput, MergeFiguresArgs};
#[cfg(feature = "plots")]
use crate::api::{CommandSummary, OutputFile, OutputKind};
use crate::errors::{Result, ViewBsError};

#[cfg(feature = "plots")]
const DEFAULT_PX_PER_CM: f64 = 96.0 / 2.54;

pub fn merge_figures(args: MergeFiguresArgs) -> Result<CommandOutput> {
    #[cfg(feature = "plots")]
    {
        merge_figures_with_plots(args)
    }

    #[cfg(not(feature = "plots"))]
    {
        let _ = args;
        Err(ViewBsError::PlotError {
            message: "merge-figures requires the `plots` feature".to_string(),
        })
    }
}

#[cfg(feature = "plots")]
fn merge_figures_with_plots(args: MergeFiguresArgs) -> Result<CommandOutput> {
    validate_args(&args)?;
    create_output_parent(&args.output)?;

    let labels = labels_for(&args);
    let svg = compose_svg_grid(&args, &labels)?;
    let format = output_format(&args.output)?;

    match format {
        "svg" => {
            fs::write(&args.output, svg).map_err(|source| ViewBsError::io(&args.output, source))?;
        }
        "pdf" => {
            let pdf = svg_to_pdf(&svg)?;
            fs::write(&args.output, pdf).map_err(|source| ViewBsError::io(&args.output, source))?;
        }
        _ => unreachable!("output_format only returns supported formats"),
    }

    Ok(CommandOutput {
        tables: Vec::new(),
        plots: vec![OutputFile {
            kind: OutputKind::Plot,
            path: args.output,
            format: format.to_string(),
        }],
        logs: Vec::new(),
        summary: CommandSummary {
            command: "MergeFigures".to_string(),
            samples: 0,
            records_read: args.inputs.len() as u64,
            records_used: args.inputs.len() as u64,
        },
    })
}

#[cfg(feature = "plots")]
fn validate_args(args: &MergeFiguresArgs) -> Result<()> {
    if args.inputs.is_empty() {
        return Err(ViewBsError::invalid_input(
            "--input",
            "merge-figures requires at least one SVG input",
        ));
    }
    if args.ncol == 0 {
        return Err(ViewBsError::invalid_input(
            "--ncol",
            "merge-figures requires --ncol to be at least 1",
        ));
    }
    if args.base_height_cm <= 0.0 || !args.base_height_cm.is_finite() {
        return Err(ViewBsError::invalid_input(
            "--base-height",
            "merge-figures requires a positive finite base height",
        ));
    }
    if args.base_aspect_ratio <= 0.0 || !args.base_aspect_ratio.is_finite() {
        return Err(ViewBsError::invalid_input(
            "--base-aspect-ratio",
            "merge-figures requires a positive finite aspect ratio",
        ));
    }
    if !args.labels.is_empty() && args.labels.len() != args.inputs.len() {
        return Err(ViewBsError::invalid_input(
            "--labels",
            "number of labels must match number of inputs",
        ));
    }
    output_format(&args.output)?;
    Ok(())
}

#[cfg(feature = "plots")]
fn compose_svg_grid(args: &MergeFiguresArgs, labels: &[String]) -> Result<String> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine as _;

    let cell_height = args.base_height_cm * DEFAULT_PX_PER_CM;
    let cell_width = cell_height * args.base_aspect_ratio;
    let rows = args.inputs.len().div_ceil(args.ncol);
    let width = cell_width * args.ncol as f64;
    let height = cell_height * rows as f64;

    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{width:.2}" height="{height:.2}" viewBox="0 0 {width:.2} {height:.2}" data-viewbs-command="merge-figures">"#
    ));
    svg.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);

    for (index, input) in args.inputs.iter().enumerate() {
        let source = fs::read_to_string(input).map_err(|source| ViewBsError::io(input, source))?;
        validate_svg_input(input, &source)?;
        let encoded = STANDARD.encode(source.as_bytes());
        let row = index / args.ncol;
        let col = index % args.ncol;
        let x = col as f64 * cell_width;
        let y = row as f64 * cell_height;
        svg.push_str(&format!(
            r##"<g transform="translate({x:.2},{y:.2})"><image width="{cell_width:.2}" height="{cell_height:.2}" preserveAspectRatio="xMidYMid meet" href="data:image/svg+xml;base64,{encoded}"/><text x="8" y="22" font-family="DejaVu Sans, Arial, sans-serif" font-size="20" font-weight="700" fill="#111">{}</text></g>"##,
            escape_text(&labels[index])
        ));
    }

    svg.push_str("</svg>");
    Ok(svg)
}

#[cfg(feature = "plots")]
fn validate_svg_input(path: &Path, svg: &str) -> Result<()> {
    let mut options = svg2pdf::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    svg2pdf::usvg::Tree::from_str(svg, &options).map_err(|source| {
        ViewBsError::invalid_input(path, format!("expected a valid SVG input: {source}"))
    })?;
    Ok(())
}

#[cfg(feature = "plots")]
fn svg_to_pdf(svg: &str) -> Result<Vec<u8>> {
    let mut options = svg2pdf::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree =
        svg2pdf::usvg::Tree::from_str(svg, &options).map_err(|source| ViewBsError::PlotError {
            message: format!("failed to parse merged SVG before PDF conversion: {source}"),
        })?;
    svg2pdf::to_pdf(
        &tree,
        svg2pdf::ConversionOptions::default(),
        svg2pdf::PageOptions::default(),
    )
    .map_err(|source| ViewBsError::PlotError {
        message: format!("failed to write merged PDF: {source}"),
    })
}

#[cfg(feature = "plots")]
fn labels_for(args: &MergeFiguresArgs) -> Vec<String> {
    if !args.labels.is_empty() {
        return args.labels.clone();
    }

    (0..args.inputs.len()).map(default_label).collect()
}

#[cfg(feature = "plots")]
fn default_label(mut index: usize) -> String {
    let mut label = String::new();
    loop {
        let rem = index % 26;
        label.insert(0, (b'A' + rem as u8) as char);
        if index < 26 {
            break;
        }
        index = index / 26 - 1;
    }
    label
}

#[cfg(feature = "plots")]
fn output_format(path: &Path) -> Result<&'static str> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("svg") => Ok("svg"),
        Some("pdf") => Ok("pdf"),
        Some(other) => Err(ViewBsError::invalid_input(
            path,
            format!("unsupported merge-figures output extension `{other}`; use .svg or .pdf"),
        )),
        None => Err(ViewBsError::invalid_input(
            path,
            "merge-figures output must end in .svg or .pdf",
        )),
    }
}

#[cfg(feature = "plots")]
fn create_output_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| ViewBsError::io(parent, source))?;
    }
    Ok(())
}

#[cfg(feature = "plots")]
fn escape_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
