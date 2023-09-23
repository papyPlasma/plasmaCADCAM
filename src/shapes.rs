// use web_sys::console;

use crate::math::*;
use std::{
    f64::consts::PI,
    sync::atomic::{AtomicUsize, Ordering},
};

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
    fn get_selection(&self) -> HandleSelection;
    fn set_selection(&mut self, handle_selection: HandleSelection);
    fn get_selection_from_position(&mut self, p: &WXY, precision: f64) -> HandleSelection;
    fn remove_any_selection(&mut self);
    fn move_selection(&mut self, p: &WXY, dp: &WXY, snap_distance: f64);
    fn get_construction(&self) -> Vec<ConstructionType>;
    fn get_handles_construction(&self, size_handle: WXY) -> Vec<ConstructionType>;
    fn get_helpers_construction(&self) -> Vec<ConstructionType>;
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
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn get_selection_from_position(&mut self, p: &WXY, precision: f64) -> HandleSelection {
        if is_point_on_point(p, &self.start, precision) {
            HandleSelection::Start
        } else {
            if is_point_on_point(p, &self.end, precision) {
                HandleSelection::End
            } else {
                if is_point_on_segment(p, &self.start, &self.end, precision) {
                    HandleSelection::All
                } else {
                    HandleSelection::None
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
                magnet(&self.end, &mut self.start, &p, snap_distance);
                snap_to_snap_grid(&mut self.start, snap_distance);
                if self.start == self.end {
                    self.start += snap_distance;
                }
            }
            End => {
                magnet(&self.start, &mut self.end, &p, snap_distance);
                snap_to_snap_grid(&mut self.end, snap_distance);
                if self.start == self.end {
                    self.end += snap_distance;
                }
            }
            All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_snap_grid_x(&mut self.tmp, snap_distance);
                    self.start.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_snap_grid_y(&mut self.tmp, snap_distance);
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
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use HandleSelection::*;
        match self.selection {
            Start | End | All => {
                push_vertical(&self.start, &self.end, false, &mut cst);
                push_horizontal(&self.start, &self.end, false, &mut cst);
                push_45_135(&self.start, &self.end, false, &mut cst);
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
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn get_selection_from_position(&mut self, p: &WXY, precision: f64) -> HandleSelection {
        if is_point_on_point(p, &self.start, precision) {
            HandleSelection::Start
        } else {
            if is_point_on_point(p, &self.ctrl, precision) {
                use HandleSelection::*;
                match self.selection {
                    None => HandleSelection::None,
                    _ => HandleSelection::Ctrl,
                }
            } else {
                if is_point_on_point(p, &self.end, precision) {
                    HandleSelection::End
                } else {
                    if is_point_on_quadbezier(p, &self.start, &self.ctrl, &self.end, precision) {
                        HandleSelection::All
                    } else {
                        HandleSelection::None
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
                magnet(&self.end, &mut self.start, &p, snap_distance);
                snap_to_snap_grid(&mut self.start, snap_distance);
                if self.start == self.end {
                    self.start += snap_distance;
                }
            }
            Ctrl => {
                let mut tmp_ctrla = self.ctrl;
                let mut tmp_ctrlb = self.ctrl;
                if magnet(&self.end, &mut tmp_ctrla, &p, snap_distance) {
                    self.ctrl = tmp_ctrla;
                } else {
                    magnet(&self.start, &mut tmp_ctrlb, &p, snap_distance);
                    self.ctrl = tmp_ctrlb;
                }
                snap_to_snap_grid(&mut self.ctrl, snap_distance);
            }
            End => {
                magnet(&self.start, &mut self.end, &p, snap_distance);
                snap_to_snap_grid(&mut self.end, snap_distance);
                if self.start == self.end {
                    self.end += snap_distance;
                }
                if self.init {
                    self.ctrl = (self.start + self.end) / 2.;
                }
            }
            All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_snap_grid_x(&mut self.tmp, snap_distance);
                    self.start.wx += self.tmp.wx;
                    self.ctrl.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_snap_grid_y(&mut self.tmp, snap_distance);
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
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use HandleSelection::*;
        match self.selection {
            Start | End => {
                push_vertical(&self.start, &self.end, true, &mut cst);
                push_horizontal(&self.start, &self.end, true, &mut cst);
                push_45_135(&self.start, &self.end, true, &mut cst);
            }
            Ctrl => {
                push_vertical(&self.ctrl, &self.start, true, &mut cst);
                push_horizontal(&self.ctrl, &self.start, true, &mut cst);
                push_45_135(&self.ctrl, &self.start, true, &mut cst);
                push_vertical(&self.ctrl, &self.end, true, &mut cst);
                push_horizontal(&self.ctrl, &self.end, true, &mut cst);
                push_45_135(&self.ctrl, &self.end, true, &mut cst);
            }
            All => {
                push_vertical(&self.start, &self.end, true, &mut cst);
                push_horizontal(&self.start, &self.end, true, &mut cst);
                push_45_135(&self.start, &self.end, true, &mut cst);
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
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn get_selection_from_position(&mut self, p: &WXY, precision: f64) -> HandleSelection {
        if is_point_on_point(p, &self.start, precision) {
            HandleSelection::Start
        } else {
            if is_point_on_point(p, &self.ctrl1, precision) {
                use HandleSelection::*;
                match self.selection {
                    None => HandleSelection::None,
                    _ => HandleSelection::Ctrl1,
                }
            } else {
                if is_point_on_point(p, &self.ctrl2, precision) {
                    use HandleSelection::*;
                    match self.selection {
                        None => HandleSelection::None,
                        _ => HandleSelection::Ctrl2,
                    }
                } else {
                    if is_point_on_point(p, &self.end, precision) {
                        HandleSelection::End
                    } else {
                        if is_point_on_cubicbezier(
                            p,
                            &self.start,
                            &self.ctrl1,
                            &self.ctrl2,
                            &self.end,
                            precision,
                        ) {
                            HandleSelection::All
                        } else {
                            HandleSelection::None
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
                magnet(&self.end, &mut self.start, &p, snap_distance);
                snap_to_snap_grid(&mut self.start, snap_distance);
                if self.start == self.end {
                    self.start += snap_distance;
                }
            }
            Ctrl1 => {
                let mut tmp_ctrla = self.ctrl1;
                let mut tmp_ctrlb = self.ctrl1;
                let mut tmp_ctrlc = self.ctrl1;
                if magnet(&self.end, &mut tmp_ctrla, &p, snap_distance) {
                    self.ctrl1 = tmp_ctrla;
                } else {
                    if magnet(&self.start, &mut tmp_ctrlb, &p, snap_distance) {
                        self.ctrl1 = tmp_ctrlb;
                    } else {
                        magnet(&self.ctrl2, &mut tmp_ctrlc, &p, snap_distance);
                        self.ctrl1 = tmp_ctrlc;
                    }
                }
                snap_to_snap_grid(&mut self.ctrl1, snap_distance);
            }
            Ctrl2 => {
                let mut tmp_ctrla = self.ctrl2;
                let mut tmp_ctrlb = self.ctrl2;
                let mut tmp_ctrlc = self.ctrl1;
                if magnet(&self.end, &mut tmp_ctrla, &p, snap_distance) {
                    self.ctrl2 = tmp_ctrla;
                } else {
                    if magnet(&self.start, &mut tmp_ctrlb, &p, snap_distance) {
                        self.ctrl2 = tmp_ctrlb;
                    } else {
                        magnet(&self.ctrl1, &mut tmp_ctrlc, &p, snap_distance);
                        self.ctrl2 = tmp_ctrlc;
                    }
                }
                snap_to_snap_grid(&mut self.ctrl2, snap_distance);
            }
            End => {
                magnet(&self.start, &mut self.end, &p, snap_distance);
                snap_to_snap_grid(&mut self.end, snap_distance);
                if self.start == self.end {
                    self.end += snap_distance;
                }
                if self.init {
                    self.ctrl1 = self.start + (self.end - self.start) / 3.;
                    self.ctrl2 = self.start + (self.end - self.start) / 3. * 2.;
                }
            }
            All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_snap_grid_x(&mut self.tmp, snap_distance);
                    self.start.wx += self.tmp.wx;
                    self.ctrl1.wx += self.tmp.wx;
                    self.ctrl2.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_snap_grid_y(&mut self.tmp, snap_distance);
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
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use HandleSelection::*;
        match self.selection {
            Start | End => {
                push_vertical(&self.start, &self.end, true, &mut cst);
                push_horizontal(&self.start, &self.end, true, &mut cst);
                push_45_135(&self.start, &self.end, true, &mut cst);
            }
            Ctrl1 => {
                push_vertical(&self.ctrl1, &self.start, true, &mut cst);
                push_horizontal(&self.ctrl1, &self.start, true, &mut cst);
                push_45_135(&self.ctrl1, &self.start, true, &mut cst);
                push_vertical(&self.ctrl1, &self.end, true, &mut cst);
                push_horizontal(&self.ctrl1, &self.end, true, &mut cst);
                push_45_135(&self.ctrl1, &self.end, true, &mut cst);
                push_vertical(&self.ctrl1, &self.ctrl2, true, &mut cst);
                push_horizontal(&self.ctrl1, &self.ctrl2, true, &mut cst);
                push_45_135(&self.ctrl1, &self.ctrl2, true, &mut cst);
            }
            Ctrl2 => {
                push_vertical(&self.ctrl2, &self.start, true, &mut cst);
                push_horizontal(&self.ctrl2, &self.start, true, &mut cst);
                push_45_135(&self.ctrl2, &self.start, true, &mut cst);
                push_vertical(&self.ctrl2, &self.end, true, &mut cst);
                push_horizontal(&self.ctrl2, &self.end, true, &mut cst);
                push_45_135(&self.ctrl2, &self.end, true, &mut cst);
                push_vertical(&self.ctrl2, &self.ctrl1, true, &mut cst);
                push_horizontal(&self.ctrl2, &self.ctrl1, true, &mut cst);
                push_45_135(&self.ctrl2, &self.ctrl1, true, &mut cst);
            }
            All => {
                push_vertical(&self.start, &self.end, true, &mut cst);
                push_horizontal(&self.start, &self.end, true, &mut cst);
                push_45_135(&self.start, &self.end, true, &mut cst);
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
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn get_selection_from_position(&mut self, p: &WXY, precision: f64) -> HandleSelection {
        if is_point_on_point(p, &self.start, precision) {
            HandleSelection::Start
        } else {
            if is_point_on_point(p, &self.mid_top, precision) {
                HandleSelection::MidTop
            } else {
                if is_point_on_point(p, &self.mid_right, precision) {
                    HandleSelection::MidRight
                } else {
                    if is_point_on_point(p, &self.end, precision) {
                        HandleSelection::End
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
                            HandleSelection::All
                        } else {
                            HandleSelection::None
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
                magnet(&self.end, &mut self.start, &p, snap_distance);
                snap_to_snap_grid(&mut self.start, snap_distance);
                if self.start.wx == self.end.wx {
                    self.start.wx += snap_distance;
                }
                if self.start.wy == self.end.wy {
                    self.start.wy += snap_distance;
                }
                self.mid_top.wx = (self.start.wx + self.end.wx) / 2.;
                self.mid_top.wy = self.end.wy;
                self.mid_right.wx = self.end.wx;
                self.mid_right.wy = (self.start.wy + self.end.wy) / 2.;
            }
            MidTop => {
                let mut mid_top_tmp1 = self.mid_top;
                let mut mid_top_tmp2 = self.mid_top;
                let v_start = WXY {
                    wx: self.start.wx,
                    wy: self.start.wy + 2. * (self.mid_top.wx - self.start.wx),
                };
                if magnet(&v_start, &mut mid_top_tmp1, &p, snap_distance) {
                    self.mid_top.wy = mid_top_tmp1.wy;
                } else {
                    let v_start = WXY {
                        wx: self.start.wx,
                        wy: self.start.wy - 2. * (self.mid_top.wx - self.start.wx),
                    };
                    magnet(&v_start, &mut mid_top_tmp2, &p, snap_distance);
                    self.mid_top.wy = mid_top_tmp2.wy;
                }

                snap_to_snap_grid_y(&mut self.mid_top, snap_distance);
                if self.mid_top.wy == self.start.wy {
                    self.mid_top.wy += snap_distance;
                }
                self.end.wy = self.mid_top.wy;
                self.mid_right.wy = (self.start.wy + self.end.wy) / 2.;
            }
            MidRight => {
                let mut mid_right_tmp1 = self.mid_right;
                let mut mid_right_tmp2 = self.mid_right;
                let v_start = WXY {
                    wx: self.start.wx + 2. * (self.mid_right.wy - self.start.wy),
                    wy: self.start.wy,
                };
                if magnet(&v_start, &mut mid_right_tmp1, &p, snap_distance) {
                    self.mid_right.wx = mid_right_tmp1.wx;
                } else {
                    let v_start = WXY {
                        wx: self.start.wx - 2. * (self.mid_right.wy - self.start.wy),
                        wy: self.start.wy,
                    };
                    magnet(&v_start, &mut mid_right_tmp2, &p, snap_distance);
                    self.mid_right.wx = mid_right_tmp2.wx;
                }

                snap_to_snap_grid_x(&mut self.mid_right, snap_distance);
                if self.mid_right.wx == self.start.wx {
                    self.mid_right.wx += snap_distance;
                }
                self.end.wx = self.mid_right.wx;
                self.mid_top.wx = (self.start.wx + self.end.wx) / 2.;
            }
            End => {
                magnet(&self.start, &mut self.end, &p, snap_distance);
                snap_to_snap_grid(&mut self.end, snap_distance);
                if self.start.wx == self.end.wx {
                    self.end.wx += snap_distance;
                }
                if self.start.wy == self.end.wy {
                    self.end.wy += snap_distance;
                }
                self.mid_top.wx = (self.start.wx + self.end.wx) / 2.;
                self.mid_top.wy = self.end.wy;
                self.mid_right.wx = self.end.wx;
                self.mid_right.wy = (self.start.wy + self.end.wy) / 2.;
            }
            All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_snap_grid_x(&mut self.tmp, snap_distance);
                    self.start.wx += self.tmp.wx;
                    self.mid_top.wx += self.tmp.wx;
                    self.mid_right.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_snap_grid_y(&mut self.tmp, snap_distance);
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
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use HandleSelection::*;
        match self.selection {
            Start | End => {
                push_45_135(&self.start, &self.end, true, &mut cst);
            }
            MidTop => {
                push_45_135(&self.start, &self.end, true, &mut cst);
            }
            MidRight => {
                push_45_135(&self.start, &self.end, true, &mut cst);
            }
            All => {
                push_45_135(&self.start, &self.end, true, &mut cst);
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
    fn get_selection(&self) -> HandleSelection {
        self.selection
    }
    fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.selection = handle_selection;
    }
    fn get_selection_from_position(&mut self, p: &WXY, precision: f64) -> HandleSelection {
        if is_point_on_point(p, &self.center, precision) {
            HandleSelection::Center
        } else {
            if is_point_on_point(p, &self.mid_top, precision) {
                HandleSelection::MidTop
            } else {
                if is_point_on_point(p, &self.mid_right, precision) {
                    HandleSelection::MidRight
                } else {
                    if is_point_on_point(p, &self.end, precision) {
                        use HandleSelection::*;
                        match self.selection {
                            None => HandleSelection::None,
                            _ => HandleSelection::End,
                        }
                    } else {
                        let radius = self.end - self.center;
                        if is_point_on_ellipse(p, &self.center, &radius, precision) {
                            HandleSelection::All
                        } else {
                            HandleSelection::None
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
                let mut mid_top_tmp1 = self.mid_top;
                let mut mid_top_tmp2 = self.mid_top;
                let v_start = WXY {
                    wx: self.mid_right.wx,
                    wy: self.mid_right.wy + (self.mid_top.wx - self.mid_right.wx),
                };
                if magnet(&v_start, &mut mid_top_tmp1, &p, snap_distance) {
                    self.mid_top.wy = mid_top_tmp1.wy;
                } else {
                    let v_start = WXY {
                        wx: self.mid_right.wx,
                        wy: self.mid_right.wy - (self.mid_top.wx - self.mid_right.wx),
                    };
                    magnet(&v_start, &mut mid_top_tmp2, &p, snap_distance);
                    self.mid_top.wy = mid_top_tmp2.wy;
                }
                snap_to_snap_grid_y(&mut self.mid_top, snap_distance);
                if self.mid_top.wy == self.center.wy {
                    self.mid_top.wy += snap_distance;
                }
                self.end.wy = self.mid_top.wy;
            }
            MidRight => {
                let mut mid_right_tmp1 = self.mid_right;
                let mut mid_right_tmp2 = self.mid_right;
                let v_start = WXY {
                    wx: self.mid_top.wx + (self.mid_right.wy - self.mid_top.wy),
                    wy: self.mid_top.wy,
                };
                if magnet(&v_start, &mut mid_right_tmp1, &p, snap_distance) {
                    self.mid_right.wx = mid_right_tmp1.wx;
                } else {
                    let v_start = WXY {
                        wx: self.mid_top.wx - (self.mid_right.wy - self.mid_top.wy),
                        wy: self.mid_top.wy,
                    };
                    magnet(&v_start, &mut mid_right_tmp2, &p, snap_distance);
                    self.mid_right.wx = mid_right_tmp2.wx;
                }
                snap_to_snap_grid_x(&mut self.mid_right, snap_distance);
                if self.mid_right.wx == self.center.wx {
                    self.mid_right.wx += snap_distance;
                }
                self.end.wx = self.mid_right.wx;
            }
            End => {
                magnet(&self.center, &mut self.end, &p, snap_distance);
                snap_to_snap_grid(&mut self.end, snap_distance);
                if self.center.wx == self.end.wx {
                    self.end.wx += snap_distance;
                }
                if self.center.wy == self.end.wy {
                    self.end.wy += snap_distance;
                }
                self.mid_top.wx = self.center.wx;
                self.mid_top.wy = self.end.wy;
                self.mid_right.wx = self.end.wx;
                self.mid_right.wy = self.center.wy;
            }
            Center | All => {
                self.tmp += *dp;
                if self.tmp.wx > snap_distance || self.tmp.wx < -snap_distance {
                    snap_to_snap_grid_x(&mut self.tmp, snap_distance);
                    self.center.wx += self.tmp.wx;
                    self.mid_top.wx += self.tmp.wx;
                    self.mid_right.wx += self.tmp.wx;
                    self.end.wx += self.tmp.wx;
                    self.tmp.wx = 0.;
                }
                if self.tmp.wy > snap_distance || self.tmp.wy < -snap_distance {
                    snap_to_snap_grid_y(&mut self.tmp, snap_distance);
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
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use HandleSelection::*;
        match self.selection {
            Center | End => {
                push_45_135(&self.center, &self.end, true, &mut cst);
            }
            MidTop => {
                push_45_135(&self.center, &self.end, true, &mut cst);
            }
            MidRight => {
                push_45_135(&self.center, &self.end, true, &mut cst);
            }
            All => {
                push_45_135(&self.center, &self.end, true, &mut cst);
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
    fn get_selection_from_position(&mut self, p: &WXY, precision: f64) -> HandleSelection {
        use ShapeType::*;
        match self {
            Line(line) => line.get_selection_from_position(p, precision),
            QuadBezier(quadbezier) => quadbezier.get_selection_from_position(p, precision),
            CubicBezier(cubicbezier) => cubicbezier.get_selection_from_position(p, precision),
            Rectangle(rectangle) => rectangle.get_selection_from_position(p, precision),
            Ellipse(ellipse) => ellipse.get_selection_from_position(p, precision),
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
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        use ShapeType::*;
        match self {
            Line(line) => line.get_helpers_construction(),
            QuadBezier(quadbezier) => quadbezier.get_helpers_construction(),
            CubicBezier(cubicbezier) => cubicbezier.get_helpers_construction(),
            Rectangle(rectangle) => rectangle.get_helpers_construction(),
            Ellipse(ellipse) => ellipse.get_helpers_construction(),
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
    id: usize,
    shape: ShapeType,
    handles_size: WXY,
}

static COUNTER: AtomicUsize = AtomicUsize::new(0);

impl Shape {
    pub fn new(shape: ShapeType, handles_size: f64) -> Shape {
        let id = COUNTER.fetch_add(1, Ordering::Relaxed);
        Shape {
            id,
            shape,
            handles_size: WXY {
                wx: handles_size,
                wy: handles_size,
            },
        }
    }
    pub fn get_id(&self) -> usize {
        self.id
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
    }
    pub fn set_selection(&mut self, handle_selection: HandleSelection) {
        self.shape.set_selection(handle_selection);
    }
    pub fn get_selection_from_position(&mut self, pos: &WXY, precision: f64) -> HandleSelection {
        self.shape.get_selection_from_position(pos, precision)
    }
    pub fn remove_selection(&mut self) {
        self.shape.remove_any_selection();
    }
    pub fn get_construction(&self) -> Vec<ConstructionType> {
        self.shape.get_construction()
    }
    pub fn get_handles_construction(&self) -> Vec<ConstructionType> {
        self.shape.get_handles_construction(self.handles_size)
    }
    pub fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        self.shape.get_helpers_construction()
    }
    pub fn get_bounding_box(&self) -> [WXY; 2] {
        self.shape.get_bounding_box()
    }
    pub fn init_done(&mut self) {
        self.shape.init_done()
    }
}
