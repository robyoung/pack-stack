use image::{Rgba, RgbaImage};

#[derive(Copy, Clone, PartialEq, Debug, Hash, Eq)]
pub struct Point(u32, u32);

impl Point {
    fn x(&self) -> u32 {
        self.0
    }
    fn y(&self) -> u32 {
        self.1
    }
}

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
        Point(margin, margin),
        Point(
            width.floor() as u32 - margin,
            height.floor() as u32 - margin,
        ),
    )
}

pub(crate) fn detect(buffer: &mut RgbaImage, line_colour: Rgba<u8>) -> bool {
    let width = buffer.width();
    let height = buffer.height();
    let mut hits = 0;

    let (top_left, bottom_right) = get_corners(width, height);
    let miss_colour = Rgba([255, 0, 0, 255]);
    let hit_colour = Rgba([0, 255, 0, 255]);

    // horizontal lines
    let boundary_width = (width as f32 * 0.05).ceil() as u32;

    for y in [top_left.y(), bottom_right.y()].iter() {
        // if any pixel either side of the horizontal line
        let start = if boundary_width > *y {
            0
        } else {
            y - boundary_width
        };
        let end = if y + boundary_width > height {
            height
        } else {
            y + boundary_width
        };
        for x in top_left.x()..bottom_right.x() {
            if (start..end)
                .any(|y| *buffer.get_pixel(x, y) == line_colour)
            {
                hits += 1;
                buffer.put_pixel(x, *y, hit_colour);
            } else {
                buffer.put_pixel(x, *y, miss_colour);
            }
        }
    }

    // vertical lines
    let boundary_width = (height as f32 * 0.05).ceil() as u32;

    for x in [top_left.x(), bottom_right.x()].iter() {
        // if any pixel either size of the vertical line
        let start =if boundary_width > *x {
            0
        } else {
            x - boundary_width
        };
        let end = if x + boundary_width > width {
            width
        } else {
            x + boundary_width
        };
        for y in (top_left.y() + 1)..(bottom_right.y() - 1) {
            if (start..end)
                .any(|x| *buffer.get_pixel(x, y) == line_colour)
            {
                hits += 1;
                buffer.put_pixel(*x, y, hit_colour);
            } else {
                buffer.put_pixel(*x, y, miss_colour);
            }
        }
    }

    let circumference = (bottom_right.1 - top_left.1) * 2 + (bottom_right.0 + top_left.0) * 2;

    let percent = dbg!(hits as f32 / circumference as f32);

    percent > 0.9
}

#[cfg(test)]
mod tests {
    use super::{detect, get_corners, Point};
    use crate::edge::Canny;

    #[test]
    fn test_get_corners() {
        let test_box = (Point(2, 2), Point(38, 54));

        assert_eq!(get_corners(40, 56), test_box);
        // smallest edge used matches test_box
        assert_eq!(get_corners(49, 56), test_box);
        assert_eq!(get_corners(40, 59), test_box);
        // smallest edge used does not match test_box
        assert_ne!(get_corners(39, 56), test_box);
        assert_ne!(get_corners(40, 54), test_box);
    }

    #[test]
    fn test_draw_edges() {
        let mut img = image::open("test_images/uno-7.jpg").unwrap().to_rgba();
        let mut canny = Canny::new(
            img.width() as usize,
            img.height() as usize,
            150.0,
            300.0,
            255,
            false,
        );

        canny.detect(&mut img);

        img.save("test_images/uno-7-with-edges.jpg")
            .expect("write file with edges");
    }

    #[test]
    fn test_detect_boundary() {
        let mut img = image::open("test_images/uno-7.jpg").unwrap().to_rgba();
        let mut canny = Canny::new(
            img.width() as usize,
            img.height() as usize,
            150.0,
            300.0,
            255,
            false,
        );

        canny.detect(&mut img);

        assert!(detect(&mut img, canny.line_colour()));

        img.save("test_images/uno-7-with-edges-and-boundary.jpg")
            .expect("write file with edges");
    }
}
