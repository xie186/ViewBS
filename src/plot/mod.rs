pub mod bis_non_conv_rate;
pub mod global_meth_lev;
pub mod meth_coverage;
pub mod meth_geno;
pub mod meth_heatmap;
pub mod meth_lev_dist;
pub mod meth_one_region;
pub mod meth_over_region;

#[cfg(feature = "plots")]
use crate::api::PlotOptions;
#[cfg(feature = "plots")]
use crate::errors::{Result, ViewBsError};

#[cfg(feature = "plots")]
pub(crate) fn validate_plot_options(options: &PlotOptions) -> Result<()> {
    if options.width_cm.is_finite()
        && options.width_cm > 0.0
        && options.height_cm.is_finite()
        && options.height_cm > 0.0
    {
        Ok(())
    } else {
        Err(ViewBsError::invalid_input(
            "<plot>",
            "plot width and height must be finite positive centimeter values",
        ))
    }
}
