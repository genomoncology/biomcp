use super::{ChartRenderOptions, OutputTarget, ensure_non_empty, output_target};
use crate::error::BioMcpError;

const DEFAULT_IMAGE_WIDTH: u32 = 800;
const DEFAULT_IMAGE_HEIGHT: u32 = 600;
const MAX_PNG_PIXELS: u64 = 16_777_216;

pub(crate) fn numeric_range(values: &[f64]) -> Result<(f64, f64), BioMcpError> {
    ensure_non_empty(values, "Histogram")?;
    let min = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if min == max {
        Ok((min - 0.5, max + 0.5))
    } else {
        Ok((min, max))
    }
}

pub(crate) fn suggest_bins(sample_count: usize) -> usize {
    ((sample_count as f64).sqrt().round() as usize).clamp(5, 20)
}

fn checked_png_pixels(width: f64, height: f64, scale: f32) -> Result<u64, BioMcpError> {
    let scaled_width = (width * f64::from(scale)).ceil();
    let scaled_height = (height * f64::from(scale)).ceil();
    if !scaled_width.is_finite()
        || !scaled_height.is_finite()
        || scaled_width < 1.0
        || scaled_height < 1.0
        || scaled_width > u32::MAX.into()
        || scaled_height > u32::MAX.into()
    {
        return Err(BioMcpError::InvalidArgument(
            "PNG output dimensions are invalid".into(),
        ));
    }
    let width = scaled_width as u64;
    let height = scaled_height as u64;
    width.checked_mul(height).ok_or_else(|| {
        BioMcpError::InvalidArgument(format!("PNG output is limited to {MAX_PNG_PIXELS} pixels"))
    })
}

pub(super) fn validate_png_pixels(width: f64, height: f64, scale: f32) -> Result<(), BioMcpError> {
    if checked_png_pixels(width, height, scale)? > MAX_PNG_PIXELS {
        return Err(BioMcpError::InvalidArgument(format!(
            "PNG output is limited to {MAX_PNG_PIXELS} pixels"
        )));
    }
    Ok(())
}

pub(crate) fn validate_chart_output_options(
    options: &ChartRenderOptions,
) -> Result<(), BioMcpError> {
    if let OutputTarget::Png { scale, .. } = output_target(options)? {
        validate_png_pixels(
            f64::from(options.width.unwrap_or(DEFAULT_IMAGE_WIDTH)),
            f64::from(options.height.unwrap_or(DEFAULT_IMAGE_HEIGHT)),
            scale,
        )?;
    }
    Ok(())
}
