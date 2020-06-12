
pub type Point = [usize; 2];

#[derive(Debug, PartialEq, Clone)]
pub struct Rectangle(pub [Point; 2]);

impl Rectangle {
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
        (self.0[0][0])..(self.0[1][0])
    }

    pub fn y_range(&self) -> std::ops::Range<usize> {
        (self.0[0][1])..(self.0[1][1])
    }

    pub fn width(&self) -> usize {
        self.0[1][0] - self.0[0][0]
    }

    pub fn height(&self) -> usize {
        self.0[1][1] - self.0[0][1]
    }

    pub fn grow(&self, n: usize) -> Self {
        Self([[self.0[0][0] - n, self.0[0][1] - n], [self.0[1][0] + n, self.0[1][1] + n]])
    }

    pub fn shrink(&self, n: usize) -> Self {
        Self([[self.0[0][0] + n, self.0[0][1] + n], [self.0[1][0] - n, self.0[1][1] - n]])
    }
}
