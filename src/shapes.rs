use web_sys::console;

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
    Text(WXY, String),
}

pub trait Snap {
    fn snap_geometry(&mut self, p: &WXY, dp: &WXY, snap_precision: f64);
    fn get_selection(&self) -> HandleSelection;
    fn set_selection(&mut self, handle_selection: HandleSelection);
    fn set_selection_from_position(&mut self, p: &WXY, precision: f64);
    fn remove_any_selection(&mut self);
    fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64);
    fn get_construction(&self) -> Vec<ConstructionType>;
    fn get_handles_construction(&self, size_handle: WXY) -> Vec<ConstructionType>;
    fn get_bounding_box(&self) -> [WXY; 2];
    fn init_done(&mut self);
}

#[derive(Copy, Clone)]
pub enum HandleSelection {
    None,
    Start,
    Ctrl,
    Ctrl1,
    Ctrl2,
    MidTop,
    MidRight,
    End,
    Center,
    All,
}

#[derive(Copy, Clone)]
pub struct SLine {
    start: WXY,
    end: WXY,
    selection: HandleSelection,
    tmp: WXY,
}
impl SLine {
    pub fn new(start: WXY, end: WXY) -> SLine {
        SLine {
            start,
            end,
            selection: HandleSelection::End,
            tmp: WXY::default(),
        }
    }
}
impl Snap for SLine {
    fn snap_geometry(&mut self, p: &WXY, dp: &WXY, snap_precision: f64) {
        use HandleSelection::*;
        match self.selection {
            Start => snap_h_v_45_135(&self.end, &mut self.start, snap_precision),
            End => snap_h_v_45_135(&self.start, &mut self.end, snap_precision),
            _ => (),
        }
    }
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn set_selection_from_position(&mut self, p: &WXY, precision: f64) {
        if is_point_on_point(p, &self.start, precision) {
            self.selection = HandleSelection::Start;
        } else {
            if is_point_on_point(p, &self.end, precision) {
                self.selection = HandleSelection::End;
            } else {
                if is_point_on_segment(p, &self.start, &self.end, precision) {
                    self.selection = HandleSelection::All;
                } else {
                    self.selection = HandleSelection::None;
                }
            }
        }
    }
    fn remove_any_selection(&mut self) {
        self.selection = HandleSelection::None;
    }
    fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64) {
        use HandleSelection::*;
        match self.selection {
            Start => {
                self.start = *p;
                snap_to_grid(&mut self.start, snap_distance);
            }
            End => {
                self.end = *p;
                snap_to_grid(&mut self.end, snap_distance);
            }
            All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_grid_x(&mut self.tmp, snap_distance);
                    self.start.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_grid_y(&mut self.tmp, snap_distance);
                    self.start.wy += self.tmp.wy;
                    self.end.wy += self.tmp.wy;
                    self.tmp.wy = 0.;
                }
            }
            _ => (),
        }
    }
    fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        cst.push(Move(self.start));
        cst.push(Line(self.end));
        cst
    }
    fn get_handles_construction(&self, size_handle: WXY) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        use HandleSelection::*;
        match self.selection {
            Start => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, true));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            End => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, true));
            }
            All => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            _ => (),
        }
        cst
    }
    fn get_bounding_box(&self) -> [WXY; 2] {
        [self.start, self.end]
    }
    fn init_done(&mut self) {}
}

#[derive(Copy, Clone)]
pub struct SQuadBezier {
    start: WXY,
    ctrl: WXY,
    end: WXY,
    selection: HandleSelection,
    init: bool,
    tmp: WXY,
}
impl SQuadBezier {
    pub fn new(start: WXY, ctrl: WXY, end: WXY) -> SQuadBezier {
        SQuadBezier {
            start,
            ctrl,
            end,
            selection: HandleSelection::End,
            init: true,
            tmp: WXY::default(),
        }
    }
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
}
impl Snap for SQuadBezier {
    fn snap_geometry(&mut self, p: &WXY, dp: &WXY, snap_precision: f64) {
        use HandleSelection::*;
        match self.selection {
            Start => {
                snap_h_v_45_135(&self.end, &mut self.start, snap_precision);
                snap_h_v_45_135(&self.ctrl, &mut self.start, snap_precision);
            }
            Ctrl => {
                snap_h_v_45_135(&self.end, &mut self.ctrl, snap_precision);
                snap_h_v_45_135(&self.start, &mut self.ctrl, snap_precision);
            }
            End => {
                snap_h_v_45_135(&self.start, &mut self.end, snap_precision);
                snap_h_v_45_135(&self.ctrl, &mut self.end, snap_precision);
            }
            _ => (),
        }
    }
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn set_selection_from_position(&mut self, p: &WXY, precision: f64) {
        if is_point_on_point(p, &self.start, precision) {
            self.selection = HandleSelection::Start;
        } else {
            if is_point_on_point(p, &self.ctrl, precision) {
                self.selection = HandleSelection::Ctrl;
            } else {
                if is_point_on_point(p, &self.end, precision) {
                    self.selection = HandleSelection::End;
                } else {
                    if is_point_on_quadbezier(p, &self.start, &self.ctrl, &self.end, precision) {
                        self.selection = HandleSelection::All;
                    } else {
                        self.selection = HandleSelection::None;
                    }
                }
            }
        }
    }
    fn remove_any_selection(&mut self) {
        self.selection = HandleSelection::None;
    }
    fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64) {
        use HandleSelection::*;
        match self.selection {
            Start => {
                self.start = *p;
                snap_to_grid(&mut self.start, snap_distance);
            }
            Ctrl => {
                self.ctrl += *dp;
            }
            End => {
                self.end = *p;
                snap_to_grid(&mut self.end, snap_distance);
                if self.init {
                    self.ctrl = (self.start + self.end) / 2.;
                }
            }
            All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_grid_x(&mut self.tmp, snap_distance);
                    self.start.wx += self.tmp.wx;
                    self.ctrl.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_grid_y(&mut self.tmp, snap_distance);
                    self.start.wy += self.tmp.wy;
                    self.ctrl.wy += self.tmp.wy;
                    self.end.wy += self.tmp.wy;
                    self.tmp.wy = 0.;
                }
            }
            _ => (),
        }
    }
    fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        cst.push(Move(self.start));
        cst.push(Quadratic(self.ctrl, self.end));
        cst
    }
    fn get_handles_construction(&self, size_handle: WXY) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        use HandleSelection::*;
        match self.selection {
            Start => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, true));
                cst.push(Rectangle(self.ctrl - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            Ctrl => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl - size_handle / 2., size_handle, true));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            End => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, true));
            }
            All => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            _ => (),
        }
        cst
    }
    fn get_bounding_box(&self) -> [WXY; 2] {
        [self.start, self.end]
    }
    fn init_done(&mut self) {
        self.init = false;
    }
}

#[derive(Copy, Clone)]
pub struct SCubicBezier {
    start: WXY,
    ctrl1: WXY,
    ctrl2: WXY,
    end: WXY,
    selection: HandleSelection,
    init: bool,
    tmp: WXY,
}
impl SCubicBezier {
    pub fn new(start: WXY, ctrl1: WXY, ctrl2: WXY, end: WXY) -> SCubicBezier {
        SCubicBezier {
            start,
            ctrl1,
            ctrl2,
            end,
            selection: HandleSelection::End,
            init: true,
            tmp: WXY::default(),
        }
    }
}
impl Snap for SCubicBezier {
    fn snap_geometry(&mut self, p: &WXY, dp: &WXY, snap_precision: f64) {
        use HandleSelection::*;
        match self.selection {
            Start => {
                snap_h_v_45_135(&self.ctrl1, &mut self.start, snap_precision);
                snap_h_v_45_135(&self.ctrl2, &mut self.start, snap_precision);
                snap_h_v_45_135(&self.end, &mut self.start, snap_precision);
            }
            Ctrl1 => {
                snap_h_v_45_135(&self.start, &mut self.ctrl1, snap_precision);
                snap_h_v_45_135(&self.ctrl2, &mut self.ctrl1, snap_precision);
                snap_h_v_45_135(&self.end, &mut self.ctrl1, snap_precision);
            }
            Ctrl2 => {
                snap_h_v_45_135(&self.start, &mut self.ctrl2, snap_precision);
                snap_h_v_45_135(&self.ctrl1, &mut self.ctrl2, snap_precision);
                snap_h_v_45_135(&self.end, &mut self.ctrl2, snap_precision);
            }
            End => {
                snap_h_v_45_135(&self.start, &mut self.end, snap_precision);
                snap_h_v_45_135(&self.ctrl1, &mut self.end, snap_precision);
                snap_h_v_45_135(&self.ctrl2, &mut self.end, snap_precision);
            }
            _ => (),
        }
    }
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn set_selection_from_position(&mut self, p: &WXY, precision: f64) {
        if is_point_on_point(p, &self.start, precision) {
            self.selection = HandleSelection::Start;
        } else {
            if is_point_on_point(p, &self.ctrl1, precision) {
                self.selection = HandleSelection::Ctrl1;
            } else {
                if is_point_on_point(p, &self.ctrl2, precision) {
                    self.selection = HandleSelection::Ctrl2;
                } else {
                    if is_point_on_point(p, &self.end, precision) {
                        self.selection = HandleSelection::End;
                    } else {
                        if is_point_on_cubicbezier(
                            p,
                            &self.start,
                            &self.ctrl1,
                            &self.ctrl2,
                            &self.end,
                            precision,
                        ) {
                            self.selection = HandleSelection::All;
                        } else {
                            self.selection = HandleSelection::None;
                        }
                    }
                }
            }
        }
    }
    fn remove_any_selection(&mut self) {
        self.selection = HandleSelection::None;
    }
    fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64) {
        use HandleSelection::*;
        match self.selection {
            Start => {
                self.start = *p;
                snap_to_grid(&mut self.start, snap_distance);
            }
            Ctrl1 => {
                self.ctrl1 += *dp;
            }
            Ctrl2 => {
                self.ctrl2 += *dp;
            }
            End => {
                self.end = *p;
                snap_to_grid(&mut self.end, snap_distance);
                if self.init {
                    self.ctrl1 = (self.start + self.end) / 3.;
                    self.ctrl2 = (self.start + self.end) / 3. * 2.;
                }
            }
            All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_grid_x(&mut self.tmp, snap_distance);
                    self.start.wx += self.tmp.wx;
                    self.ctrl1.wx += self.tmp.wx;
                    self.ctrl2.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_grid_y(&mut self.tmp, snap_distance);
                    self.start.wy += self.tmp.wy;
                    self.ctrl1.wy += self.tmp.wy;
                    self.ctrl2.wy += self.tmp.wy;
                    self.end.wy += self.tmp.wy;
                    self.tmp.wy = 0.;
                }
            }
            _ => (),
        }
    }
    fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        cst.push(Move(self.start));
        cst.push(Bezier(self.ctrl1, self.ctrl2, self.end));
        cst
    }
    fn get_handles_construction(&self, size_handle: WXY) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        use HandleSelection::*;
        match self.selection {
            Start => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, true));
                cst.push(Rectangle(self.ctrl1 - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl2 - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            Ctrl1 => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl1 - size_handle / 2., size_handle, true));
                cst.push(Rectangle(self.ctrl2 - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            Ctrl2 => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl1 - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl2 - size_handle / 2., size_handle, true));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            End => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl1 - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl2 - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, true));
            }
            All => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl1 - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.ctrl2 - size_handle / 2., size_handle, false));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            _ => (),
        }
        cst
    }
    fn get_bounding_box(&self) -> [WXY; 2] {
        [self.start, self.end]
    }
    fn init_done(&mut self) {
        self.init = false;
    }
}

#[derive(Copy, Clone)]
pub struct SRectangle {
    start: WXY,
    mid_top: WXY,
    mid_right: WXY,
    end: WXY,
    selection: HandleSelection,
    tmp: WXY,
}
impl SRectangle {
    pub fn new(start: WXY, dim: WXY) -> SRectangle {
        SRectangle {
            start,
            mid_top: WXY {
                wx: start.wx + dim.wx / 2.,
                wy: start.wy + dim.wy,
            },
            mid_right: WXY {
                wx: start.wx + dim.wx,
                wy: start.wy + dim.wy / 2.,
            },
            end: WXY {
                wx: start.wx + dim.wx,
                wy: start.wy + dim.wy,
            },
            selection: HandleSelection::End,
            tmp: WXY::default(),
        }
    }
}
impl Snap for SRectangle {
    fn snap_geometry(&mut self, p: &WXY, dp: &WXY, snap_precision: f64) {
        use HandleSelection::*;
        match self.selection {
            Start => {
                snap_h_v_45_135(&self.end, &mut self.start, snap_precision);
            }
            MidTop => {
                // snap_h_v_45_135(&self.start, &mut self.end, snap_precision);
                // self.mid_top.wy = self.end.wy;
            }
            MidRight => {
                // snap_h_v_45_135(&self.start, &mut self.end, snap_precision);
                // self.mid_right.wx = self.end.wx;
            }
            End => {
                snap_h_v_45_135(&self.start, &mut self.end, snap_precision);
            }
            _ => (),
        }
    }
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn set_selection_from_position(&mut self, p: &WXY, precision: f64) {
        if is_point_on_point(p, &self.start, precision) {
            self.selection = HandleSelection::Start;
        } else {
            if is_point_on_point(p, &self.mid_top, precision) {
                self.selection = HandleSelection::MidTop;
            } else {
                if is_point_on_point(p, &self.mid_right, precision) {
                    self.selection = HandleSelection::MidRight;
                } else {
                    if is_point_on_point(p, &self.end, precision) {
                        self.selection = HandleSelection::End;
                    } else {
                        let tl = WXY {
                            wx: self.start.wx,
                            wy: self.end.wy,
                        };
                        let br = WXY {
                            wx: self.end.wx,
                            wy: self.start.wy,
                        };
                        if is_point_on_segment(p, &self.start, &tl, precision)
                            || is_point_on_segment(p, &tl, &self.end, precision)
                            || is_point_on_segment(p, &self.end, &br, precision)
                            || is_point_on_segment(p, &br, &self.start, precision)
                        {
                            self.selection = HandleSelection::All;
                        } else {
                            self.selection = HandleSelection::None;
                        }
                    }
                }
            }
        }
    }
    fn remove_any_selection(&mut self) {
        self.selection = HandleSelection::None;
    }
    fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64) {
        use HandleSelection::*;
        match self.selection {
            Start => {
                self.start = *p;
                snap_to_grid(&mut self.start, snap_distance);
                self.mid_top.wx = (self.start.wx + self.end.wx) / 2.;
                self.mid_top.wy = self.end.wy;
                self.mid_right.wx = self.end.wx;
                self.mid_right.wy = (self.start.wy + self.end.wy) / 2.;
            }
            MidTop => {
                self.mid_top.wy = p.wy;
                snap_to_grid_y(&mut self.mid_top, snap_distance);
                self.end.wy = self.mid_top.wy;
                self.mid_right.wy = (self.start.wy + self.end.wy) / 2.;
            }
            MidRight => {
                self.mid_right.wx = p.wx;
                snap_to_grid_x(&mut self.mid_right, snap_distance);
                self.end.wx = self.mid_right.wx;
                self.mid_top.wx = (self.start.wx + self.end.wx) / 2.;
            }
            End => {
                self.end = *p;
                snap_to_grid(&mut self.end, snap_distance);
                self.mid_top.wx = (self.start.wx + self.end.wx) / 2.;
                self.mid_top.wy = self.end.wy;
                self.mid_right.wx = self.end.wx;
                self.mid_right.wy = (self.start.wy + self.end.wy) / 2.;
            }
            All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_grid_x(&mut self.tmp, snap_distance);
                    self.start.wx += self.tmp.wx;
                    self.mid_top.wx += self.tmp.wx;
                    self.mid_right.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_grid_y(&mut self.tmp, snap_distance);
                    self.start.wy += self.tmp.wy;
                    self.mid_top.wy += self.tmp.wy;
                    self.mid_right.wy += self.tmp.wy;
                    self.end.wy += self.tmp.wy;
                    self.tmp.wy = 0.;
                }
            }
            _ => (),
        }
    }
    fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        cst.push(Move(self.start));
        cst.push(Line(WXY {
            wx: self.start.wx,
            wy: self.end.wy,
        }));
        cst.push(Line(WXY {
            wx: self.end.wx,
            wy: self.end.wy,
        }));
        cst.push(Line(WXY {
            wx: self.end.wx,
            wy: self.start.wy,
        }));
        cst.push(Line(WXY {
            wx: self.start.wx,
            wy: self.start.wy,
        }));
        cst
    }
    fn get_handles_construction(&self, size_handle: WXY) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        use HandleSelection::*;
        match self.selection {
            Start => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, true));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            MidTop => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    true,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            MidRight => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    true,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            End => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, true));
            }
            All => {
                cst.push(Rectangle(self.start - size_handle / 2., size_handle, false));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            _ => (),
        }
        cst
    }
    fn get_bounding_box(&self) -> [WXY; 2] {
        [self.start, self.end]
    }
    fn init_done(&mut self) {}
}

#[derive(Copy, Clone)]
pub struct SEllipse {
    center: WXY,
    mid_top: WXY,
    mid_right: WXY,
    end: WXY,
    selection: HandleSelection,
    tmp: WXY,
}
impl SEllipse {
    pub fn new(center: WXY, radius: WXY) -> SEllipse {
        SEllipse {
            center,
            mid_top: WXY {
                wx: center.wx,
                wy: center.wy + radius.wy,
            },
            mid_right: WXY {
                wx: center.wx + radius.wx,
                wy: center.wy,
            },
            end: WXY {
                wx: center.wx + radius.wx,
                wy: center.wy + radius.wy,
            },
            selection: HandleSelection::End,
            tmp: WXY::default(),
        }
    }
}
impl Snap for SEllipse {
    fn snap_geometry(&mut self, _p: &WXY, _dp: &WXY, snap_precision: f64) {
        use HandleSelection::*;
        match self.selection {
            Center => {
                snap_h_v_45_135(&self.end, &mut self.center, snap_precision);
            }
            MidTop => {
                snap_h_v_45_135(&self.mid_right, &mut self.mid_top, snap_precision);
            }
            MidRight => {
                snap_h_v_45_135(&self.mid_top, &mut self.mid_right, snap_precision);
            }
            End => {
                snap_h_v_45_135(&self.center, &mut self.end, snap_precision);
            }
            _ => (),
        }
    }
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn set_selection_from_position(&mut self, p: &WXY, precision: f64) {
        if is_point_on_point(p, &self.center, precision) {
            self.selection = HandleSelection::Center;
        } else {
            if is_point_on_point(p, &self.mid_top, precision) {
                self.selection = HandleSelection::MidTop;
            } else {
                if is_point_on_point(p, &self.mid_right, precision) {
                    self.selection = HandleSelection::MidRight;
                } else {
                    if is_point_on_point(p, &self.end, precision) {
                        self.selection = HandleSelection::End;
                    } else {
                        let radius = self.end - self.center;
                        if is_point_on_ellipse(p, &self.center, &radius, precision) {
                            self.selection = HandleSelection::All;
                        } else {
                            self.selection = HandleSelection::None;
                        }
                    }
                }
            }
        }
    }
    fn remove_any_selection(&mut self) {
        self.selection = HandleSelection::None;
    }
    fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64) {
        use HandleSelection::*;
        match self.selection {
            MidTop => {
                self.mid_top.wy = p.wy;
                snap_to_grid_y(&mut self.mid_top, snap_distance);
                self.end.wy = self.mid_top.wy;
            }
            MidRight => {
                self.mid_right.wx = p.wx;
                snap_to_grid_x(&mut self.mid_right, snap_distance);
                self.end.wx = self.mid_right.wx;
            }
            End => {
                self.end = *p;
                snap_to_grid(&mut self.end, snap_distance);
                self.mid_top.wx = self.center.wx;
                self.mid_top.wy = self.end.wy;
                self.mid_right.wx = self.end.wx;
                self.mid_right.wy = self.center.wy;
            }
            Center | All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_grid_x(&mut self.tmp, snap_distance);
                    self.center.wx += self.tmp.wx;
                    self.mid_top.wx += self.tmp.wx;
                    self.mid_right.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_grid_y(&mut self.tmp, snap_distance);
                    self.center.wy += self.tmp.wy;
                    self.mid_top.wy += self.tmp.wy;
                    self.mid_right.wy += self.tmp.wy;
                    self.end.wy += self.tmp.wy;
                    self.tmp.wy = 0.;
                }
            }
            _ => (),
        }
    }
    fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        let radius = WXY {
            wx: (self.end.wx - self.center.wx).abs(),
            wy: (self.end.wy - self.center.wy).abs(),
        };
        cst.push(Move(
            self.center
                + WXY {
                    wx: radius.wx,
                    wy: 0.,
                },
        ));
        cst.push(Ellipse(self.center, radius, 0., 0., 2. * PI, false));
        cst
    }
    fn get_handles_construction(&self, size_handle: WXY) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use ConstructionType::*;
        use HandleSelection::*;
        match self.selection {
            Center => {
                cst.push(Rectangle(self.center - size_handle / 2., size_handle, true));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            MidTop => {
                cst.push(Rectangle(
                    self.center - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    true,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            MidRight => {
                cst.push(Rectangle(
                    self.center - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    true,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            End => {
                cst.push(Rectangle(
                    self.center - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, true));
            }
            All => {
                cst.push(Rectangle(
                    self.center - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_top - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(
                    self.mid_right - size_handle / 2.,
                    size_handle,
                    false,
                ));
                cst.push(Rectangle(self.end - size_handle / 2., size_handle, false));
            }
            _ => (),
        }
        cst
    }
    fn get_bounding_box(&self) -> [WXY; 2] {
        [self.center - (self.end - self.center), self.end]
    }
    fn init_done(&mut self) {}
}

#[derive(Copy, Clone)]
pub enum ShapeType {
    Line(SLine),
    QuadBezier(SQuadBezier),
    CubicBezier(SCubicBezier),
    Rectangle(SRectangle),
    Ellipse(SEllipse),
}
impl Snap for ShapeType {
    fn snap_geometry(&mut self, p: &WXY, dp: &WXY, snap_precision: f64) {
        use ShapeType::*;
        match self {
            Line(line) => line.snap_geometry(p, dp, snap_precision),
            QuadBezier(quadbezier) => quadbezier.snap_geometry(p, dp, snap_precision),
            CubicBezier(cubicbezier) => cubicbezier.snap_geometry(p, dp, snap_precision),
            Rectangle(rectangle) => rectangle.snap_geometry(p, dp, snap_precision),
            Ellipse(ellipse) => ellipse.snap_geometry(p, dp, snap_precision),
        };
    }

    fn get_selection(&self) -> HandleSelection {
        use ShapeType::*;
        match self {
            Line(line) => line.get_selection(),
            QuadBezier(quadbezier) => quadbezier.get_selection(),
            CubicBezier(cubicbezier) => cubicbezier.get_selection(),
            Rectangle(rectangle) => rectangle.get_selection(),
            Ellipse(ellipse) => ellipse.get_selection(),
        }
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        use ShapeType::*;
        match self {
            Line(line) => line.set_selection(handle_selection),
            QuadBezier(quadbezier) => quadbezier.set_selection(handle_selection),
            CubicBezier(cubicbezier) => cubicbezier.set_selection(handle_selection),
            Rectangle(rectangle) => rectangle.set_selection(handle_selection),
            Ellipse(ellipse) => ellipse.set_selection(handle_selection),
        }
    }
    fn set_selection_from_position(&mut self, p: &WXY, precision: f64) {
        use ShapeType::*;
        match self {
            Line(line) => line.set_selection_from_position(p, precision),
            QuadBezier(quadbezier) => quadbezier.set_selection_from_position(p, precision),
            CubicBezier(cubicbezier) => cubicbezier.set_selection_from_position(p, precision),
            Rectangle(rectangle) => rectangle.set_selection_from_position(p, precision),
            Ellipse(ellipse) => ellipse.set_selection_from_position(p, precision),
        }
    }
    fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64) {
        use ShapeType::*;
        match self {
            Line(line) => line.move_selection(p, dp, snap_distance),
            QuadBezier(quadbezier) => quadbezier.move_selection(p, dp, snap_distance),
            CubicBezier(cubicbezier) => cubicbezier.move_selection(p, dp, snap_distance),
            Rectangle(rectangle) => rectangle.move_selection(p, dp, snap_distance),
            Ellipse(ellipse) => ellipse.move_selection(p, dp, snap_distance),
        };
    }
    fn get_construction(&self) -> Vec<ConstructionType> {
        use ShapeType::*;
        match self {
            Line(line) => line.get_construction(),
            QuadBezier(quadbezier) => quadbezier.get_construction(),
            CubicBezier(cubicbezier) => cubicbezier.get_construction(),
            Rectangle(rectangle) => rectangle.get_construction(),
            Ellipse(ellipse) => ellipse.get_construction(),
        }
    }
    fn get_handles_construction(&self, size_handle: WXY) -> Vec<ConstructionType> {
        use ShapeType::*;
        match self {
            Line(line) => line.get_handles_construction(size_handle),
            QuadBezier(quadbezier) => quadbezier.get_handles_construction(size_handle),
            CubicBezier(cubicbezier) => cubicbezier.get_handles_construction(size_handle),
            Rectangle(rectangle) => rectangle.get_handles_construction(size_handle),
            Ellipse(ellipse) => ellipse.get_handles_construction(size_handle),
        }
    }
    fn remove_any_selection(&mut self) {
        use ShapeType::*;
        match self {
            Line(line) => line.remove_any_selection(),
            QuadBezier(quadbezier) => quadbezier.remove_any_selection(),
            CubicBezier(cubicbezier) => cubicbezier.remove_any_selection(),
            Rectangle(rectangle) => rectangle.remove_any_selection(),
            Ellipse(ellipse) => ellipse.remove_any_selection(),
        }
    }
    fn get_bounding_box(&self) -> [WXY; 2] {
        use ShapeType::*;
        match self {
            Line(line) => line.get_bounding_box(),
            QuadBezier(quadbezier) => quadbezier.get_bounding_box(),
            CubicBezier(cubicbezier) => cubicbezier.get_bounding_box(),
            Rectangle(rectangle) => rectangle.get_bounding_box(),
            Ellipse(ellipse) => ellipse.get_bounding_box(),
        }
    }
    fn init_done(&mut self) {
        use ShapeType::*;
        match self {
            Line(line) => line.init_done(),
            QuadBezier(quadbezier) => quadbezier.init_done(),
            CubicBezier(cubicbezier) => cubicbezier.init_done(),
            Rectangle(rectangle) => rectangle.init_done(),
            Ellipse(ellipse) => ellipse.init_done(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Shape {
    shape: ShapeType,
    snap_precision: f64,
    handles_size: WXY,
}

impl Shape {
    pub fn new(shape: ShapeType, snap_precision: f64, handles_size: f64) -> Shape {
        Shape {
            shape,
            snap_precision,
            handles_size: WXY {
                wx: handles_size,
                wy: handles_size,
            },
        }
    }
    pub fn has_selection(&self) -> bool {
        use HandleSelection::*;
        match self.shape.get_selection() {
            None => false,
            Start | Ctrl | Ctrl1 | Ctrl2 | MidTop | MidRight | End | Center | All => true,
        }
    }
    pub fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64) {
        self.shape.move_selection(p, dp, snap_distance);
        self.shape.snap_geometry(p, dp, self.snap_precision);
    }
    pub fn get_handles_construction(&self) -> Vec<ConstructionType> {
        self.shape.get_handles_construction(self.handles_size)
    }

    // pub fn get_snap_construction(&self) -> Vec<ConstructionType> {
    //     let mut cst = Vec::new();
    //     for snap_type in self.snaps.iter() {
    //         match snap_type {
    //             SnapType::Geometry(idx1, idx2) => match &self.shape {
    //                 ShapeType::Line(handles)
    //                 | ShapeType::QuadBezier(handles)
    //                 | ShapeType::CubicBezier(handles)
    //                 | ShapeType::Rectangle(handles)
    //                 | ShapeType::Ellipse(handles) => {
    //                     let handle1 = handles[*idx1];
    //                     let handle2 = handles[*idx2];
    //                     let start = handle1 + self.offset;
    //                     let end = handle2 + self.offset;
    //                     extend_points(&mut [start, end]);
    //                     cst.push(ConstructionType::Move(start));
    //                     cst.push(ConstructionType::Line(end));
    //                 }
    //             },
    //             SnapType::Middle(_idx_mid, _idxs) => (),
    //         }
    //     }
    //     cst
    // }

    pub fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.shape.set_selection(handle_selection);
    }

    pub fn set_selection_from_position(&mut self, pos: &WXY, precision: f64) {
        self.shape.set_selection_from_position(pos, precision);
    }
    pub fn remove_selection(&mut self) {
        self.shape.remove_any_selection();
    }
    pub fn get_construction(&self) -> Vec<ConstructionType> {
        self.shape.get_construction()
    }
    pub fn get_bounding_box(&self) -> [WXY; 2] {
        self.shape.get_bounding_box()
    }
    pub fn init_done(&mut self) {
        self.shape.init_done()
    }
}
