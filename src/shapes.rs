use crate::math::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use web_sys::{console, Path2d};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SnapType {
    Geometry(usize, usize),
    Middle(usize, [usize; 2]),
}

#[derive(Clone)]
pub enum ShapeType {
    Line(Vec<XY>),
    QuadBezier(Vec<XY>),
    CubicBezier(Vec<XY>),
}

#[derive(Copy, Clone)]
pub enum SegmentSnapping {
    None,
    Horizontal,
    Vertical,
    Diagonal45,
    Diagonal135,
    Middle,
}

#[derive(Clone)]
pub struct Shape {
    offset: XY,
    handles: ShapeType,
    snaps: Vec<(SnapType, SegmentSnapping)>,
    handle_selected: i32,
    snap_val: f64,
    init: bool,
}

impl Shape {
    pub fn new(shape_type: ShapeType, snap_val: f64) -> Shape {
        match shape_type {
            ShapeType::Line(handles) => Shape {
                offset: handles[0],
                handles: ShapeType::Line(vec![XY::default(), handles[1] - handles[0]]),
                snaps: {
                    let mut tmp = Vec::new();
                    tmp.push((SnapType::Geometry(0, 1), SegmentSnapping::None));
                    tmp
                },
                handle_selected: 1,
                snap_val,
                init: true,
            },
            ShapeType::QuadBezier(handles) => Shape {
                offset: handles[0],
                handles: ShapeType::QuadBezier(vec![
                    XY::default(),
                    handles[1] - handles[0],
                    handles[2] - handles[0],
                ]),
                snaps: {
                    let mut tmp = Vec::new();
                    tmp.push((SnapType::Geometry(0, 2), SegmentSnapping::None));
                    tmp.push((SnapType::Geometry(0, 1), SegmentSnapping::None));
                    tmp.push((SnapType::Geometry(2, 1), SegmentSnapping::None));
                    tmp.push((SnapType::Middle(1, [0, 2]), SegmentSnapping::None));
                    tmp
                },
                handle_selected: 2,
                snap_val,
                init: true,
            },
            ShapeType::CubicBezier(handles) => Shape {
                offset: handles[0],
                handles: ShapeType::CubicBezier(vec![
                    XY::default(),
                    handles[1] - handles[0],
                    handles[2] - handles[0],
                    handles[3] - handles[0],
                ]),
                snaps: {
                    let mut tmp = Vec::new();
                    tmp.push((SnapType::Geometry(0, 3), SegmentSnapping::None));
                    tmp.push((SnapType::Geometry(0, 1), SegmentSnapping::None));
                    tmp.push((SnapType::Geometry(2, 3), SegmentSnapping::None));

                    tmp.push((SnapType::Geometry(1, 2), SegmentSnapping::None));
                    tmp.push((SnapType::Geometry(1, 3), SegmentSnapping::None));
                    tmp.push((SnapType::Geometry(2, 3), SegmentSnapping::None));
                    tmp.push((SnapType::Geometry(2, 0), SegmentSnapping::None));
                    // tmp.push((SnapType::Middle(1, [0, 2]), SegmentSnapping::None));
                    tmp
                },
                handle_selected: 3,
                snap_val,
                init: true,
            },
        }
    }

    pub fn get_handle(&self, idx: usize) -> XY {
        match &self.handles {
            ShapeType::Line(handles) => handles[idx] + self.offset,
            ShapeType::QuadBezier(handles) => handles[idx] + self.offset,
            ShapeType::CubicBezier(handles) => handles[idx] + self.offset,
        }
    }
    pub fn get_handle_selected(&self) -> i32 {
        self.handle_selected
    }
    pub fn move_to(&mut self, p: &XY) {
        self.offset += *p;
    }
    pub fn modify(&mut self, _p: &XY, dp: &XY) {
        if self.handle_selected == -1 {
            self.move_to(dp)
        } else {
            // Update position
            match &mut self.handles {
                ShapeType::Line(handles) => handles[self.handle_selected as usize] += *dp,
                ShapeType::QuadBezier(handles) => {
                    if self.init {
                        handles[2] += *dp;
                        handles[1] = (handles[0] + handles[2]) / 2.;
                    } else {
                        handles[self.handle_selected as usize] += *dp
                    }
                }
                ShapeType::CubicBezier(handles) => {
                    if self.init {
                        handles[3] += *dp;
                        handles[1] = (handles[0] + handles[3]) / 3.;
                        handles[2] = (handles[0] + handles[3]) / 3. * 2.;
                    } else {
                        handles[self.handle_selected as usize] += *dp
                    }
                }
            }

            // Detect and set snapping
            for (snap_type, snap) in self.snaps.iter_mut() {
                match snap_type {
                    SnapType::Geometry(idx1, idx2) => {
                        let idx1 = *idx1;
                        let idx2 = *idx2;
                        if self.handle_selected as usize == idx1
                            || self.handle_selected as usize == idx2
                        {
                            match &mut self.handles {
                                ShapeType::Line(handles) => {
                                    *snap = snap_h_v_45_135(
                                        handles,
                                        &idx1,
                                        &idx2,
                                        self.handle_selected == 1,
                                        self.snap_val,
                                    );
                                }
                                #[allow(unreachable_code)]
                                ShapeType::QuadBezier(handles) => {
                                    *snap = snap_h_v_45_135(
                                        handles,
                                        &idx1,
                                        &idx2,
                                        self.handle_selected as usize == idx2,
                                        self.snap_val,
                                    );
                                }
                                ShapeType::CubicBezier(handles) => {
                                    *snap = snap_h_v_45_135(
                                        handles,
                                        &idx1,
                                        &idx2,
                                        self.handle_selected as usize == idx2,
                                        self.snap_val,
                                    );
                                }
                            }
                        }
                    }
                    SnapType::Middle(idx_middle, idxs) => {
                        if self.handle_selected as usize == *idx_middle {
                            match &mut self.handles {
                                ShapeType::Line(_) => (),
                                ShapeType::QuadBezier(handles) => {
                                    *snap =
                                        snap_equidistant(handles, idx_middle, idxs, self.snap_val);
                                }
                                ShapeType::CubicBezier(_) => (),
                            }
                        } else {
                            *snap = SegmentSnapping::None;
                        }
                    }
                }
            }
        }
    }
    pub fn get_path_shape(&self) -> Path2d {
        match &self.handles {
            ShapeType::Line(handles) => {
                let start = handles[0] + self.offset;
                let end = handles[1] + self.offset;
                let p = Path2d::new().unwrap();
                p.move_to(start.x, start.y);
                p.line_to(end.x, end.y);
                p
            }
            ShapeType::QuadBezier(handles) => {
                let start = handles[0] + self.offset;
                let ctrl = handles[1] + self.offset;
                let end = handles[2] + self.offset;
                let p = Path2d::new().unwrap();
                p.move_to(start.x, start.y);
                p.quadratic_curve_to(ctrl.x, ctrl.y, end.x, end.y);
                p
            }
            ShapeType::CubicBezier(handles) => {
                let start = handles[0] + self.offset;
                let ctrl1 = handles[1] + self.offset;
                let ctrl2 = handles[2] + self.offset;
                let end = handles[3] + self.offset;
                let p = Path2d::new().unwrap();
                p.move_to(start.x, start.y);
                p.bezier_curve_to(ctrl1.x, ctrl1.y, ctrl2.x, ctrl2.y, end.x, end.y);
                p
            }
        }
    }
    pub fn get_handles_positions(&self) -> Vec<(XY, bool)> {
        let mut handles_pos: Vec<(XY, bool)> = Vec::new();
        match &self.handles {
            ShapeType::Line(handles) => match self.handle_selected {
                -1 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, false));
                }
                0 => {
                    handles_pos.push((handles[0] + self.offset, true));
                    handles_pos.push((handles[1] + self.offset, false));
                }
                1 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, true));
                }
                _ => (),
            },
            ShapeType::QuadBezier(handles) => match self.handle_selected {
                -1 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, false));
                    handles_pos.push((handles[2] + self.offset, false));
                }
                0 => {
                    handles_pos.push((handles[0] + self.offset, true));
                    handles_pos.push((handles[1] + self.offset, false));
                    handles_pos.push((handles[2] + self.offset, false));
                }
                1 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, true));
                    handles_pos.push((handles[2] + self.offset, false));
                }
                2 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, false));
                    handles_pos.push((handles[2] + self.offset, true));
                }
                _ => (),
            },
            ShapeType::CubicBezier(handles) => match self.handle_selected {
                -1 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, false));
                    handles_pos.push((handles[2] + self.offset, false));
                    handles_pos.push((handles[3] + self.offset, false));
                }
                0 => {
                    handles_pos.push((handles[0] + self.offset, true));
                    handles_pos.push((handles[1] + self.offset, false));
                    handles_pos.push((handles[2] + self.offset, false));
                    handles_pos.push((handles[3] + self.offset, false));
                }
                1 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, true));
                    handles_pos.push((handles[2] + self.offset, false));
                    handles_pos.push((handles[3] + self.offset, false));
                }
                2 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, false));
                    handles_pos.push((handles[2] + self.offset, true));
                    handles_pos.push((handles[3] + self.offset, false));
                }
                3 => {
                    handles_pos.push((handles[0] + self.offset, false));
                    handles_pos.push((handles[1] + self.offset, false));
                    handles_pos.push((handles[2] + self.offset, false));
                    handles_pos.push((handles[3] + self.offset, true));
                }
                _ => (),
            },
        }

        handles_pos
    }
    pub fn snap(&mut self, grid_spacing: f64) {
        self.init = false;
        if self.handle_selected == -1 {
            snap_to_grid(&mut self.offset, grid_spacing);
        } else {
            match &mut self.handles {
                ShapeType::Line(handles) => {
                    if self.handle_selected == 0 {
                        snap_to_grid(&mut handles[0], grid_spacing);
                        if handles[0].x == handles[1].x && handles[0].y == handles[1].y {
                            handles[0].x += grid_spacing;
                            handles[0].y += grid_spacing;
                        }
                    } else {
                        if self.handle_selected == 1 {
                            snap_to_grid(&mut handles[1], grid_spacing);
                            if handles[0].x == handles[1].x && handles[0].y == handles[1].y {
                                handles[1].x += grid_spacing;
                                handles[1].y += grid_spacing;
                            }
                        }
                    }
                }
                ShapeType::QuadBezier(handles) => {
                    if self.handle_selected == 0 {
                        snap_to_grid(&mut handles[0], grid_spacing);
                        if handles[0].x == handles[1].x && handles[0].y == handles[1].y {
                            handles[0].x += grid_spacing;
                            handles[0].y += grid_spacing;
                        }
                    } else {
                        if self.handle_selected == 1 {
                            if (handles[0].x == handles[1].x && handles[0].y == handles[1].y)
                                || (handles[2].x == handles[1].x && handles[2].y == handles[1].y)
                            {
                                handles[1].x += grid_spacing;
                                handles[1].y += grid_spacing;
                            }
                        } else {
                            if self.handle_selected == 2 {
                                snap_to_grid(&mut handles[2], grid_spacing);
                                if handles[0].x == handles[2].x && handles[0].y == handles[2].y {
                                    handles[2].x += grid_spacing;
                                    handles[2].y += grid_spacing;
                                }
                            }
                        }
                    }
                }
                ShapeType::CubicBezier(handles) => {
                    if self.handle_selected == 0 {
                        snap_to_grid(&mut handles[0], grid_spacing);
                        if handles[0].x == handles[1].x && handles[0].y == handles[1].y {
                            handles[0].x += grid_spacing;
                            handles[0].y += grid_spacing;
                        }
                    } else {
                        if self.handle_selected == 1 {
                            if (handles[0].x == handles[1].x && handles[0].y == handles[1].y)
                                || (handles[2].x == handles[1].x && handles[2].y == handles[1].y)
                            {
                                handles[1].x += grid_spacing;
                                handles[1].y += grid_spacing;
                            }
                        } else {
                            if self.handle_selected == 2 {
                                if (handles[0].x == handles[2].x && handles[0].y == handles[2].y)
                                    || (handles[3].x == handles[2].x
                                        && handles[3].y == handles[2].y)
                                {
                                    handles[2].x += grid_spacing;
                                    handles[2].y += grid_spacing;
                                }
                            } else {
                                if self.handle_selected == 3 {
                                    snap_to_grid(&mut handles[3], grid_spacing);
                                    if handles[0].x == handles[3].x && handles[0].y == handles[3].y
                                    {
                                        handles[3].x += grid_spacing;
                                        handles[3].y += grid_spacing;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn valid(&self) -> bool {
        match &self.handles {
            ShapeType::Line(handles) => {
                handles[0].x != handles[1].x || handles[0].y != handles[1].y
            }
            ShapeType::QuadBezier(handles) => {
                handles[0].x != handles[2].x || handles[0].y != handles[2].y
            }
            ShapeType::CubicBezier(handles) => {
                handles[0].x != handles[3].x || handles[0].y != handles[3].y
            }
        }
    }
    pub fn set_selection(&mut self, pos: &XY, precision: f64) {
        match &mut self.handles {
            ShapeType::Line(handles) => {
                let start = handles[0];
                let end = handles[1];
                if is_point_on_point(pos, &(start + self.offset), precision) {
                    self.handle_selected = 0;
                } else {
                    if is_point_on_point(pos, &(end + self.offset), precision) {
                        self.handle_selected = 1;
                    } else {
                        if is_point_on_segment(
                            pos,
                            &(start + self.offset),
                            &(end + self.offset),
                            precision,
                        ) {
                            self.handle_selected = -1;
                        } else {
                            self.handle_selected = -2;
                        }
                    }
                }
            }
            ShapeType::QuadBezier(handles) => {
                let start = handles[0];
                let mid = handles[1];
                let end = handles[2];
                if is_point_on_point(pos, &(start + self.offset), precision) {
                    self.handle_selected = 0;
                } else {
                    if is_point_on_point(pos, &(mid + self.offset), precision) {
                        if self.handle_selected > -2 {
                            self.handle_selected = 1;
                        }
                    } else {
                        if is_point_on_point(pos, &(end + self.offset), precision) {
                            self.handle_selected = 2;
                        } else {
                            if is_point_on_quadbezier(
                                pos,
                                &(start + self.offset),
                                &(mid + self.offset),
                                &(end + self.offset),
                                precision,
                            ) {
                                self.handle_selected = -1;
                            } else {
                                self.handle_selected = -2;
                            }
                        }
                    }
                }
            }
            ShapeType::CubicBezier(handles) => {
                let start = handles[0];
                let ctrl1 = handles[1];
                let ctrl2 = handles[2];
                let end = handles[3];
                if is_point_on_point(pos, &(start + self.offset), precision) {
                    self.handle_selected = 0;
                } else {
                    if is_point_on_point(pos, &(ctrl1 + self.offset), precision) {
                        if self.handle_selected > -2 {
                            self.handle_selected = 1;
                        }
                    } else {
                        if is_point_on_point(pos, &(ctrl2 + self.offset), precision) {
                            if self.handle_selected > -2 {
                                self.handle_selected = 2;
                            }
                        } else {
                            if is_point_on_point(pos, &(end + self.offset), precision) {
                                self.handle_selected = 3;
                            } else {
                                if is_point_on_cubicbezier(
                                    pos,
                                    &(start + self.offset),
                                    &(ctrl1 + self.offset),
                                    &(ctrl2 + self.offset),
                                    &(end + self.offset),
                                    precision,
                                ) {
                                    self.handle_selected = -1;
                                } else {
                                    self.handle_selected = -2;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn remove_selection(&mut self) {
        self.handle_selected = -2;
    }
    pub fn get_snaps(&self) -> &Vec<(SnapType, SegmentSnapping)> {
        &self.snaps
    }
}

#[derive(Copy, Clone)]
pub struct XY {
    pub x: f64,
    pub y: f64,
}
impl XY {
    pub fn dist(&self, other: &XY) -> f64 {
        let dpt = *self - *other;
        (dpt.x * dpt.x + dpt.y * dpt.y).sqrt()
    }
    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
}

impl Default for XY {
    fn default() -> Self {
        XY { x: 0.0, y: 0.0 }
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
impl AddAssign for XY {
    fn add_assign(&mut self, other: XY) {
        self.x += other.x;
        self.y += other.y;
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
impl SubAssign for XY {
    fn sub_assign(&mut self, other: XY) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Div<f64> for XY {
    type Output = XY;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        XY {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}
impl DivAssign<f64> for XY {
    fn div_assign(&mut self, rhs: f64) {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Mul<f64> for XY {
    type Output = XY;

    fn mul(self, rhs: f64) -> Self::Output {
        XY {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}
impl MulAssign<f64> for XY {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}
