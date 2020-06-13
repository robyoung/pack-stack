//! Manage packs of cards for playing online
#![deny(missing_docs, clippy::pedantic)]
#![feature(test)]

use image::RgbaImage;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

mod boundary;
mod data;
mod edge;
#[cfg(target_arch = "wasm32")]
mod performance;

/// Preallocated canny edge detector
#[wasm_bindgen]
pub struct Detector {
    canny: edge::Canny<edge::RectangleInRectangleWindow>,
    boundary_match: bool,
}

#[wasm_bindgen]
impl Detector {
    /// Create a new detector of a given size
    pub fn new(width: u32, height: u32) -> Detector {
        let boundary = boundary::get_corners(width, height);
        let window = edge::RectangleInRectangleWindow::new(
            boundary.grow(6),
            boundary.shrink(7),
        );
        Detector {
            canny: edge::CannyBuilder::with_window(width as usize, height as usize, window)
                .low_threshold(50.0)
                .build(),
            boundary_match: false,
        }
    }

    /// has a box been seen?
    pub fn boundary_match(&self) -> bool {
        self.boundary_match
    }

    /// detect edges
    pub fn detect(&mut self, input: Clamped<Vec<u8>>) -> Clamped<Vec<u8>> {
        let width = self.canny.width as u32;
        let height = self.canny.height as u32;

        let mut input = RgbaImage::from_raw(width, height, input.0).expect("Could not load image");

        self.canny.detect(&mut input);
        self.boundary_match = boundary::detect(&mut input, self.canny.line_colour());

        Clamped(input.into_raw())
    }
}
