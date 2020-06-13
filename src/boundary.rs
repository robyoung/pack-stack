use crate::data::Rectangle;
use image::{Rgba, RgbaImage};

/// ratio taken from standard card dimensions of 2.5 x 3.5
const RATIO: f32 = 5.0 / 7.0;

/// 5% margin between edge of frame and corner lines
const MARGIN: f32 = 0.05;

pub(crate) fn get_corners(width: u32, height: u32) -> Rectangle {
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

pub(crate) fn detect(buffer: &mut RgbaImage, line_colour: Rgba<u8>) -> bool {
    let width = buffer.width();
    let height = buffer.height();
    let mut hits = 0;

    let rectangle = get_corners(width, height);
    let miss_colour = Rgba([255, 0, 0, 255]);
    let hit_colour = Rgba([0, 255, 0, 255]);

    // horizontal lines
    let boundary_width = (width as f32 * 0.05).ceil() as u32;

    for y in rectangle.iter().map(|p| p[1] as u32) {
        // if any pixel either side of the horizontal line
        let start = if boundary_width > y {
            0
        } else {
            y - boundary_width
        };
        let end = if y + boundary_width > height {
            height
        } else {
            y + boundary_width
        };
        for x in rectangle.x_range().map(|x| x as u32) {
            if (start..end).any(|y| *buffer.get_pixel(x, y) == line_colour) {
                hits += 1;
                buffer.put_pixel(x, y, hit_colour);
            } else {
                buffer.put_pixel(x, y, miss_colour);
            }
        }
    }

    // vertical lines
    let boundary_width = (height as f32 * 0.05).ceil() as u32;

    for x in rectangle.iter().map(|p| p[0] as u32) {
        // if any pixel either size of the vertical line
        let start = if boundary_width > x {
            0
        } else {
            x - boundary_width
        };
        let end = if x + boundary_width > width {
            width
        } else {
            x + boundary_width
        };
        for y in rectangle.shrink(1).y_range().map(|y| y as u32) {
            if (start..end).any(|x| *buffer.get_pixel(x, y) == line_colour) {
                hits += 1;
                buffer.put_pixel(x, y, hit_colour);
            } else {
                buffer.put_pixel(x, y, miss_colour);
            }
        }
    }

    let circumference = rectangle.width() * 2 + rectangle.height() * 2;

    let percent = dbg!(hits as f32 / circumference as f32);

    percent > 0.9
}

#[cfg(test)]
mod tests {
    extern crate test;

    use test::Bencher;

    use crate::data::Rectangle;
    use crate::edge::{CannyBuilder, RectangleInRectangleWindow};

    use super::{detect, get_corners};

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
    fn test_draw_edges() {
        let mut img = image::open("test_images/uno-7.jpg").unwrap().to_rgba();
        let mut canny = CannyBuilder::new(img.width() as usize, img.height() as usize).build();

        canny.detect(&mut img);

        img.save("test_images/uno-7-with-edges.jpg")
            .expect("write file with edges");
    }

    #[test]
    fn test_detect_boundary() {
        let mut img = image::open("test_images/uno-7.jpg").unwrap().to_rgba();
        let mut canny = CannyBuilder::new(img.width() as usize, img.height() as usize).build();

        canny.detect(&mut img);

        assert!(detect(&mut img, canny.line_colour()));

        img.save("test_images/uno-7-with-edges-and-boundary.jpg")
            .expect("write file with edges");
    }

    #[bench]
    fn bench_detect_boundary(b: &mut Bencher) {
        let img = image::open("test_images/uno-7.jpg").unwrap().to_rgba();
        let width = img.width();
        let height = img.height();

        let rect = get_corners(width, height);
        let window = RectangleInRectangleWindow::new(
            rect.grow(6),
            rect.shrink(7),
        );

        let mut canny = CannyBuilder::with_window(width as usize, height as usize, window).build();

        b.iter(|| {
            let mut img = img.clone();
            canny.detect(&mut img);
            assert!(detect(&mut img, canny.line_colour()));
        });
    }
}
