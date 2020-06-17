pub type Point = [usize; 2];

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Rectangle(pub [Point; 2]);

impl Rectangle {
    pub fn new(top_left: Point, bottom_right: Point) -> Self {
        assert!(
            bottom_right[0] > top_left[0] && bottom_right[1] > top_left[1],
            "bottom right must be down and to the right of top left"
        );
        Self([top_left, bottom_right])
    }

    pub fn from_dimensions(width: usize, height: usize) -> Self {
        Self::new([0, 0], [width, height])
    }

    pub fn top_left(&self) -> &Point {
        &self.0[0]
    }

    pub fn bottom_right(&self) -> &Point {
        &self.0[1]
    }

    pub fn iter(&self) -> std::slice::Iter<Point> {
        self.0.iter()
    }

    pub fn x_range(&self) -> std::ops::Range<usize> {
        (self.top_left()[0])..(self.bottom_right()[0])
    }

    pub fn y_range(&self) -> std::ops::Range<usize> {
        (self.top_left()[1])..(self.bottom_right()[1])
    }

    pub fn width(&self) -> usize {
        self.bottom_right()[0] - self.top_left()[0]
    }

    pub fn height(&self) -> usize {
        self.bottom_right()[1] - self.top_left()[1]
    }

    pub fn grow(&self, n: usize) -> Self {
        assert!(
            n <= self.top_left()[0] && n <= self.top_left()[1],
            "cannot grow beyond (0, 0)"
        );
        Self::new(
            [self.top_left()[0] - n, self.top_left()[1] - n],
            [self.bottom_right()[0] + n, self.bottom_right()[1] + n],
        )
    }

    pub fn clamped_grow(&self, n: usize, rect: &Rectangle) -> Self {
        Self::new([
            clamp_sub(self.top_left()[0], n, rect.top_left()[0]),
            clamp_sub(self.top_left()[1], n, rect.top_left()[1]),
        ], [
            clamp_add(self.bottom_right()[0], n, rect.bottom_right()[0]),
            clamp_add(self.bottom_right()[1], n, rect.bottom_right()[1]),
        ])
    }

    pub fn shrink(&self, n: usize) -> Self {
        assert!(
            n <= self.bottom_right()[0] && n <= self.bottom_right()[1],
            "cannot shring beyong (0, 0)"
        );
        Self::new(
            [self.top_left()[0] + n, self.top_left()[1] + n],
            [self.bottom_right()[0] - n, self.bottom_right()[1] - n],
        )
    }

    pub fn clamped_shrink(&self, n: usize, rect: &Rectangle) -> Self {
        Self::new([
            clamp_add(self.top_left()[0], n, rect.bottom_right()[0]),
            clamp_add(self.top_left()[1], n, rect.bottom_right()[1]),
        ], [
            clamp_sub(self.bottom_right()[0], n, rect.top_left()[0]),
            clamp_sub(self.bottom_right()[1], n, rect.top_left()[1]),
        ])
    }

    pub fn contains(&self, other: &Rectangle) -> bool {
        self[0][0] < other[0][0]
            && self[0][1] < other[0][1]
            && self[1][0] > other[1][0]
            && self[1][1] > other[1][1]
    }

    pub fn draw<P: image::Pixel + 'static>(&self, img: &mut image::ImageBuffer<P, Vec<P::Subpixel>>, colour: P) {
        for y in self.iter().map(|&p| p[1]) {
            for x in self.x_range() {
                img.put_pixel(x as u32, y as u32, colour);
            }
        }

        for x in self.iter().map(|&p| p[0]) {
            for y in self.y_range() {
                img.put_pixel(x as u32, y as u32, colour);
            }
        }
    }
}

fn clamp_sub(value: usize, n: usize, min: usize) -> usize {
    if n + min > value {
        min
    } else {
        value - n
    }
}

fn clamp_add(value: usize, n: usize, max: usize) -> usize {
    std::cmp::min(value + n + 1, max) - 1
}

impl std::ops::Index<usize> for Rectangle {
    type Output = Point;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[cfg(test)]
mod tests {
    use super::Rectangle;

    #[test]
    fn test_contains() {
        let rect = Rectangle([[5, 5], [10, 10]]);

        assert!(rect.contains(&Rectangle([[6, 6], [9, 9]])));
        assert!(!rect.contains(&Rectangle([[4, 6], [9, 9]])));
    }
}
