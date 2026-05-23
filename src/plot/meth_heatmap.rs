#[cfg(feature = "plots")]
use std::fs;
#[cfg(feature = "plots")]
use std::path::{Path, PathBuf};

use crate::api::{OutputFile, PlotOptions};
#[cfg(feature = "plots")]
use crate::api::{OutputKind, PlotFormat};
use crate::commands::meth_heatmap::MethHeatmapTable;
use crate::errors::{Result, ViewBsError};

pub(crate) fn write_plots(
    table: &MethHeatmapTable,
    heatmap_options: &PlotOptions,
    distribution_options: &PlotOptions,
) -> Result<Vec<OutputFile>> {
    #[cfg(feature = "plots")]
    {
        let mut outputs = Vec::new();
        let complete = complete_matrix(table);
        if complete.data.is_empty() {
            return Ok(outputs);
        }
        if heatmap_options.enabled {
            outputs.push(write_heatmap(table, &complete, heatmap_options)?);
        }
        if distribution_options.enabled {
            outputs.push(write_distribution(table, &complete, distribution_options)?);
        }
        Ok(outputs)
    }

    #[cfg(not(feature = "plots"))]
    {
        let _ = (table, heatmap_options, distribution_options);
        Err(ViewBsError::PlotError {
            message: "plotting requires the `plots` feature".to_string(),
        })
    }
}

#[cfg(feature = "plots")]
struct CompleteMatrix {
    data: Vec<Vec<f64>>,
    row_labels: Vec<String>,
    columns: Vec<String>,
}

#[cfg(feature = "plots")]
fn complete_matrix(table: &MethHeatmapTable) -> CompleteMatrix {
    let mut data = Vec::new();
    let mut row_labels = Vec::new();
    let limit = table.random_region;

    for row in &table.rows {
        if data.len() >= limit {
            break;
        }
        let values = row.values.iter().copied().collect::<Option<Vec<_>>>();
        if let Some(values) = values {
            data.push(values);
            row_labels.push(row.id.clone());
        }
    }

    CompleteMatrix {
        data,
        row_labels,
        columns: table.columns.clone(),
    }
}

#[cfg(feature = "plots")]
fn write_heatmap(
    table: &MethHeatmapTable,
    complete: &CompleteMatrix,
    options: &PlotOptions,
) -> Result<OutputFile> {
    use kuva::prelude::*;

    super::validate_plot_options(options)?;
    let cluster_rows = table.cluster_rows && complete.data.len() > 1;
    let cluster_cols = table.cluster_cols && complete.columns.len() > 1;
    let plot = Clustermap::new()
        .with_data(complete.data.clone())
        .with_row_labels(complete.row_labels.clone())
        .with_col_labels(complete.columns.clone())
        .with_cluster_rows(cluster_rows)
        .with_cluster_cols(cluster_cols)
        .with_color_map(ColorMap::Viridis)
        .with_legend("Methylation level");

    let plots = vec![Plot::Clustermap(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(format!("MethHeatmap {}", table.label))
        .with_width(cm_to_px(options.width_cm))
        .with_height(cm_to_px(options.height_cm));

    let path = table
        .path
        .with_extension(options.format.extension())
        .to_path_buf();
    write_kuva_output(plots, layout, &path, options.format)?;

    if options.keep_svg && !matches!(options.format, PlotFormat::Svg) {
        let svg_path = table.path.with_extension("svg");
        let (svg_plots, svg_layout) = build_heatmap_svg(table, complete, options);
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
fn build_heatmap_svg(
    table: &MethHeatmapTable,
    complete: &CompleteMatrix,
    options: &PlotOptions,
) -> (Vec<kuva::prelude::Plot>, kuva::prelude::Layout) {
    use kuva::prelude::*;

    let cluster_rows = table.cluster_rows && complete.data.len() > 1;
    let cluster_cols = table.cluster_cols && complete.columns.len() > 1;
    let plot = Clustermap::new()
        .with_data(complete.data.clone())
        .with_row_labels(complete.row_labels.clone())
        .with_col_labels(complete.columns.clone())
        .with_cluster_rows(cluster_rows)
        .with_cluster_cols(cluster_cols)
        .with_color_map(ColorMap::Viridis)
        .with_legend("Methylation level");
    let plots = vec![Plot::Clustermap(plot)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(format!("MethHeatmap {}", table.label))
        .with_width(cm_to_px(options.width_cm))
        .with_height(cm_to_px(options.height_cm));
    (plots, layout)
}

#[cfg(feature = "plots")]
fn write_distribution(
    table: &MethHeatmapTable,
    complete: &CompleteMatrix,
    options: &PlotOptions,
) -> Result<OutputFile> {
    use kuva::prelude::*;

    super::validate_plot_options(options)?;
    let mut violin = ViolinPlot::new()
        .with_color("rgba(70,130,180,0.45)")
        .with_width(24.0);
    let mut boxes = BoxPlot::new()
        .with_color("rgba(80,80,80,0.35)")
        .with_width(0.25);

    for (column_index, column) in complete.columns.iter().enumerate() {
        let values = complete
            .data
            .iter()
            .map(|row| row[column_index])
            .collect::<Vec<_>>();
        violin = violin.with_group(column.clone(), values.clone());
        boxes = boxes.with_group(column.clone(), values);
    }

    let plots = vec![Plot::Violin(violin), Plot::Box(boxes)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(format!("MethHist {}", table.label))
        .with_x_label("Sample")
        .with_y_label("Methylation level")
        .with_width(cm_to_px(options.width_cm))
        .with_height(cm_to_px(options.height_cm));

    let path = distribution_path(&table.path, options.format.extension());
    write_kuva_output(plots, layout, &path, options.format)?;

    if options.keep_svg && !matches!(options.format, PlotFormat::Svg) {
        let svg_path = distribution_path(&table.path, "svg");
        let (svg_plots, svg_layout) = build_distribution_svg(table, complete, options);
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
fn build_distribution_svg(
    table: &MethHeatmapTable,
    complete: &CompleteMatrix,
    options: &PlotOptions,
) -> (Vec<kuva::prelude::Plot>, kuva::prelude::Layout) {
    use kuva::prelude::*;

    let mut violin = ViolinPlot::new()
        .with_color("rgba(70,130,180,0.45)")
        .with_width(24.0);
    let mut boxes = BoxPlot::new()
        .with_color("rgba(80,80,80,0.35)")
        .with_width(0.25);
    for (column_index, column) in complete.columns.iter().enumerate() {
        let values = complete
            .data
            .iter()
            .map(|row| row[column_index])
            .collect::<Vec<_>>();
        violin = violin.with_group(column.clone(), values.clone());
        boxes = boxes.with_group(column.clone(), values);
    }
    let plots = vec![Plot::Violin(violin), Plot::Box(boxes)];
    let layout = Layout::auto_from_plots(&plots)
        .with_title(format!("MethHist {}", table.label))
        .with_x_label("Sample")
        .with_y_label("Methylation level")
        .with_width(cm_to_px(options.width_cm))
        .with_height(cm_to_px(options.height_cm));
    (plots, layout)
}

#[cfg(feature = "plots")]
fn distribution_path(table_path: &Path, extension: &str) -> PathBuf {
    let file_name = table_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("MethHeatmap.txt")
        .replace("_MethHeatmap_", "_MethHist_");
    table_path
        .with_file_name(file_name)
        .with_extension(extension)
        .to_path_buf()
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
