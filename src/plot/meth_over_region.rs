#[cfg(feature = "plots")]
use std::collections::BTreeMap;
#[cfg(feature = "plots")]
use std::fs;
#[cfg(feature = "plots")]
use std::path::PathBuf;

use crate::api::{OutputFile, PlotOptions};
#[cfg(feature = "plots")]
use crate::api::{OutputKind, PlotFormat};
use crate::commands::meth_over_region::MethOverRegionTable;
use crate::errors::{Result, ViewBsError};

pub(crate) fn write_plot(
    table: &MethOverRegionTable,
    options: &PlotOptions,
) -> Result<Option<OutputFile>> {
    #[cfg(feature = "plots")]
    {
        write_plot_with_kuva(table, options).map(Some)
    }

    #[cfg(not(feature = "plots"))]
    {
        let _ = (table, options);
        Err(ViewBsError::PlotError {
            message: "plotting requires the `plots` feature".to_string(),
        })
    }
}

#[cfg(feature = "plots")]
fn write_plot_with_kuva(table: &MethOverRegionTable, options: &PlotOptions) -> Result<OutputFile> {
    use kuva::prelude::*;

    super::validate_plot_options(options)?;
    let (plots, layout) = build_over_region_plot(table, options);
    let path = table
        .path
        .with_extension(options.format.extension())
        .to_path_buf();
    write_kuva_output(plots, layout, &path, options.format)?;

    if options.keep_svg && !matches!(options.format, PlotFormat::Svg) {
        let svg_path = table.path.with_extension("svg");
        let (svg_plots, svg_layout) = build_over_region_plot(table, options);
        let svg = render_to_svg(svg_plots, svg_layout);
        fs::write(&svg_path, svg).map_err(|source| ViewBsError::io(&svg_path, source))?;
    }

    Ok(OutputFile {
        kind: OutputKind::Plot,
        path,
        format: options.format.extension().to_string(),
    })
}

#[cfg(feature = "plots")]
fn build_over_region_plot(
    table: &MethOverRegionTable,
    options: &PlotOptions,
) -> (Vec<kuva::prelude::Plot>, kuva::prelude::Layout) {
    use kuva::prelude::*;

    let colors = [
        "steelblue",
        "tomato",
        "seagreen",
        "goldenrod",
        "mediumpurple",
        "darkcyan",
        "gray40",
        "firebrick",
        "darkorange",
    ];

    let mut by_sample: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for row in &table.rows {
        by_sample
            .entry(row.sample.clone())
            .or_default()
            .push((row.bin as f64, row.level));
    }

    let plots = by_sample
        .into_iter()
        .enumerate()
        .map(|(index, (sample, mut points))| {
            points.sort_by(|a, b| a.0.total_cmp(&b.0));
            Plot::Line(
                LinePlot::new()
                    .with_data(points)
                    .with_color(colors[index % colors.len()])
                    .with_stroke_width(1.5)
                    .with_legend(sample),
            )
        })
        .collect::<Vec<_>>();

    let layout = Layout::auto_from_plots(&plots)
        .with_title(format!("MethOverRegion {}", table.context.as_str()))
        .with_x_label(table.region_name.as_str())
        .with_y_label("Methylation level")
        .with_width(cm_to_px(options.width_cm))
        .with_height(cm_to_px(options.height_cm));

    (plots, layout)
}

#[cfg(feature = "plots")]
fn write_kuva_output(
    plots: Vec<kuva::prelude::Plot>,
    layout: kuva::prelude::Layout,
    path: &PathBuf,
    format: PlotFormat,
) -> Result<()> {
    use kuva::prelude::*;

    match format {
        PlotFormat::Svg => {
            let svg = render_to_svg(plots, layout);
            fs::write(path, svg).map_err(|source| ViewBsError::io(path, source))?;
        }
        PlotFormat::Pdf => {
            let pdf = render_to_pdf(plots, layout).map_err(|error| ViewBsError::PlotError {
                message: error.to_string(),
            })?;
            fs::write(path, pdf).map_err(|source| ViewBsError::io(path, source))?;
        }
        PlotFormat::Png => {
            let png =
                render_to_png(plots, layout, 2.0).map_err(|error| ViewBsError::PlotError {
                    message: error.to_string(),
                })?;
            fs::write(path, png).map_err(|source| ViewBsError::io(path, source))?;
        }
    }
    Ok(())
}

#[cfg(feature = "plots")]
fn cm_to_px(cm: f64) -> f64 {
    cm / 2.54 * 96.0
}
