//! Manage packs of cards for playing online
#![deny(missing_docs, clippy::pedantic)]
#![feature(test)]

use image::RgbaImage;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;

mod data;
pub mod edge;
#[cfg(target_arch = "wasm32")]
mod performance;
pub mod card;

/// Preallocated canny edge detector
#[wasm_bindgen]
pub struct Detector {
    detector: card::Detector,
    boundary_match: bool,
}

#[wasm_bindgen]
impl Detector {
    /// Create a new detector of a given size
    pub fn new(width: u32, height: u32) -> Detector {
        let detector = card::Detector::builder()
            .card_edge_width(3)
            .detection_window_width(20)
            .low_threshold(150.0)
            .high_threshold(200.0)
            .build(width as usize, height as usize);

        Detector { detector, boundary_match: false }
    }

    /// has a box been seen?
    pub fn boundary_match(&self) -> bool {
        self.boundary_match
    }

    fn width(&self) -> u32 {
        self.detector.width() as u32
    }

    fn height(&self) -> u32 {
        self.detector.height() as u32
    }

    /// detect edges
    pub fn detect(&mut self, input: Clamped<Vec<u8>>) -> Clamped<Vec<u8>> {
        let mut input = RgbaImage::from_raw(self.width(), self.height(), input.0).expect("Could not load image");

        self.boundary_match = self.detector.detect(&mut input);

        Clamped(input.into_raw())
    }
}
