#[cfg(feature = "plots")]
use std::fs;
#[cfg(feature = "plots")]
use std::path::PathBuf;

use crate::api::{OutputFile, PlotOptions};
#[cfg(feature = "plots")]
use crate::api::{OutputKind, PlotFormat};
use crate::commands::global_meth_lev::GlobalMethLevTable;
use crate::errors::{Result, ViewBsError};

pub(crate) fn write_plot(
    table: &GlobalMethLevTable,
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
fn write_plot_with_kuva(table: &GlobalMethLevTable, options: &PlotOptions) -> Result<OutputFile> {
    use kuva::prelude::*;

    super::validate_plot_options(options)?;
    let (plots, layout) = build_global_plot(table, options);
    let path = table
        .path
        .with_extension(options.format.extension())
        .to_path_buf();
    write_kuva_output(plots, layout, &path, options.format)?;

    if options.keep_svg && !matches!(options.format, PlotFormat::Svg) {
        let svg_path = table.path.with_extension("svg");
        let (svg_plots, svg_layout) = build_global_plot(table, options);
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
fn build_global_plot(
    table: &GlobalMethLevTable,
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
    ];

    let mut bar = BarPlot::new();
    for context in &table.contexts {
        let values = table
            .sample_order
            .iter()
            .enumerate()
            .map(|(index, sample)| {
                let value = table
                    .values
                    .get(sample)
                    .and_then(|values| values.get(context))
                    .copied()
                    .unwrap_or(0.0);
                (value, colors[index % colors.len()])
            })
            .collect::<Vec<_>>();
        bar = bar.with_group(context.as_str(), values);
    }

    let legend = table
        .sample_order
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let bar = bar.with_legend(legend);
    let plots: Vec<Plot> = vec![bar.into()];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Global methylation level")
        .with_x_label("Context")
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
