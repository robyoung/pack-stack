//! Canny edge detection
//!
//! Most of this is taken from https://github.com/nicksrandall/edge-detection-wasm
use std::{
    cmp::{max, min},
    f32, i16,
    ops::Deref,
};
use image::{buffer::ConvertBuffer, GrayImage, RgbaImage};

#[cfg(target_arch = "wasm32")]
use crate::performance;

pub struct Canny {
    gx: Vec<i16>,
    gy: Vec<i16>,
    filtered: Vec<f32>,
    supressed: Vec<f32>,

    pub width: usize,
    pub height: usize,
    low_threshold: f32,
    high_threshold: f32,
    line_shade: u8,
    use_thick: bool,
}

impl Canny {
    pub fn new(
        width: usize,
        height: usize,
        low_threshold: f32,
        high_threshold: f32,
        line_shade: u8,
        use_thick: bool,
    ) -> Self {
        Self {
            gx: vec![0; width * height],
            gy: vec![0; width * height],
            filtered: vec![0_f32; width * height],
            supressed: vec![0_f32; width * height],

            width,
            height,
            low_threshold,
            high_threshold,
            line_shade,
            use_thick,
        }
    }

    pub fn detect(&mut self, src: &mut RgbaImage) {
        #[cfg(target_arch = "wasm32")]
        let timer = performance::Timer::new("canny::setup-struct");
        // TODO: convert into existing image when possible
        let mut result = src;
        let src: GrayImage = result.convert();
        let src = src.deref();
        // TODO: pull up to struct somehow?
        // let mut result = GrayImage::new(self.width as u32, self.height as u32);
        #[cfg(target_arch = "wasm32")]
        std::mem::drop(timer);

        filter(
            self.width,
            self.height,
            src,
            &mut self.gx,
            &mut self.gy,
            &mut self.filtered,
        );

        non_maximum_suppression(
            self.width,
            self.height,
            &self.filtered,
            &self.gx,
            &self.gy,
            &mut self.supressed,
        );

        hysteresis(
            self.width as u32,
            self.height as u32,
            &self.supressed,
            &mut result,
            self.low_threshold,
            self.high_threshold,
            self.line_shade,
            self.use_thick,
        );
    }
}

const BLACK_32: f32 = 0.0;

/// Sobel filter for detecting vertical gradients.
const VERTICAL_SOBEL: [i32; 9] = [-1, -2, -1, 0, 0, 0, 1, 2, 1];

/// Sobel filter for detecting horizontal gradients.
const HORIZONTAL_SOBEL: [i32; 9] = [-1, 0, 1, -2, 0, 2, -1, 0, 1];


/// Finds local maxima to make the edges thinner.
pub fn non_maximum_suppression(
    width: usize,
    height: usize,
    g: &Vec<f32>,
    gx: &Vec<i16>,
    gy: &Vec<i16>,
    out: &mut Vec<f32>,
) {
    #[cfg(target_arch = "wasm32")]
    let _timer = performance::Timer::new("canny::non_max");
    const RADIANS_TO_DEGREES: f32 = 180f32 / f32::consts::PI;
    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let i = y * width + x;
            let x_gradient = gx[i] as f32;
            let y_gradient = gy[i] as f32;
            let mut angle = atan2_approx(y_gradient, x_gradient) * RADIANS_TO_DEGREES;
            if angle < 0.0 {
                angle += 180.0
            }
            let (cmp1, cmp2) = if angle >= 157.5 || angle < 22.5 {
                (g[y * width + x - 1], g[y * width + x + 1])
            } else if angle >= 22.5 && angle < 67.5 {
                (g[(y + 1) * width + x + 1], g[(y - 1) * width + x - 1])
            } else if angle >= 67.5 && angle < 112.5 {
                (g[(y - 1) * width + x], g[(y + 1) + x])
            } else if angle >= 112.5 && angle < 157.5 {
                (g[(y + 1) * width + x - 1], g[(y - 1) * width + x + 1])
            } else {
                unreachable!()
            };

            let pixel = g[y * width + x];
            // If the pixel is not a local maximum, suppress it.
            if pixel < cmp1 || pixel < cmp2 {
                out[i] = BLACK_32;
            } else {
                out[i] = pixel;
            }
        }
    }
}

/// Filter out edges with the thresholds.
/// Non-recursive breadth-first search.
pub fn hysteresis(
    width: u32,
    height: u32,
    input: &Vec<f32>,
    out: &mut RgbaImage,
    low_thresh: f32,
    high_thresh: f32,
    line_shade: u8,
    use_thick: bool,
) {
    #[cfg(target_arch = "wasm32")]
    let _timer = performance::Timer::new("canny::hysteresis");
    let low_thresh = low_thresh * low_thresh;
    let high_thresh = high_thresh * high_thresh;
    let pixel = image::Rgba([0, u8::MAX, 0, line_shade]);
    let mut edges = Vec::with_capacity((width * height) as usize / 2);

    for y in 1..height - 1 {
        for x in 1..width - 1 {
            let (inp_pix, out_pix) = (
                input[((y * width) + x) as usize],
                out.get_pixel(x as u32, y as u32),
            );
            // If the edge strength is higher than high_thresh, mark it as an edge.
            if inp_pix >= high_thresh && out_pix[0] != pixel[0] {
                out.put_pixel(x as u32, y as u32, pixel);
                edges.push((x, y));

                // Track neighbors until no neighbor is >= low_thresh.
                while !edges.is_empty() {
                    let (nx, ny) = edges.pop().unwrap();
                    let neighbor_indices = [
                        (nx + 1, ny, nx - 1, ny),
                        (nx + 1, ny + 1, nx - 1, ny + 1),
                        (nx, ny + 1, nx + 1, ny),
                        (nx - 1, ny - 1, nx + 1, ny + 1),
                        (nx - 1, ny, nx, ny + 1),
                        (nx - 1, ny + 1, nx - 1, ny - 1),
                    ];
                    // let neighbor_indices = [
                    //     (nx + 1, ny),
                    //     (nx + 1, ny + 1),
                    //     (nx, ny + 1),
                    //     (nx - 1, ny - 1),
                    //     (nx - 1, ny),
                    //     (nx - 1, ny + 1),
                    // ];

                    for neighbor_idx in &neighbor_indices {
                        let (in_neighbor, out_neighbor) = (
                            input[((neighbor_idx.1 * width) + neighbor_idx.0) as usize],
                            out.get_pixel(neighbor_idx.0 as u32, neighbor_idx.1 as u32),
                        );
                        if in_neighbor >= low_thresh && out_neighbor[0] != pixel[0] {
                            out.put_pixel(neighbor_idx.0 as u32, neighbor_idx.1 as u32, pixel);
                            if use_thick {
                                out.put_pixel(neighbor_idx.2 as u32, neighbor_idx.3 as u32, pixel);
                            }
                            edges.push((neighbor_idx.0, neighbor_idx.1));
                        }
                    }
                }
            }
        }
    }
}

pub fn filter(
    width: usize,
    height: usize,
    image: &[u8],
    hout: &mut Vec<i16>,
    vout: &mut Vec<i16>,
    out: &mut Vec<f32>,
) {
    #[cfg(target_arch = "wasm32")]
    let _timer = performance::Timer::new("canny::filter");
    let (k_width, k_height) = (3, 3);

    let mut hacc = 0_i32;
    let mut vacc = 0_i32;

    for y in 0..height {
        for x in 0..width {
            // inner loops
            for k_y in 0..k_height {
                // TODO: pull out of inner loop
                let y_p = min(
                    height + height - 1,
                    max(height, height + y + k_y - k_height / 2),
                ) - height; // min(height - 1, max(0, y + k_y - k_height / 2))
                for k_x in 0..k_width {
                    // TODO: pull out of inner loop
                    let x_p =
                        min(width + width - 1, max(width, width + x + k_x - k_width / 2)) - width;

                    let (p, hk, vk) = (
                        image[y_p * width + x_p],
                        HORIZONTAL_SOBEL[k_y * k_width + k_x],
                        VERTICAL_SOBEL[k_y * k_width + k_x],
                    );
                    hacc = accumulate(hacc, p, hk);
                    vacc = accumulate(vacc, p, vk);
                } // end k_width loop
            } // end k_height loop

            let h = clamp(hacc);
            let v = clamp(vacc);
            let p = (h as f32) * (h as f32) + (v as f32) * (v as f32);
            hacc = 0_i32;
            vacc = 0_i32;
            let i = y * width + x;
            hout[i] = h;
            vout[i] = v;
            // TODO: out == h^2 + v^2 so do them all at the end?
            out[i] = p;
        }
    }
}

#[inline]
fn clamp(x: i32) -> i16 {
    if x < i16::MAX as i32 {
        if x > i16::MIN as i32 {
            x as i16
        } else {
            i16::MIN
        }
    } else {
        i16::MAX
    }
}

#[inline]
fn accumulate(acc: i32, pixel: u8, weight: i32) -> i32 {
    acc + pixel as i32 * weight
}

// borrowed this code from: https://gist.github.com/volkansalma/2972237
pub fn atan2_approx(y: f32, x: f32) -> f32 {
    const ONEQTR_PI: f32 = f32::consts::PI / 4.0;
    const THRQTR_PI: f32 = 3.0 * f32::consts::PI / 4.0;
    let abs_y = (y).abs() + 1e-10_f32;
    let (r, angle) = if x < 0.0 {
        ((x + abs_y) / (abs_y - x), THRQTR_PI)
    } else {
        ((x - abs_y) / (x + abs_y), ONEQTR_PI)
    };
    let angle = angle + (0.1963 * r * r - 0.9817) * r;
    if y < 0.0 {
        -angle
    } else {
        angle
    }
}

#[cfg(test)]
mod test {
    use super::filter;

    #[test]
    fn test_filter() {
        let (width, height) = (10, 10);
        let image = vec![0; width * height];
        let mut hout = vec![0; width * height];
        let mut vout = vec![0; width * height];
        let mut out = vec![0_f32; width * height];

        filter(width, height, &image, &mut hout, &mut vout, &mut out);
    }
}

#[cfg(test)]
mod tests {
    extern crate test;

    use super::Canny;
    use image;
    use test::Bencher;

    #[bench]
    fn bench_detect(b: &mut Bencher) {
        let img = image::open("test_images/test.jpg").unwrap().to_rgba();
        let width = img.width();
        let height = img.height();

        let mut canny = Canny::new(width as usize, height as usize, 150.0, 300.0, 255, false);
        b.iter(|| {
            let mut img = img.clone();
            canny.detect(&mut img);
        });
    }
}
