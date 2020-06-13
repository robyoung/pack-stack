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

    pub fn contains(&self, other: &Rectangle) -> bool {
        self[0][0] < other[0][0]
            && self[0][1] < other[0][1]
            && self[1][0] > other[1][0]
            && self[1][1] > other[1][1]
    }
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
