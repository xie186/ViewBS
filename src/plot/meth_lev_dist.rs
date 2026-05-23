#[cfg(feature = "plots")]
use std::collections::HashMap;
#[cfg(feature = "plots")]
use std::fs;
#[cfg(feature = "plots")]
use std::path::PathBuf;

use crate::api::{OutputFile, PlotOptions};
#[cfg(feature = "plots")]
use crate::api::{OutputKind, PlotFormat};
use crate::commands::meth_lev_dist::MethLevDistTable;
use crate::errors::{Result, ViewBsError};

pub(crate) fn write_plot(
    table: &MethLevDistTable,
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
fn write_plot_with_kuva(table: &MethLevDistTable, options: &PlotOptions) -> Result<OutputFile> {
    use kuva::prelude::*;

    super::validate_plot_options(options)?;
    let (plots, layout) = build_dist_plot(table, options);
    let path = table
        .path
        .with_extension(options.format.extension())
        .to_path_buf();
    write_kuva_output(plots, layout, &path, options.format)?;

    if options.keep_svg && !matches!(options.format, PlotFormat::Svg) {
        let svg_path = table.path.with_extension("svg");
        let (svg_plots, svg_layout) = build_dist_plot(table, options);
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
fn build_dist_plot(
    table: &MethLevDistTable,
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

    let mut groups = Vec::new();
    let mut series = Vec::new();
    let mut values: HashMap<(String, String), f64> = HashMap::new();

    for row in &table.rows {
        let group = row.midpoint.to_string();
        let label = format!("{}-{}", row.sample, row.context.as_str());
        if !groups.contains(&group) {
            groups.push(group.clone());
        }
        if !series.contains(&label) {
            series.push(label.clone());
        }
        values.insert((group, label), row.percentage);
    }

    let mut bar = BarPlot::new();
    for group in &groups {
        let group_values = series
            .iter()
            .enumerate()
            .map(|(index, label)| {
                let value = values
                    .get(&(group.clone(), label.clone()))
                    .copied()
                    .unwrap_or_default();
                (value, colors[index % colors.len()])
            })
            .collect::<Vec<_>>();
        bar = bar.with_group(group.as_str(), group_values);
    }

    let legend = series.iter().map(String::as_str).collect::<Vec<_>>();
    let plots: Vec<Plot> = vec![bar.with_legend(legend).into()];
    let layout = Layout::auto_from_plots(&plots)
        .with_title("Methylation level distribution")
        .with_x_label("Methylation level bin midpoint")
        .with_y_label("Percentage")
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
