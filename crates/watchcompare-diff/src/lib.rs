use image::{DynamicImage, Rgb, RgbImage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DiffMetrics {
    pub width: u32,
    pub height: u32,
    pub pixel_count: u64,
    pub mean_abs_error: f64,
    pub root_mean_square_error: f64,
    pub max_channel_error: u8,
    pub exact_pixel_fraction: f64,
    pub pixels_above_threshold: u64,
    pub fraction_above_threshold: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct DiffLimits {
    pub threshold: u8,
    pub max_mean_abs_error: f64,
    pub max_fraction_above_threshold: f64,
}

impl Default for DiffLimits {
    fn default() -> Self {
        Self {
            threshold: 8,
            max_mean_abs_error: 2.0,
            max_fraction_above_threshold: 0.01,
        }
    }
}

#[derive(Debug)]
pub enum DiffError {
    DimensionMismatch {
        reference: (u32, u32),
        candidate: (u32, u32),
    },
}

impl std::fmt::Display for DiffError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DimensionMismatch { reference, candidate } => write!(
                f,
                "dimension mismatch: reference={}x{}, candidate={}x{}",
                reference.0, reference.1, candidate.0, candidate.1
            ),
        }
    }
}

impl std::error::Error for DiffError {}

pub fn compare_images(
    reference: &DynamicImage,
    candidate: &DynamicImage,
    threshold: u8,
) -> Result<DiffMetrics, DiffError> {
    let reference = reference.to_rgb8();
    let candidate = candidate.to_rgb8();
    compare_rgb(&reference, &candidate, threshold)
}

pub fn compare_rgb(
    reference: &RgbImage,
    candidate: &RgbImage,
    threshold: u8,
) -> Result<DiffMetrics, DiffError> {
    if reference.dimensions() != candidate.dimensions() {
        return Err(DiffError::DimensionMismatch {
            reference: reference.dimensions(),
            candidate: candidate.dimensions(),
        });
    }

    let (width, height) = reference.dimensions();
    let pixel_count = width as u64 * height as u64;
    let channel_count = pixel_count * 3;
    let mut absolute_sum = 0_u64;
    let mut squared_sum = 0_u64;
    let mut max_channel_error = 0_u8;
    let mut exact_pixels = 0_u64;
    let mut pixels_above_threshold = 0_u64;

    for (a, b) in reference.pixels().zip(candidate.pixels()) {
        let mut pixel_max = 0_u8;
        let mut exact = true;
        for channel in 0..3 {
            let delta = a[channel].abs_diff(b[channel]);
            exact &= delta == 0;
            pixel_max = pixel_max.max(delta);
            max_channel_error = max_channel_error.max(delta);
            absolute_sum += delta as u64;
            squared_sum += (delta as u64) * (delta as u64);
        }
        if exact {
            exact_pixels += 1;
        }
        if pixel_max > threshold {
            pixels_above_threshold += 1;
        }
    }

    let mean_abs_error = if channel_count == 0 {
        0.0
    } else {
        absolute_sum as f64 / channel_count as f64
    };
    let root_mean_square_error = if channel_count == 0 {
        0.0
    } else {
        (squared_sum as f64 / channel_count as f64).sqrt()
    };
    let exact_pixel_fraction = if pixel_count == 0 {
        1.0
    } else {
        exact_pixels as f64 / pixel_count as f64
    };
    let fraction_above_threshold = if pixel_count == 0 {
        0.0
    } else {
        pixels_above_threshold as f64 / pixel_count as f64
    };

    Ok(DiffMetrics {
        width,
        height,
        pixel_count,
        mean_abs_error,
        root_mean_square_error,
        max_channel_error,
        exact_pixel_fraction,
        pixels_above_threshold,
        fraction_above_threshold,
    })
}

pub fn passes(metrics: DiffMetrics, limits: DiffLimits) -> bool {
    metrics.mean_abs_error <= limits.max_mean_abs_error
        && metrics.fraction_above_threshold <= limits.max_fraction_above_threshold
}

/// Writes an unscaled absolute RGB error image. A one-pixel reference error stays
/// one pixel in the output; this tool never resamples either fidelity input.
pub fn difference_image(
    reference: &DynamicImage,
    candidate: &DynamicImage,
) -> Result<RgbImage, DiffError> {
    let reference = reference.to_rgb8();
    let candidate = candidate.to_rgb8();
    if reference.dimensions() != candidate.dimensions() {
        return Err(DiffError::DimensionMismatch {
            reference: reference.dimensions(),
            candidate: candidate.dimensions(),
        });
    }

    let (width, height) = reference.dimensions();
    let mut out = RgbImage::new(width, height);
    for ((x, y, a), b) in reference.enumerate_pixels().zip(candidate.pixels()) {
        out.put_pixel(
            x,
            y,
            Rgb([
                a[0].abs_diff(b[0]),
                a[1].abs_diff(b[1]),
                a[2].abs_diff(b[2]),
            ]),
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(width: u32, height: u32, value: [u8; 3]) -> RgbImage {
        RgbImage::from_pixel(width, height, Rgb(value))
    }

    #[test]
    fn identical_images_are_exact() {
        let a = solid(4, 3, [10, 20, 30]);
        let metrics = compare_rgb(&a, &a, 0).unwrap();
        assert_eq!(metrics.mean_abs_error, 0.0);
        assert_eq!(metrics.root_mean_square_error, 0.0);
        assert_eq!(metrics.max_channel_error, 0);
        assert_eq!(metrics.exact_pixel_fraction, 1.0);
        assert_eq!(metrics.fraction_above_threshold, 0.0);
    }

    #[test]
    fn one_bad_pixel_is_counted_without_resampling() {
        let a = solid(2, 2, [0, 0, 0]);
        let mut b = a.clone();
        b.put_pixel(1, 1, Rgb([12, 0, 0]));
        let metrics = compare_rgb(&a, &b, 8).unwrap();
        assert_eq!(metrics.pixel_count, 4);
        assert_eq!(metrics.pixels_above_threshold, 1);
        assert_eq!(metrics.fraction_above_threshold, 0.25);
        assert_eq!(metrics.max_channel_error, 12);
        assert_eq!(metrics.exact_pixel_fraction, 0.75);
    }

    #[test]
    fn wrong_dimensions_are_rejected() {
        let a = solid(1920, 1080, [0, 0, 0]);
        let b = solid(1919, 1080, [0, 0, 0]);
        assert!(matches!(
            compare_rgb(&a, &b, 0),
            Err(DiffError::DimensionMismatch { .. })
        ));
    }
}
