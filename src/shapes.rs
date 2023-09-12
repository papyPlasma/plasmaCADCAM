use crate::math::*;
use std::ops::{Add, AddAssign, Sub};
use web_sys::{console, Path2d};

pub trait Shape {
    //
    fn get_handle_selected(&self) -> i32;
    //
    fn move_to(&mut self, p: &XY);
    fn modify(&mut self, p: &XY, dp: &XY);
    fn get_path_shape(&self) -> Path2d;
    fn get_position_handles(&self) -> Vec<(XY, bool)>;
    fn snap(&mut self, grid_spacing: f64);
    fn valid(&self) -> bool;
    fn set_selection(&mut self, pos: &XY, precision: f64);
    fn remove_selection(&mut self);
}

pub enum Shapes {
    Line(LineShape),
    // Arc,
    // QuadBezier,
    // CubicBezier,
    // Square,
    // Circle,
}
impl Clone for Shapes {
    fn clone(&self) -> Shapes {
        use Shapes::*;
        match self {
            Line(line) => Shapes::Line(line.clone()),
        }
    }
}

impl Shape for Shapes {
    fn get_handle_selected(&self) -> i32 {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.get_handle_selected(),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
    fn move_to(&mut self, p: &XY) {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.move_to(p),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
    fn modify(&mut self, p: &XY, dp: &XY) {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.modify(p, dp),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
    fn get_path_shape(&self) -> Path2d {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.get_path_shape(),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
    fn get_position_handles(&self) -> Vec<(XY, bool)> {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.get_position_handles(),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
    fn snap(&mut self, grid_spacing: f64) {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.snap(grid_spacing),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
    fn valid(&self) -> bool {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.valid(),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
    fn set_selection(&mut self, pos: &XY, precision: f64) {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.set_selection(pos, precision),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
    fn remove_selection(&mut self) {
        use Shapes::*;
        match self {
            Line(line_shape) => line_shape.remove_selection(),
            // Arc => (),
            // QuadBezier => (),
            // CubicBezier => (),
            // Square => (),
            // Circle => (),
        }
    }
}

#[derive(Copy, Clone)]
pub struct LineShape {
    offset: XY,
    handles: [XY; 2],
    snap_couples: [(usize, usize); 1],
    handle_selected: i32,
    snap_val: f64,
}
impl LineShape {
    pub fn new(start: XY, end: XY, snap_val: f64) -> LineShape {
        LineShape {
            offset: start,
            handles: [XY::default(), end - start],
            snap_couples: [(0, 1); 1],
            handle_selected: 1,
            snap_val,
        }
    }
}

impl Shape for LineShape {
    fn get_handle_selected(&self) -> i32 {
        self.handle_selected
    }
    fn move_to(&mut self, p: &XY) {
        self.offset += *p;
    }
    fn modify(&mut self, _p: &XY, dp: &XY) {
        if self.handle_selected == -1 {
            self.move_to(dp)
        } else {
            // Update position
            self.handles[self.handle_selected as usize] += *dp;
            // Detect and set snapping
            for couple in self.snap_couples.iter() {
                if self.handle_selected as usize == couple.0
                    || self.handle_selected as usize == couple.1
                {
                    let mut seg = Segment {
                        start: self.handles[couple.0],
                        end: self.handles[couple.1],
                    };
                    use SegmentSnapping::*;
                    match if self.handle_selected == 1 {
                        snap_segment(&mut seg, true, self.snap_val)
                    } else {
                        snap_segment(&mut seg, false, self.snap_val)
                    } {
                        None => (),
                        Horizontal => (),
                        Vertical => (),
                        Diagonal45 => (),
                        Diagonal135 => (),
                    }
                    self.handles[couple.0] = seg.start;
                    self.handles[couple.1] = seg.end;
                }
            }
        }
    }
    fn get_path_shape(&self) -> Path2d {
        // let snap_val = 2.;
        // let stroke_default: &str = pa_ref.stroke_default.as_ref();
        // let stroke_selected: &str = pa_ref.stroke_selected.as_ref();
        // let stroke_light: &str = pa_ref.stroke_light.as_ref();
        // pa_ref.ctx.set_stroke_style(&stroke_default.into());
        let start = self.handles[0];
        let end = self.handles[1];
        // Draw non cutting things
        // if (self.handle_selected > -2) {
        //     self.drawDimension(start, end);
        //     self.drawVertical(start, end, snapVal, 50);
        //     self.draw45(start, end);
        //     self.draw135(start, end);
        //     if (start.y < end.y) {
        //         if (!self.drawHorizontal(start, end, snapVal, 50))
        //             self.drawAngle(start, end);
        //     } else
        //         if (!self.drawHorizontal(start, end, snapVal, 50))
        //             self.drawAngle(end, start);
        // }

        // Draw actual line
        let p = Path2d::new().unwrap();
        let mut pos = self.offset;
        pos += start;
        p.move_to(pos.x, pos.y);
        let mut pos = self.offset;
        pos += end;
        p.line_to(pos.x, pos.y);
        p
    }
    fn get_position_handles(&self) -> Vec<(XY, bool)> {
        let mut handles_pos: Vec<(XY, bool)> = Vec::new();
        match self.handle_selected {
            -1 => {
                handles_pos.push((self.handles[0] + self.offset, false));
                handles_pos.push((self.handles[1] + self.offset, false));
            }
            0 => {
                handles_pos.push((self.handles[0] + self.offset, true));
                handles_pos.push((self.handles[1] + self.offset, false));
            }
            1 => {
                handles_pos.push((self.handles[0] + self.offset, false));
                handles_pos.push((self.handles[1] + self.offset, true));
            }
            _ => (),
        }
        handles_pos
    }
    fn snap(&mut self, grid_spacing: f64) {
        if self.handle_selected == -1 {
            self.offset = snap_to_grid(&self.offset, grid_spacing);
        } else {
            if self.handle_selected == 0 {
                self.handles[0] = snap_to_grid(&self.handles[0], grid_spacing);
                if self.handles[0].x == self.handles[1].x && self.handles[0].y == self.handles[1].y
                {
                    self.handles[0].x += grid_spacing;
                    self.handles[0].y += grid_spacing;
                }
            } else {
                if self.handle_selected == 1 {
                    self.handles[1] = snap_to_grid(&self.handles[1], grid_spacing);
                    if self.handles[0].x == self.handles[1].x
                        && self.handles[0].y == self.handles[1].y
                    {
                        self.handles[1].x += grid_spacing;
                        self.handles[1].y += grid_spacing;
                    }
                }
            }
        }
    }
    fn valid(&self) -> bool {
        self.handles[0].x != self.handles[1].x || self.handles[0].y != self.handles[1].y
    }
    fn set_selection(&mut self, pos: &XY, precision: f64) {
        let start = self.handles[0];
        let end = self.handles[1];
        if is_point_on_point(pos, &(start + self.offset), precision) {
            self.handle_selected = 0;
        } else {
            if is_point_on_point(pos, &(end + self.offset), precision) {
                self.handle_selected = 1;
            } else {
                if is_point_on_segment(pos, &(start + self.offset), &(end + self.offset), precision)
                {
                    self.handle_selected = -1;
                } else {
                    self.handle_selected = -2;
                }
            }
        }
    }
    fn remove_selection(&mut self) {
        self.handle_selected = -2;
    }
}

// Simple types
#[derive(Copy, Clone)]
pub struct Segment {
    pub start: XY,
    pub end: XY,
}
impl Default for Segment {
    fn default() -> Self {
        Segment {
            start: XY::default(),
            end: XY::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct XY {
    pub x: f64,
    pub y: f64,
}
impl Default for XY {
    fn default() -> Self {
        XY { x: 0.0, y: 0.0 }
    }
}
impl AddAssign for XY {
    fn add_assign(&mut self, other: XY) {
        self.x += other.x;
        self.y += other.y;
    }
}
impl Add for XY {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
impl Sub for XY {
    type Output = XY;
    fn sub(self, other: XY) -> XY {
        XY {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

pub struct Precision {
    e: XY,
    se: f64,
}
