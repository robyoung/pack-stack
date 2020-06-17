//! Canny edge detection
//!
//! Most of this is taken from https://github.com/nicksrandall/edge-detection-wasm
use image::{buffer::ConvertBuffer, GrayImage, Rgba, RgbaImage};
use std::{
    cmp::{max, min},
    f32, i16,
    ops::Deref,
};

use crate::data::{Point, Rectangle};
#[cfg(target_arch = "wasm32")]
use crate::performance;

/// A window within which the edge detection should run
///
/// This is useful when you don't need to detect edges in the whole image.
pub trait Window: Copy {
    /// Iterate over a series of Points
    type Iterator: Iterator<Item = Point>;

    /// The Points that need to be visited by the edge detection operator
    fn gradient(&self) -> Self::Iterator;

    /// The Points that need to be visited by non-maximum suppression and hysteresis
    fn process(&self) -> Self::Iterator;
}

/// Simplest form of edge detection window
///
/// Detect edges using all Points in this Rectangle.
#[derive(Copy, Clone)]
pub struct RectangleWindow {
    rectangle: Rectangle,
}

impl Window for RectangleWindow {
    type Iterator = RectangleWindowIterator;

    fn gradient(&self) -> RectangleWindowIterator {
        let next = *self.rectangle.top_left();
        RectangleWindowIterator {
            rectangle: self.rectangle.clone(),
            next: Some(next),
        }
    }

    fn process(&self) -> RectangleWindowIterator {
        let rectangle = self.rectangle.shrink(1);
        let next = *rectangle.top_left();
        RectangleWindowIterator {
            rectangle,
            next: Some(next),
        }
    }
}

/// Edge detection window shaped like a window frame
///
/// Detect edges using Points within the outer Rectangle but not within the inner Rectangle.
pub struct RectangleWindowIterator {
    rectangle: Rectangle,
    next: Option<Point>,
}

impl Iterator for RectangleWindowIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.next {
            if next[0] == self.rectangle.0[1][0] - 1 {
                if next[1] == self.rectangle.0[1][1] - 1 {
                    self.next = None;
                } else {
                    self.next = Some([self.rectangle.0[0][0], next[1] + 1]);
                }
            } else {
                self.next = Some([next[0] + 1, next[1]]);
            }
            Some(next)
        } else {
            None
        }
    }
}

/// A detection window shaped like a window frame
#[derive(Copy, Clone)]
pub struct RectangleInRectangleWindow {
    outer: Rectangle,
    inner: Rectangle,
}

impl RectangleInRectangleWindow {
    /// Create a new window frame shared detection window
    pub fn new(outer: Rectangle, inner: Rectangle) -> Self {
        assert!(outer.contains(&inner));
        Self { outer, inner }
    }

    /// Return the inner rectangle
    pub fn inner(&self) -> Rectangle {
        self.inner
    }

    /// Return the outer rectangle
    pub fn outer(&self) -> Rectangle {
        self.outer
    }
}

impl Window for RectangleInRectangleWindow {
    type Iterator = RectangleInRectangleWindowIterator;

    fn gradient(&self) -> RectangleInRectangleWindowIterator {
        let next = *self.outer.top_left();
        RectangleInRectangleWindowIterator {
            window: self.clone(),
            next: Some(next),
        }
    }

    fn process(&self) -> RectangleInRectangleWindowIterator {
        let window = RectangleInRectangleWindow {
            outer: self.outer.shrink(1),
            inner: self.inner.grow(1),
        };
        let next = *window.outer.top_left();
        RectangleInRectangleWindowIterator {
            window,
            next: Some(next),
        }
    }
}

#[allow(missing_docs)]
pub struct RectangleInRectangleWindowIterator {
    window: RectangleInRectangleWindow,
    next: Option<Point>,
}

impl Iterator for RectangleInRectangleWindowIterator {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        match self.next {
            Some(next) => {
                if next[0] == self.window.inner[0][0] - 1 && next[1] >= self.window.inner[0][1] && next[1] < self.window.inner[1][1] {
                    self.next = Some([self.window.inner[1][0], next[1]]);
                } else if next[0] == self.window.outer[1][0] - 1 {
                    if next[1] == self.window.outer[1][1] - 1{
                        self.next = None;
                    } else {
                        self.next = Some([self.window.outer[0][0], next[1] + 1]);
                    }
                } else {
                    self.next = Some([next[0] + 1, next[1]]);
                }
                Some(next)
            }
            None => None
        }
    }
}

/// Build a Canny edge detector
pub struct CannyBuilder<T: Window> {
    width: usize,
    height: usize,
    window: T,
    low_threshold: Option<f32>,
    high_threshold: Option<f32>,
    line_colour: Option<Rgba<u8>>,
}

impl CannyBuilder<RectangleWindow> {
    /// Create a new builder with a simple Rectangle edge detection window
    ///
    /// If you need a more complex edge detection window see `with_window`.
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize) -> CannyBuilder<RectangleWindow> {
        Self::with_window(
            width,
            height,
            RectangleWindow {
                rectangle: Rectangle([[0, 0], [width - 1, height - 1]]),
            },
        )
    }
}

impl<T: Window> CannyBuilder<T> {
    /// Create a new builder with a custom edge detection window
    pub fn with_window(width: usize, height: usize, window: T) -> CannyBuilder<T> {
        Self {
            width,
            height,
            window,
            low_threshold: None,
            high_threshold: None,
            line_colour: None,
        }
    }

    /// Set hysteresis low threshold
    pub fn low_threshold(&mut self, low_threshold: f32) -> &mut CannyBuilder<T> {
        self.low_threshold = Some(low_threshold);
        self
    }

    /// Set hysteresis high threshold
    #[allow(dead_code)]
    pub fn high_threshold(&mut self, high_threshold: f32) -> &mut CannyBuilder<T> {
        self.high_threshold = Some(high_threshold);
        self
    }

    /// Set colour to use for detected edges
    #[allow(dead_code)]
    pub fn line_colour(&mut self, line_colour: Rgba<u8>) -> &mut CannyBuilder<T> {
        self.line_colour = Some(line_colour);
        self
    }

    /// Build a canny edge detector
    pub fn build(&self) -> Canny<T> {
        Canny::new(
            self.width,
            self.height,
            self.low_threshold.unwrap_or(150.0),
            self.high_threshold.unwrap_or(300.0),
            self.line_colour.unwrap_or(Rgba([0, 0, 0, 255])),
            self.window,
        )
    }
}

/// Canny edge detector
pub struct Canny<T: Window> {
    gx: Vec<i16>,
    gy: Vec<i16>,
    filtered: Vec<f32>,
    supressed: Vec<f32>,

    /// width of the image
    pub width: usize,
    ///height of the image
    pub height: usize,
    low_threshold: f32,
    high_threshold: f32,
    line_colour: Rgba<u8>,
    window: T,
}

impl<T: Window> Canny<T> {
    pub(crate) fn new(
        width: usize,
        height: usize,
        low_threshold: f32,
        high_threshold: f32,
        line_colour: Rgba<u8>,
        window: T,
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
            line_colour,
            window,
        }
    }

    /// Line colour used by edge detector
    pub fn line_colour(&self) -> Rgba<u8> {
        self.line_colour
    }

    /// Detect edges in an image
    pub fn detect(&mut self, src: &mut RgbaImage) {
        #[cfg(target_arch = "wasm32")]
        let timer = performance::Timer::new("canny::setup-struct");
        // TODO: convert into existing image when possible
        let mut result = src;
        let src: GrayImage = result.convert();
        let src = src.deref();
        #[cfg(target_arch = "wasm32")]
        std::mem::drop(timer);

        gradient(
            self.width,
            self.height,
            src,
            &mut self.gx,
            &mut self.gy,
            &mut self.filtered,
            &self.window,
        );

        non_maximum_suppression(
            self.width,
            &self.filtered,
            &self.gx,
            &self.gy,
            &mut self.supressed,
            &self.window,
        );

        hysteresis(
            self.width as u32,
            self.height as u32,
            &self.supressed,
            &mut result,
            self.low_threshold,
            self.high_threshold,
            self.line_colour,
            &self.window,
        );
    }
}

const BLACK_32: f32 = 0.0;

/// Sobel filter for detecting vertical gradients.
const VERTICAL_SOBEL: [i32; 9] = [-1, -2, -1, 0, 0, 0, 1, 2, 1];

/// Sobel filter for detecting horizontal gradients.
const HORIZONTAL_SOBEL: [i32; 9] = [-1, 0, 1, -2, 0, 2, -1, 0, 1];

/// Finds local maxima to make the edges thinner.
fn non_maximum_suppression<T: Window>(
    width: usize,
    g: &Vec<f32>,
    gx: &Vec<i16>,
    gy: &Vec<i16>,
    out: &mut Vec<f32>,
    window: &T,
) {
    #[cfg(target_arch = "wasm32")]
    let _timer = performance::Timer::new("canny::non_max");
    const RADIANS_TO_DEGREES: f32 = 180_f32 / f32::consts::PI;
    for [x, y] in window.process() {
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

/// Filter out edges with the thresholds.
/// Non-recursive breadth-first search.
fn hysteresis<T: Window>(
    width: u32,
    height: u32,
    input: &Vec<f32>,
    out: &mut RgbaImage,
    low_thresh: f32,
    high_thresh: f32,
    line_colour: Rgba<u8>,
    window: &T,
) {
    #[cfg(target_arch = "wasm32")]
    let _timer = performance::Timer::new("canny::hysteresis");
    let low_thresh = low_thresh * low_thresh;
    let high_thresh = high_thresh * high_thresh;
    let mut edges = Vec::with_capacity((width * height) as usize / 2);

    for [x, y] in window.process() {
        let (inp_pix, out_pix) = (
            input[((y * width as usize) + x)],
            out.get_pixel(x as u32, y as u32),
        );
        // If the edge strength is higher than high_thresh, mark it as an edge.
        if inp_pix >= high_thresh && out_pix[0] != line_colour[0] {
            out.put_pixel(x as u32, y as u32, line_colour);
            edges.push((x as u32, y as u32));

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
                    if in_neighbor >= low_thresh && out_neighbor[0] != line_colour[0] {
                        out.put_pixel(
                            neighbor_idx.0 as u32,
                            neighbor_idx.1 as u32,
                            line_colour,
                        );
                        edges.push((neighbor_idx.0, neighbor_idx.1));
                    }
                }
            }
        }
    }
}

#[allow(clippy::similar_names)]
fn gradient<T: Window>(
    width: usize,
    height: usize,
    image: &[u8],
    hout: &mut Vec<i16>,
    vout: &mut Vec<i16>,
    out: &mut Vec<f32>,
    window: &T,
) {
    #[cfg(target_arch = "wasm32")]
    let _timer = performance::Timer::new("canny::gradient");
    let (k_width, k_height) = (3, 3);

    let mut hacc = 0_i32;
    let mut vacc = 0_i32;

    for [x, y] in window.gradient() {
        // inner loops
        for k_y in 0..k_height {
            let y_p = min(
                height + height - 1,
                max(height, height + y + k_y - k_height / 2),
            ) - height; // min(height - 1, max(0, y + k_y - k_height / 2))
            for k_x in 0..k_width {
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
fn atan2_approx(y: f32, x: f32) -> f32 {
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
mod tests {
    extern crate test;

    use super::{gradient, CannyBuilder, RectangleWindow, RectangleInRectangleWindow, Window};
    use crate::data::{Rectangle, Point};
    use image;
    use test::Bencher;

    #[test]
    fn test_filter() {
        let (width, height) = (10, 10);
        let image = vec![0; width * height];
        let mut hout = vec![0; width * height];
        let mut vout = vec![0; width * height];
        let mut out = vec![0_f32; width * height];
        let window = RectangleWindow {
            rectangle: Rectangle([[0, 0], [width - 1, height - 1]]),
        };

        gradient(width, height, &image, &mut hout, &mut vout, &mut out, &window);
    }

    #[test]
    fn test_rect_window() {
        let window = RectangleWindow {
            rectangle: Rectangle([[0, 0], [10, 10]]),
        };
        let points = window.gradient().collect::<Vec<Point>>();

        assert_eq!(points.len(), 100);
        assert_eq!(points.iter().map(|&p| p[0]).max(), Some(9));
        assert_eq!(points.iter().map(|&p| p[1]).max(), Some(9));
    }

    #[test]
    fn test_rect_in_rect_window() {
        let window = RectangleInRectangleWindow {
            outer: Rectangle([[0, 0], [10, 10]]),
            inner: Rectangle([[2, 2], [6, 6]]),
        };
        let points = window.gradient().collect::<Vec<Point>>();

        assert_eq!(points.len(), 84);
    }

    #[bench]
    fn bench_detect(b: &mut Bencher) {
        let img = image::open("test_images/test.jpg").unwrap().to_rgba();
        let width = img.width();
        let height = img.height();
        let outer = Rectangle([[0, 0], [width as usize, height as usize]]);
        let window = RectangleInRectangleWindow::new(
            outer,
            outer.shrink(14),
        );

        let mut canny = CannyBuilder::with_window(width as usize, height as usize, window).build();

        b.iter(|| {
            let mut img = img.clone();
            canny.detect(&mut img);
        });
    }
}
