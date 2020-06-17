//! Detecting whether a playing card is present in a given area

use image::{Rgba, RgbaImage};

use crate::data::Rectangle;
use crate::edge::{Canny, RectangleInRectangleWindow};

/// Detect whether a playing card is present exactly in the boundary
pub struct Detector {
    card_edge_width: usize,

    canny: Canny<RectangleInRectangleWindow>,
    boundary: Rectangle,
    inner_boundary: Rectangle,
    outer_boundary: Rectangle,
}

const EDGE_COLOUR: Rgba<u8> = Rgba([0, 0, 0, 254]);
const MISS_COLOUR: Rgba<u8> = Rgba([255, 0, 0, 255]);
const HIT_COLOUR: Rgba<u8> = Rgba([0, 255, 0, 255]);

impl Detector {
    /// Create a builder object to help create this detector
    pub fn builder() -> DetectorBuilder {
        DetectorBuilder::default()
    }

    fn new(
        width: usize,
        height: usize,
        card_edge_width: usize,
        detection_window_width: usize,
        low_threshold: f32,
        high_threshold: f32,
    ) -> Self {
        let image_rect = Rectangle::from_dimensions(width, height);
        let boundary = get_corners(width, height);
        let outer_boundary = boundary.clamped_grow(detection_window_width / 2, &image_rect);
        let inner_boundary = boundary.clamped_shrink(detection_window_width / 2, &image_rect);

        let window = RectangleInRectangleWindow::new(outer_boundary, inner_boundary);

        Detector {
            card_edge_width,

            canny: Canny::new(
                width,
                height,
                low_threshold,
                high_threshold,
                EDGE_COLOUR,
                window,
            ),
            boundary,
            outer_boundary,
            inner_boundary,
        }
    }

    /// Width of the image
    pub fn width(&self) -> usize {
        self.canny.width
    }

    /// Height of the image
    pub fn height(&self) -> usize {
        self.canny.height
    }

    /// Detect if a card is in the boundary
    pub fn detect(&mut self, img: &mut RgbaImage) -> bool {
        self.canny.detect(img);

        let mut scores = vec![];

        // horizontal lines
        let horizontal_lines = [
            // top line
            (
                self.boundary.top_left()[1] as u32,
                self.outer_boundary.top_left()[1] as u32,
                self.inner_boundary.top_left()[1] as u32,
            ),
            // bottom line
            (
                self.boundary.bottom_right()[1] as u32,
                self.inner_boundary.bottom_right()[1] as u32,
                self.outer_boundary.bottom_right()[1] as u32,
            ),
        ];

        for (y, min_y, max_y) in horizontal_lines.iter().cloned() {
            let hits = self
                .boundary
                .x_range()
                .filter(|&x| {
                    let x = x as u32;

                    let (colour, hit) =
                        if (min_y..max_y).any(|y| *img.get_pixel(x, y) == EDGE_COLOUR) {
                            (HIT_COLOUR, true)
                        } else {
                            (MISS_COLOUR, false)
                        };

                    let y_range = if y < self.boundary.height() as u32 / 2 {
                        y - self.card_edge_width as u32..=y
                    } else {
                        y..=y + self.card_edge_width as u32
                    };
                    for y in y_range {
                        img.put_pixel(x, y, colour);
                    }
                    hit
                })
                .count();
            scores.push(hits as f32 / self.boundary.width() as f32);
        }

        // vertical lines
        let vertical_lines = [
            // left line
            (
                self.boundary.top_left()[0] as u32,
                self.outer_boundary.top_left()[0] as u32,
                self.inner_boundary.top_left()[0] as u32,
            ),
            // right line
            (
                self.boundary.bottom_right()[0] as u32,
                self.inner_boundary.bottom_right()[0] as u32,
                self.outer_boundary.bottom_right()[0] as u32,
            ),
        ];

        for (x, min_x, max_x) in vertical_lines.iter().cloned() {
            let hits = self
                .boundary
                .y_range()
                .filter(|&y| {
                    let y = y as u32;

                    let (colour, hit) =
                        if (min_x..max_x).any(|x| *img.get_pixel(x, y) == EDGE_COLOUR) {
                            (HIT_COLOUR, true)
                        } else {
                            (MISS_COLOUR, true)
                        };

                    let x_range = if x < self.boundary.width() as u32 / 2 {
                        x - self.card_edge_width as u32..=x
                    } else {
                        x..=x + self.card_edge_width as u32
                    };
                    for x in x_range {
                        img.put_pixel(x, y, colour);
                    }
                    hit
                })
                .count();
            scores.push(hits as f32 / self.boundary.height() as f32);
        }

        // at least 3 sides have scores above 80%
        scores.into_iter().filter(|&s| s > 0.8).count() >= 3
    }
}

/// Builder for a card detector
#[derive(Default)]
pub struct DetectorBuilder {
    card_edge_width: Option<usize>,
    detection_window_width: Option<usize>,
    low_threshold: Option<f32>,
    high_threshold: Option<f32>,
}

impl DetectorBuilder {
    /// Width in pixels of the drawn card edge
    ///
    /// A line is drawn showing the boundary of where the card is expected, this is how thick that
    /// drawn line should be.
    pub fn card_edge_width(&mut self, value: usize) -> &mut Self {
        self.card_edge_width = Some(value);
        self
    }

    /// Width of the detection window around the card edge
    ///
    /// When detecting whether a card is in the boundary we check the pixels either side of where
    /// we expect the card to be. This value defines how many pixels either side we look. A value
    /// of 20 would mean up to 10 pixels either side.
    pub fn detection_window_width(&mut self, value: usize) -> &mut Self {
        self.detection_window_width = Some(value);
        self
    }

    /// Canny hysterisis low threshold
    ///
    /// The low threshold of the hysterisis stage of the canny edge detector.
    pub fn low_threshold(&mut self, value: f32) -> &mut Self {
        self.low_threshold = Some(value);
        self
    }

    /// Canny hysterisis high threshold
    ///
    /// The high threshold of the hysterisis stage of the canny edge detector.
    pub fn high_threshold(&mut self, value: f32) -> &mut Self {
        self.high_threshold = Some(value);
        self
    }

    /// Build the Detector
    pub fn build(&self, width: usize, height: usize) -> Detector {
        Detector::new(
            width,
            height,
            self.card_edge_width.unwrap_or(3),
            self.detection_window_width.unwrap_or(20),
            self.low_threshold.unwrap_or(150.0),
            self.high_threshold.unwrap_or(200.0),
        )
    }
}

/// ratio taken from standard card dimensions of 2.5 x 3.5
const RATIO: f32 = 5.0 / 7.0;

/// 5% margin between edge of frame and corner lines
const MARGIN: f32 = 0.05;

/// Get the corners of the boundary
fn get_corners(width: usize, height: usize) -> Rectangle {
    let height = height as f32;
    let width = width as f32;

    let (width, height) = if width / height > RATIO {
        (height * RATIO, height)
    } else {
        (width, width / RATIO)
    };
    let margin = (width * MARGIN).floor() as usize;
    Rectangle([
        [margin, margin],
        [
            width.floor() as usize - margin,
            height.floor() as usize - margin,
        ],
    ])
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;
    use crate::data::Rectangle;

    use super::{Detector, get_corners};

    #[test]
    fn test_get_corners() {
        let test_box = Rectangle([[2, 2], [38, 54]]);

        assert_eq!(get_corners(40, 56), test_box);
        // smallest edge used matches test_box
        assert_eq!(get_corners(49, 56), test_box);
        assert_eq!(get_corners(40, 59), test_box);
        // smallest edge used does not match test_box
        assert_ne!(get_corners(39, 56), test_box);
        assert_ne!(get_corners(40, 54), test_box);
    }

    #[test]
    fn test_detect() {
        let mut img = image::open("test_images/uno-7.jpg").unwrap().to_rgba();
        let mut detector = Detector::builder()
            .card_edge_width(0)
            .detection_window_width(20)
            .build(img.width() as usize, img.height() as usize);

        assert!(detector.detect(&mut img));

        img.save("test_images/uno-7-save.jpg").unwrap();
    }

    #[bench]
    fn bench_detect(b: &mut Bencher) {
        let img = image::open("test_images/uno-7.jpg").unwrap().to_rgba();
        let mut detector = Detector::builder()
            .card_edge_width(0)
            .detection_window_width(20)
            .build(img.width() as usize, img.height() as usize);

        b.iter(|| {
            let mut img = img.clone();
            assert!(detector.detect(&mut img));
        });
    }
}
