use crate::math::*;
use std::f64::consts::PI;

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum DimensionComplexity {
    Simple(usize, usize),
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum DimensionType {
    Linear(f64, i32, DimensionComplexity),
    Horizontal(f64, i32, DimensionComplexity),
    Vertical(f64, i32, DimensionComplexity),
    Radius(f64, i32, DimensionComplexity),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum ConstructionType {
    Move(WXY),
    Line(WXY),
    Quadratic(WXY, WXY),
    Bezier(WXY, WXY, WXY),
    Ellipse(WXY, WXY, f64, f64, f64, bool),
    Rectangle(WXY, WXY, bool),
    // Text(WXY, String),
}

#[derive(Clone)]
pub enum ShapeType {
    Line(Vec<WXY>),
    QuadBezier(Vec<WXY>),
    CubicBezier(Vec<WXY>),
    Square(Vec<WXY>),
    Circle(Vec<WXY>),
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

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum SnapType {
    Geometry(usize, usize),
    Middle(usize, [usize; 2]),
}

#[derive(Clone)]
pub struct Shape {
    offset: WXY,
    shape: ShapeType,
    snaps: Vec<(SnapType, SegmentSnapping)>,
    handle_selected: i32,
    snap_val: f64,
    handles_size: WXY,
    init: bool,
}

impl Shape {
    pub fn new(shape: ShapeType, snap_val: f64, handles_size: f64) -> Shape {
        match shape {
            ShapeType::Line(handles) => Shape {
                offset: handles[0],
                shape: ShapeType::Line(vec![WXY::default(), handles[1] - handles[0]]),
                snaps: {
                    let mut tmp = Vec::new();
                    tmp.push((SnapType::Geometry(0, 1), SegmentSnapping::None));
                    tmp
                },
                handle_selected: 1,
                snap_val,
                handles_size: WXY {
                    wx: handles_size,
                    wy: handles_size,
                },
                init: true,
            },
            ShapeType::QuadBezier(handles) => Shape {
                offset: handles[0],
                shape: ShapeType::QuadBezier(vec![
                    WXY::default(),
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
                handles_size: WXY {
                    wx: handles_size,
                    wy: handles_size,
                },
                init: true,
            },
            ShapeType::CubicBezier(handles) => Shape {
                offset: handles[0],
                shape: ShapeType::CubicBezier(vec![
                    WXY::default(),
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
                handles_size: WXY {
                    wx: handles_size,
                    wy: handles_size,
                },
                init: true,
            },
            ShapeType::Square(handles) => Shape {
                offset: handles[0],
                shape: ShapeType::Square(vec![WXY::default(), handles[1] - handles[0]]),
                snaps: {
                    let mut tmp = Vec::new();
                    tmp.push((SnapType::Geometry(0, 1), SegmentSnapping::None));
                    tmp
                },
                handle_selected: 1,
                snap_val,
                handles_size: WXY {
                    wx: handles_size,
                    wy: handles_size,
                },
                init: true,
            },
            ShapeType::Circle(handles) => Shape {
                offset: handles[0],
                shape: ShapeType::Circle(vec![WXY::default(), handles[1] - handles[0]]),
                snaps: {
                    let mut tmp = Vec::new();
                    tmp.push((SnapType::Geometry(0, 1), SegmentSnapping::None));
                    tmp
                },
                handle_selected: 1,
                snap_val,
                handles_size: WXY {
                    wx: handles_size,
                    wy: handles_size,
                },
                init: true,
            },
        }
    }
    pub fn get_handle_selected(&self) -> i32 {
        self.handle_selected
    }
    pub fn is_selected(&self) -> bool {
        self.handle_selected > -2
    }
    pub fn set_handle_selected(&mut self, idx: i32) {
        self.handle_selected = idx;
    }
    pub fn move_to(&mut self, p: &WXY) {
        self.offset += *p;
    }
    pub fn modify(&mut self, _p: &WXY, dp: &WXY) {
        if self.handle_selected == -1 {
            self.move_to(dp)
        } else {
            // Update position
            match &mut self.shape {
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
                ShapeType::Square(handles) => handles[self.handle_selected as usize] += *dp,
                ShapeType::Circle(handles) => handles[self.handle_selected as usize] += *dp,
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
                            match &mut self.shape {
                                ShapeType::Line(handles) => {
                                    *snap = snap_h_v_45_135(
                                        handles,
                                        &idx1,
                                        &idx2,
                                        self.handle_selected == 1,
                                        self.snap_val,
                                    );
                                }
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
                                ShapeType::Square(handles) => {
                                    *snap = snap_h_v_45_135(
                                        handles,
                                        &idx1,
                                        &idx2,
                                        self.handle_selected == 1,
                                        self.snap_val,
                                    );
                                }
                                ShapeType::Circle(handles) => {
                                    *snap = snap_h_v_45_135(
                                        handles,
                                        &idx1,
                                        &idx2,
                                        self.handle_selected == 1,
                                        self.snap_val,
                                    );
                                }
                            }
                        }
                    }
                    SnapType::Middle(idx_middle, idxs) => {
                        if self.handle_selected as usize == *idx_middle {
                            match &mut self.shape {
                                ShapeType::Line(_) => (),
                                ShapeType::QuadBezier(handles) => {
                                    *snap =
                                        snap_equidistant(handles, idx_middle, idxs, self.snap_val);
                                }
                                ShapeType::CubicBezier(_) => (),
                                ShapeType::Square(_) => (),
                                ShapeType::Circle(_) => (),
                            }
                        } else {
                            *snap = SegmentSnapping::None;
                        }
                    }
                }
            }
        }
    }
    pub fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        match &self.shape {
            ShapeType::Line(handles) => {
                let start = handles[0] + self.offset;
                let end = handles[1] + self.offset;
                cst.push(Move(start));
                cst.push(Line(end));
                cst
            }
            ShapeType::QuadBezier(handles) => {
                let start = handles[0] + self.offset;
                let ctrl = handles[1] + self.offset;
                let end = handles[2] + self.offset;
                cst.push(Move(start));
                cst.push(Quadratic(ctrl, end));
                cst
            }
            ShapeType::CubicBezier(handles) => {
                let start = handles[0] + self.offset;
                let ctrl1 = handles[1] + self.offset;
                let ctrl2 = handles[2] + self.offset;
                let end = handles[3] + self.offset;
                cst.push(Move(start));
                cst.push(Bezier(ctrl1, ctrl2, end));
                cst
            }
            ShapeType::Square(handles) => {
                let start = handles[0] + self.offset;
                let end = handles[1] + self.offset;
                cst.push(Move(start));
                cst.push(Line(WXY {
                    wx: start.wx,
                    wy: end.wy,
                }));
                cst.push(Line(WXY {
                    wx: end.wx,
                    wy: end.wy,
                }));
                cst.push(Line(WXY {
                    wx: end.wx,
                    wy: start.wy,
                }));
                cst.push(Line(WXY {
                    wx: start.wx,
                    wy: start.wy,
                }));
                cst
            }
            ShapeType::Circle(handles) => {
                let center = handles[0] + self.offset;
                let radius = WXY {
                    wx: (handles[1].wx - handles[0].wx).abs(),
                    wy: (handles[1].wy - handles[0].wy).abs(),
                };
                cst.push(Move(
                    center
                        + WXY {
                            wx: radius.wx,
                            wy: 0.,
                        },
                ));
                cst.push(Ellipse(center, radius, 0., 0., 2. * PI, false));
                cst
            }
        }
    }
    pub fn get_handles_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        match &self.shape {
            ShapeType::Line(handles)
            | ShapeType::QuadBezier(handles)
            | ShapeType::CubicBezier(handles)
            | ShapeType::Square(handles)
            | ShapeType::Circle(handles) => {
                use ConstructionType::*;
                for (idx, handle) in handles.iter().enumerate() {
                    let start = *handle - self.handles_size / 2. + self.offset;
                    cst.push(Move(start));
                    cst.push(Rectangle(
                        start,
                        self.handles_size,
                        idx as i32 == self.handle_selected,
                    ));
                }
            }
        }
        cst
    }
    pub fn get_snap_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        for (snap_type, segment_snaping) in self.snaps.iter() {
            match snap_type {
                SnapType::Geometry(idx1, idx2) => match &self.shape {
                    ShapeType::Line(handles)
                    | ShapeType::QuadBezier(handles)
                    | ShapeType::CubicBezier(handles)
                    | ShapeType::Square(handles)
                    | ShapeType::Circle(handles) => {
                        use SegmentSnapping::*;
                        match segment_snaping {
                            Horizontal | Vertical | Diagonal45 | Diagonal135 => {
                                use ConstructionType::*;
                                let handle1 = handles[*idx1];
                                let handle2 = handles[*idx2];
                                let start = handle1 + self.offset;
                                let end = handle2 + self.offset;
                                extend_points(&mut [start, end]);
                                cst.push(Move(start));
                                cst.push(Line(end));
                            }
                            _ => (),
                        }
                    }
                },
                SnapType::Middle(_idx_mid, _idxs) => (),
            }
        }
        cst
    }
    pub fn snap(&mut self, grid_spacing: f64) {
        self.init = false;
        if self.handle_selected == -1 {
            snap_to_grid(&mut self.offset, grid_spacing);
        } else {
            match &mut self.shape {
                ShapeType::Line(handles) => {
                    if self.handle_selected == 0 {
                        snap_to_grid(&mut handles[0], grid_spacing);
                        if handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy {
                            handles[0].wx += grid_spacing;
                            handles[0].wy += grid_spacing;
                        }
                    } else {
                        if self.handle_selected == 1 {
                            snap_to_grid(&mut handles[1], grid_spacing);
                            if handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy {
                                handles[1].wx += grid_spacing;
                                handles[1].wy += grid_spacing;
                            }
                        }
                    }
                }
                ShapeType::QuadBezier(handles) => {
                    if self.handle_selected == 0 {
                        snap_to_grid(&mut handles[0], grid_spacing);
                        if handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy {
                            handles[0].wx += grid_spacing;
                            handles[0].wy += grid_spacing;
                        }
                    } else {
                        if self.handle_selected == 1 {
                            if (handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy)
                                || (handles[2].wx == handles[1].wx
                                    && handles[2].wy == handles[1].wy)
                            {
                                handles[1].wx += grid_spacing;
                                handles[1].wy += grid_spacing;
                            }
                        } else {
                            if self.handle_selected == 2 {
                                snap_to_grid(&mut handles[2], grid_spacing);
                                if handles[0].wx == handles[2].wx && handles[0].wy == handles[2].wy
                                {
                                    handles[2].wx += grid_spacing;
                                    handles[2].wy += grid_spacing;
                                }
                            }
                        }
                    }
                }
                ShapeType::CubicBezier(handles) => {
                    if self.handle_selected == 0 {
                        snap_to_grid(&mut handles[0], grid_spacing);
                        if handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy {
                            handles[0].wx += grid_spacing;
                            handles[0].wy += grid_spacing;
                        }
                    } else {
                        if self.handle_selected == 1 {
                            if (handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy)
                                || (handles[2].wx == handles[1].wx
                                    && handles[2].wy == handles[1].wy)
                            {
                                handles[1].wx += grid_spacing;
                                handles[1].wy += grid_spacing;
                            }
                        } else {
                            if self.handle_selected == 2 {
                                if (handles[0].wx == handles[2].wx
                                    && handles[0].wy == handles[2].wy)
                                    || (handles[3].wx == handles[2].wx
                                        && handles[3].wy == handles[2].wy)
                                {
                                    handles[2].wx += grid_spacing;
                                    handles[2].wy += grid_spacing;
                                }
                            } else {
                                if self.handle_selected == 3 {
                                    snap_to_grid(&mut handles[3], grid_spacing);
                                    if handles[0].wx == handles[3].wx
                                        && handles[0].wy == handles[3].wy
                                    {
                                        handles[3].wx += grid_spacing;
                                        handles[3].wy += grid_spacing;
                                    }
                                }
                            }
                        }
                    }
                }
                ShapeType::Square(handles) => {
                    if self.handle_selected == 0 {
                        snap_to_grid(&mut handles[0], grid_spacing);
                        if handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy {
                            handles[0].wx += grid_spacing;
                            handles[0].wy += grid_spacing;
                        }
                    } else {
                        if self.handle_selected == 1 {
                            snap_to_grid(&mut handles[1], grid_spacing);
                            if handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy {
                                handles[1].wx += grid_spacing;
                                handles[1].wy += grid_spacing;
                            }
                        }
                    }
                }
                ShapeType::Circle(handles) => {
                    if self.handle_selected == 0 {
                        snap_to_grid(&mut handles[0], grid_spacing);
                        if handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy {
                            handles[0].wx += grid_spacing;
                            handles[0].wy += grid_spacing;
                        }
                    } else {
                        if self.handle_selected == 1 {
                            snap_to_grid(&mut handles[1], grid_spacing);
                            if handles[0].wx == handles[1].wx && handles[0].wy == handles[1].wy {
                                handles[1].wx += grid_spacing;
                                handles[1].wy += grid_spacing;
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn valid(&self) -> bool {
        match &self.shape {
            ShapeType::Line(handles) => {
                handles[0].wx != handles[1].wx || handles[0].wy != handles[1].wy
            }
            ShapeType::QuadBezier(handles) => {
                handles[0].wx != handles[2].wx || handles[0].wy != handles[2].wy
            }
            ShapeType::CubicBezier(handles) => {
                handles[0].wx != handles[3].wx || handles[0].wy != handles[3].wy
            }
            ShapeType::Square(handles) => {
                handles[0].wx != handles[1].wx || handles[0].wy != handles[1].wy
            }
            ShapeType::Circle(handles) => {
                handles[0].wx != handles[1].wx || handles[0].wy != handles[1].wy
            }
        }
    }
    pub fn set_selection(&mut self, pos: &WXY, precision: f64) {
        match &mut self.shape {
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
            ShapeType::Square(handles) => {
                let start = handles[0];
                let end = handles[1];
                if is_point_on_point(pos, &(start + self.offset), precision) {
                    self.handle_selected = 0;
                } else {
                    if is_point_on_point(pos, &(end + self.offset), precision) {
                        self.handle_selected = 1;
                    } else {
                        let tl = WXY {
                            wx: start.wx,
                            wy: end.wy,
                        };
                        let br = WXY {
                            wx: end.wx,
                            wy: start.wy,
                        };
                        if is_point_on_segment(
                            pos,
                            &(start + self.offset),
                            &(tl + self.offset),
                            precision,
                        ) || is_point_on_segment(
                            pos,
                            &(tl + self.offset),
                            &(end + self.offset),
                            precision,
                        ) || is_point_on_segment(
                            pos,
                            &(end + self.offset),
                            &(br + self.offset),
                            precision,
                        ) || is_point_on_segment(
                            pos,
                            &(br + self.offset),
                            &(start + self.offset),
                            precision,
                        ) {
                            self.handle_selected = -1;
                        } else {
                            self.handle_selected = -2;
                        }
                    }
                }
            }
            ShapeType::Circle(handles) => {
                let start = handles[0];
                let end = handles[1];
                if is_point_on_point(pos, &(start + self.offset), precision) {
                    if self.handle_selected > -2 {
                        self.handle_selected = 0;
                    }
                } else {
                    if is_point_on_point(pos, &(end + self.offset), precision) {
                        if self.handle_selected > -2 {
                            self.handle_selected = 1;
                        }
                    } else {
                        let center = handles[0] + self.offset;
                        let radius = WXY {
                            wx: (handles[1].wx - handles[0].wx).abs(),
                            wy: (handles[1].wy - handles[0].wy).abs(),
                        };
                        if is_point_on_ellipse(pos, &center, &radius, precision) {
                            self.handle_selected = -1;
                        } else {
                            self.handle_selected = -2;
                        }
                    }
                }
            }
        }
    }
    pub fn remove_selection(&mut self) {
        self.handle_selected = -2;
    }
    pub fn get_bounding_box(&self) -> [WXY; 2] {
        match &self.shape {
            ShapeType::Line(handles) => {
                let start = handles[0] + self.offset;
                let end = handles[1] + self.offset;
                [start, end]
            }
            ShapeType::QuadBezier(handles) => {
                let start = handles[0] + self.offset;
                let _ctrl = handles[1] + self.offset;
                let end = handles[2] + self.offset;
                [start, end]
            }
            ShapeType::CubicBezier(handles) => {
                let start = handles[0] + self.offset;
                let end = handles[1] + self.offset;
                [start, end]
            }
            ShapeType::Square(handles) => {
                let start = handles[0] + self.offset;
                let end = handles[1] + self.offset;
                [start, end]
            }
            ShapeType::Circle(handles) => {
                let start = handles[0] + self.offset;
                let end = handles[1] + self.offset;
                [start, end]
            }
        }
    }
}
