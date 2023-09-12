use crate::math::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use web_sys::Path2d;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SnapType {
    Geometry(usize, usize),
    Middle(usize, [usize; 2]),
}

#[derive(Clone)]
pub enum ShapeType {
    Line(Vec<XY>),
    QuadBezier(Vec<XY>),
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
        }
    }

    pub fn get_handle(&self, idx: usize) -> XY {
        match &self.handles {
            ShapeType::Line(handles) => handles[idx] + self.offset,
            ShapeType::QuadBezier(handles) => handles[idx] + self.offset,
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
                                    *snap = match (idx1, idx2) {
                                        (0, 2) => snap_h_v_45_135(
                                            handles,
                                            &idx1,
                                            &idx2,
                                            self.handle_selected as usize == idx2,
                                            self.snap_val,
                                        ),
                                        (0, 1) => snap_h_v_45_135(
                                            handles,
                                            &idx1,
                                            &idx2,
                                            self.handle_selected as usize == idx2,
                                            self.snap_val,
                                        ),
                                        (2, 1) => snap_h_v_45_135(
                                            handles,
                                            &idx1,
                                            &idx2,
                                            self.handle_selected as usize == idx2,
                                            self.snap_val,
                                        ),
                                        _ => !unreachable!(),
                                    }
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
                            snap_to_grid(&mut handles[1], grid_spacing);
                            if handles[0].x == handles[1].x
                                && handles[0].y == handles[1].y
                                && handles[2].x == handles[1].x
                                && handles[2].y == handles[1].y
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
        }
    }
    pub fn remove_selection(&mut self) {
        self.handle_selected = -2;
    }
    pub fn get_snaps(&self) -> &Vec<(SnapType, SegmentSnapping)> {
        &self.snaps
    }
}

// #[derive(Clone)]
// pub struct QuadBezierShape {
//     offset: XY,
//     handles: [XY; 3],
//     snap_segment: HashMap<(usize, usize), SegmentSnapping>,
//     handle_selected: i32,
//     snap_val: f64,
//     init: bool,
// }
// impl QuadBezierShape {
//     pub fn new(start: XY, mid: XY, end: XY, snap_val: f64) -> QuadBezierShape {
//         QuadBezierShape {
//             offset: start,
//             handles: [XY::default(), mid - start, end - start],
//             snap_segment: {
//                 let mut tmp = HashMap::new();
//                 tmp.insert((0, 2), SegmentSnapping::None);
//                 tmp.insert((0, 1), SegmentSnapping::None);
//                 tmp.insert((1, 2), SegmentSnapping::None);
//                 tmp
//             },
//             handle_selected: 2,
//             snap_val,
//             init: true,
//         }
//     }
// }

// impl Shape for QuadBezierShape {
//     fn get_handle(&self, idx: usize) -> XY {
//         self.handles[idx] + self.offset
//     }
//     fn get_handle_selected(&self) -> i32 {
//         self.handle_selected
//     }
//     fn move_to(&mut self, p: &XY) {
//         self.offset += *p;
//     }
//     fn modify(&mut self, _p: &XY, dp: &XY) {
//         if self.handle_selected == -1 {
//             self.move_to(dp)
//         } else {
//             // self.handle_selected can be only 0, 1 or 2
//             // Update position
//             self.handles[self.handle_selected as usize] += *dp;
//             if self.handle_selected != 1 {
//                 let start = self.handles[0];
//                 let end = self.handles[2];
//                 if self.init {
//                     set_egual_dist(&mut self.handles[1], &start, &end);
//                 }
//             }
//             // Detect and set snapping
//             for (couple, snap) in self.snap_segment.iter_mut() {
//                 if self.handle_selected as usize == couple.0
//                     || self.handle_selected as usize == couple.1
//                 {
//                     let snap_segment_end = if self.handle_selected as usize == couple.0 {
//                         false
//                     } else {
//                         true
//                     };

//                     let mut seg = HandleSegment {
//                         start: self.handles[couple.0],
//                         end: self.handles[couple.1],
//                     };
//                     *snap = snap_segment(&mut seg, snap_segment_end, self.snap_val);

//                     self.handles[couple.0] = seg.start;
//                     self.handles[couple.1] = seg.end;
//                 }
//             }
//         }
//     }
//     fn get_path_shape(&self) -> Path2d {
//         let start = self.handles[0] + self.offset;
//         let mid = self.handles[1] + self.offset;
//         let end = self.handles[2] + self.offset;
//         let p = Path2d::new().unwrap();
//         p.move_to(start.x, start.y);
//         p.quadratic_curve_to(mid.x, mid.y, end.x, end.y);
//         p
//     }
//     fn get_handles_positions(&self) -> Vec<(XY, bool)> {
//         let mut handles_pos: Vec<(XY, bool)> = Vec::new();
//         match self.handle_selected {
//             -1 => {
//                 handles_pos.push((self.handles[0] + self.offset, false));
//                 handles_pos.push((self.handles[1] + self.offset, false));
//                 handles_pos.push((self.handles[2] + self.offset, false));
//             }
//             0 => {
//                 handles_pos.push((self.handles[0] + self.offset, true));
//                 handles_pos.push((self.handles[1] + self.offset, false));
//                 handles_pos.push((self.handles[2] + self.offset, false));
//             }
//             1 => {
//                 handles_pos.push((self.handles[0] + self.offset, false));
//                 handles_pos.push((self.handles[1] + self.offset, true));
//                 handles_pos.push((self.handles[2] + self.offset, false));
//             }
//             2 => {
//                 handles_pos.push((self.handles[0] + self.offset, false));
//                 handles_pos.push((self.handles[1] + self.offset, false));
//                 handles_pos.push((self.handles[2] + self.offset, true));
//             }
//             _ => (),
//         }
//         handles_pos
//     }
//     fn snap(&mut self, grid_spacing: f64) {
//         self.init = false;
//         if self.handle_selected == -1 {
//             self.offset = snap_to_grid(&self.offset, grid_spacing);
//         } else {
//             if self.handle_selected == 0 {
//                 self.handles[0] = snap_to_grid(&self.handles[0], grid_spacing);
//                 if self.handles[0].x == self.handles[2].x && self.handles[0].y == self.handles[2].y
//                 {
//                     self.handles[0].x += grid_spacing;
//                     self.handles[0].y += grid_spacing;
//                 }
//             } else {
//                 if self.handle_selected == 1 {
//                     self.handles[1] = snap_to_grid(&self.handles[1], grid_spacing);
//                     if self.handles[0].x == self.handles[1].x
//                         && self.handles[0].y == self.handles[1].y
//                         && self.handles[2].x == self.handles[1].x
//                         && self.handles[2].y == self.handles[1].y
//                     {
//                         self.handles[1].x += grid_spacing;
//                         self.handles[1].y += grid_spacing;
//                     }
//                 } else {
//                     if self.handle_selected == 2 {
//                         self.handles[2] = snap_to_grid(&self.handles[2], grid_spacing);
//                         if self.handles[0].x == self.handles[2].x
//                             && self.handles[0].y == self.handles[2].y
//                         {
//                             self.handles[2].x += grid_spacing;
//                             self.handles[2].y += grid_spacing;
//                         }
//                     }
//                 }
//             }
//         }
//     }
//     fn valid(&self) -> bool {
//         self.handles[0].x != self.handles[2].x || self.handles[0].y != self.handles[2].y
//     }
//     fn set_selection(&mut self, pos: &XY, precision: f64) {
//         let start = self.handles[0];
//         let mid = self.handles[1];
//         let end = self.handles[2];
//         if is_point_on_point(pos, &(start + self.offset), precision) {
//             self.handle_selected = 0;
//         } else {
//             if is_point_on_point(pos, &(mid + self.offset), precision) {
//                 if self.handle_selected > -2 {
//                     self.handle_selected = 1;
//                 }
//             } else {
//                 if is_point_on_point(pos, &(end + self.offset), precision) {
//                     self.handle_selected = 2;
//                 } else {
//                     if is_point_on_quadbezier(
//                         pos,
//                         &(start + self.offset),
//                         &(mid + self.offset),
//                         &(end + self.offset),
//                         precision,
//                     ) {
//                         self.handle_selected = -1;
//                     } else {
//                         self.handle_selected = -2;
//                     }
//                 }
//             }
//         }
//     }
//     fn remove_selection(&mut self) {
//         self.handle_selected = -2;
//     }
//     fn get_snaps(&self) -> &HashMap<(usize, usize), SegmentSnapping> {
//         &self.snap_segment
//     }
// }

// // Simple types
// #[derive(Copy, Clone, Default)]
// pub struct Handle {
//     pub pos: XY,
//     pub function: HandleFunction,
// }

// impl Handle {
//     pub fn to_seg(h1: Handle, h2: Handle) -> HandleSegment {
//         HandleSegment { start: h1, end: h2 }
//     }
// }

// #[derive(Copy, Clone)]
// pub struct Precision {
//     e: XY,
//     se: f64,
// }

// #[derive(Copy, Clone)]
// pub struct HandleSegment {
//     pub start: Handle,
//     pub end: Handle,
// }
// impl Default for HandleSegment {
//     fn default() -> Self {
//         HandleSegment {
//             start: Handle::default(),
//             end: Handle::default(),
//         }
//     }
// }

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
