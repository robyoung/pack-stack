//! Manage packs of cards for playing online
#![deny(missing_docs, clippy::pedantic)]
#![feature(test)]

use image::{Rgba, RgbaImage};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

mod edge;
#[cfg(target_arch = "wasm32")]
mod performance;

/// Preallocated canny edge detector
#[wasm_bindgen]
pub struct Detector {
    canny: edge::Canny,
    add_box: bool,
    box_match: bool, // TODO: rename this, it's awful
}

#[wasm_bindgen]
impl Detector {
    /// Create a new detector of a given size
    pub fn new(width: u32, height: u32, add_box: bool) -> Detector {
        Detector {
            canny: edge::Canny::new(width as usize, height as usize, 50.0, 300.0, 255, false),
            add_box,
            box_match: false,
        }
    }

    /// has a box been seen?
    pub fn box_match(&self) -> bool {
        self.box_match
    }

    /// update the box_match flag
    fn update_box_match(&mut self, buf: &RgbaImage) {
        self.box_match = false;

        let width = self.canny.width as u32;
        let height = self.canny.height as u32;

        let (top_left, bottom_right) = get_corners(width, height);
        let mut hits = 0;
        let line_colour = self.canny.line_colour();

        // horizontal lines
        for y in [top_left.1, bottom_right.1].iter() {
            for x in (top_left.0)..(bottom_right.0) {
                // if any pixel either side of the horizontal line
                if ((y - 1)..(y + 1)).any(|y| *buf.get_pixel(x, y) == line_colour) {
                    hits += 1;
                }
            }
        }

        // vertical lines
        for x in [top_left.0, bottom_right.0].iter() {
            for y in (top_left.1 + 2)..(bottom_right.0 - 2) {
                if ((x - 1)..(x + 1)).any(|x| *buf.get_pixel(x,y) == line_colour) {
                    hits += 1;
                }
            }
        }

        let circumference = (bottom_right.1 - top_left.1) * 2 + (bottom_right.0 + top_left.0) * 2;
        if hits as f32 / circumference as f32 > 0.9 {
            self.box_match = true;
        }
    }

    /// detect edges
    pub fn detect(&mut self, input: Clamped<Vec<u8>>) -> Clamped<Vec<u8>> {
        let width = self.canny.width as u32;
        let height = self.canny.height as u32;

        let mut input = RgbaImage::from_raw(width, height, input.0).expect("Could not load image");

        self.canny.detect(&mut input);
        self.update_box_match(&input);

        if self.add_box {
            for pixel in get_line_pixels(get_corners(width, height), 1) {
                input.put_pixel(pixel.0, pixel.1, Rgba([255; 4]));
            }
        }

        Clamped(input.into_raw())
    }
}

type Point = (u32, u32);
type LineWeight = u32;

/// ratio taken from standard card dimensions of 2.5 x 3.5
const RATIO: f32 = 5.0 / 7.0;

/// 5% margin between edge of frame and corner lines
const MARGIN: f32 = 0.05;

pub(crate) fn get_corners(width: u32, height: u32) -> (Point, Point) {
    let height = height as f32;
    let width = width as f32;

    let (width, height) = if width / height > RATIO {
        (height * RATIO, height)
    } else {
        (width, width / RATIO)
    };
    let margin = (width * MARGIN).floor() as u32;
    (
        (margin, margin),
        (
            width.floor() as u32 - margin,
            height.floor() as u32 - margin,
        ),
    )
}

// 1. draw_lines: pass in corners and image and it will put the pixels
// 2. get_line_pixels: pass in corners and it returns an iterator over the pixels
pub(crate) fn get_line_pixels(
    corners: (Point, Point),
    weight: LineWeight,
) -> impl Iterator<Item = Point> {
    LinePixels::new(corners, weight)
}

struct LinePixels {
    top_left: Point,
    bottom_right: Point,
    weight: LineWeight,
    line: u8,
    next: Point,
}

impl LinePixels {
    fn new(corners: (Point, Point), weight: LineWeight) -> Self {
        Self {
            top_left: corners.0,
            bottom_right: corners.1,
            weight,
            line: 1,
            next: corners.0,
        }
    }
}

impl Iterator for LinePixels {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next;

        let line_length = match self.line {
            1..=4 => (self.bottom_right.0 - self.top_left.0) / 4,
            5..=8 => (self.bottom_right.1 - self.top_left.1) / 4,
            _ => unreachable!(),
        };

        match self.line {
            // top left horizontal
            1 => {
                if next.0 < self.top_left.0 + line_length {
                    // move along the line
                    self.next.0 += 1;
                } else if next.1 < (self.top_left.1 + self.weight) {
                    // next row of line
                    self.next.0 = self.top_left.0;
                    self.next.1 += 1;
                } else {
                    // move to start of next line (top right horizontal)
                    self.line += 1;
                    self.next.0 = self.bottom_right.0 - line_length;
                    self.next.1 = self.top_left.1;
                }
            }
            // top right horizontal
            2 => {
                if next.0 < self.bottom_right.0 {
                    // move along the line
                    self.next.0 += 1;
                } else if next.1 < (self.top_left.1 + self.weight) {
                    // next row of the line
                    self.next.0 = self.bottom_right.0 - line_length;
                    self.next.1 += 1;
                } else {
                    // move to start of next line (bottom left horizontal)
                    self.line += 1;
                    self.next.0 = self.top_left.0;
                    self.next.1 = self.bottom_right.1;
                }
            }
            // bottom left horizontal
            3 => {
                if next.0 < self.top_left.0 + line_length {
                    // move along the line
                    self.next.0 += 1;
                } else if self.next.1 > (self.bottom_right.1 - self.weight) {
                    // next row of line
                    self.next.0 = self.top_left.0;
                    self.next.1 -= 1;
                } else {
                    // move to start of next line (bottom right horizontal)
                    self.line += 1;
                    self.next.0 = self.bottom_right.0 - line_length;
                    self.next.1 = self.bottom_right.1;
                }
            }
            // bottom right horizontal
            4 => {
                if next.0 < self.bottom_right.0 {
                    // move along the line
                    self.next.0 += 1;
                } else if next.1 > (self.bottom_right.1 - self.weight) {
                    // next row of line
                    self.next.0 = self.bottom_right.0 - line_length;
                    self.next.1 -= 1;
                } else {
                    // move to start of next line (top left vertical)
                    self.line += 1;
                    self.next.0 = self.top_left.0;
                    self.next.1 = self.top_left.1 + self.weight;
                }
            }
            // top left vertical
            5 => {
                if next.1 < self.top_left.1 + line_length {
                    // move along the line
                    self.next.1 += 1;
                } else if next.0 < self.top_left.0 + self.weight {
                    // next column of line
                    self.next.0 += 1;
                    self.next.1 = self.top_left.1 + self.weight;
                } else {
                    // move to start of next line (top right vertical)
                    self.line += 1;
                    self.next.0 = self.bottom_right.0;
                    self.next.1 = self.top_left.1 + self.weight;
                }
            }
            // top right vertical
            6 => {
                if next.1 < self.top_left.1 + line_length {
                    // move along the line
                    self.next.1 += 1;
                } else if next.0 > self.bottom_right.0 - self.weight {
                    // next column of line
                    self.next.0 -= 1;
                    self.next.1 = self.top_left.1 + self.weight;
                } else {
                    // move to start of next line (bottom left vertical)
                    self.line += 1;
                    self.next.0 = self.top_left.0;
                    self.next.1 = self.bottom_right.1 - line_length;
                }
            }
            // bottom left vertical
            7 => {
                if next.1 < self.bottom_right.1 - self.weight {
                    // move along the line
                    self.next.1 += 1;
                } else if next.0 < self.top_left.0 + self.weight {
                    // next column of line
                    self.next.0 += 1;
                    self.next.1 = self.bottom_right.1 - line_length;
                } else {
                    // move to start of next line (bottom right vertical)
                    self.line += 1;
                    self.next.0 = self.bottom_right.0;
                    self.next.1 = self.bottom_right.1 - line_length;
                }
            }
            8 => {
                if next.1 < self.bottom_right.1 - self.weight {
                    // move along the line
                    self.next.1 += 1;
                } else if next.0 > self.bottom_right.0 - self.weight {
                    // next column of line
                    self.next.0 -= 1;
                    self.next.1 = self.bottom_right.1 - line_length;
                } else {
                    return None;
                }
            }
            _ => unreachable!(),
        }

        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::{get_corners, get_line_pixels, Point};
    use std::collections::HashSet;

    #[test]
    fn test_get_corners() {
        let test_box = ((2, 2), (38, 54));

        assert_eq!(get_corners(40, 56), test_box);
        // smallest edge used matches test_box
        assert_eq!(get_corners(49, 56), test_box);
        assert_eq!(get_corners(40, 59), test_box);
        // smallest edge used does not match test_box
        assert_ne!(get_corners(39, 56), test_box);
        assert_ne!(get_corners(40, 54), test_box);
    }

    #[test]
    fn test_get_line_pixels() {
        let corners = ((2, 2), (22, 22));
        let pixels = get_line_pixels(corners, 3).collect::<HashSet<Point>>();

        assert!(pixels.contains(&(2, 2)));
        assert!(pixels.contains(&(7, 2)));
        assert!(!pixels.contains(&(8, 2)));
        assert!(pixels.contains(&(2, 7)));
        assert!(!pixels.contains(&(2, 8)));
        assert!(!pixels.contains(&(6, 6)));
    }
}
