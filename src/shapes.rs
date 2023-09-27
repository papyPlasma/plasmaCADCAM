// use web_sys::console;

use web_sys::console;

use crate::math::*;
use std::{
    collections::HashMap,
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
#[derive(Debug, Copy, Clone)]
pub enum LayerType {
    Worksheet,
    Dimension,
    GeometryHelpers,
    Origin,
    Grid,
    SelectionTool,
    Selected,
    Highlight,
    Handle(bool),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum ConstructionType {
    Layer(LayerType),
    Move(WPoint),
    Line(WPoint),
    QuadBezier(WPoint, WPoint),
    CubicBezier(WPoint, WPoint, WPoint),
    Ellipse(WPoint, WPoint, f64, f64, f64, bool),
    Rectangle(WPoint, WPoint, bool),
    Text(WPoint, String),
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Handle {
    Start,
    Ctrl,
    Ctrl1,
    Ctrl2,
    BottomLeft,
    TopLeft,
    TopRight,
    BottomRight,
    End,
    Radius,
    Center,
    StartAngle,
    EndAngle,
    All,
}

#[derive(Copy, Clone)]
pub enum ShapeType {
    Line((Handle, WPoint), (Handle, WPoint)),
    QuadBezier((Handle, WPoint), (Handle, WPoint), (Handle, WPoint)),
    CubicBezier(
        (Handle, WPoint),
        (Handle, WPoint),
        (Handle, WPoint),
        (Handle, WPoint),
    ),
    Rectangle(
        (Handle, WPoint),
        (Handle, WPoint),
        (Handle, WPoint),
        (Handle, WPoint),
    ),
    Ellipse(
        (Handle, WPoint),
        (Handle, WPoint),
        (Handle, WPoint), // Start angle
        (Handle, WPoint), // End angle
        (f64, f64, f64),  // Rotation, start angle, end angle
    ),
}

#[derive(Copy, Clone)]
pub struct Shape {
    id: usize,
    shape: ShapeType,
    snap_distance: f64,
    handles_size: WPoint,
    highlight_size: WPoint,
    selection: Option<Handle>,
    highlight: Option<Handle>,
    tmp: WPoint,
    init: bool,
}

static COUNTER_SHAPE: AtomicUsize = AtomicUsize::new(0);

impl Shape {
    pub fn new_line(
        start_point: &WPoint,
        end_point: &WPoint,
        snap_distance: f64,
        handles_size: f64,
        highlight_size: f64,
    ) -> Self {
        let id = COUNTER_SHAPE.fetch_add(1, Ordering::Relaxed);
        let end_point = if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
            *start_point + snap_distance
        } else {
            *end_point
        };
        Self {
            id,
            shape: ShapeType::Line((Handle::Start, *start_point), (Handle::End, end_point)),
            snap_distance,
            handles_size: WPoint::default() + handles_size,
            highlight_size: WPoint::default() + highlight_size,
            selection: Some(Handle::End),
            highlight: None,
            tmp: WPoint::default(),
            init: true,
        }
    }
    pub fn new_quadbezier(
        start_point: &WPoint,
        ctrl_point: &WPoint,
        end_point: &WPoint,
        snap_distance: f64,
        handles_size: f64,
        highlight_size: f64,
    ) -> Self {
        let id = COUNTER_SHAPE.fetch_add(1, Ordering::Relaxed);
        let (end_point, ctrl_point) =
            if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
                (
                    *start_point + 2. * snap_distance,
                    *start_point + snap_distance,
                )
            } else {
                (*end_point, *ctrl_point)
            };
        Self {
            id,
            shape: ShapeType::QuadBezier(
                (Handle::Start, *start_point),
                (Handle::Ctrl, ctrl_point),
                (Handle::End, end_point),
            ),
            snap_distance,
            handles_size: WPoint::default() + handles_size,
            highlight_size: WPoint::default() + highlight_size,
            selection: Some(Handle::End),
            highlight: None,
            tmp: WPoint::default(),
            init: true,
        }
    }
    pub fn new_cubicbezier(
        start_point: &WPoint,
        ctrl1_point: &WPoint,
        ctrl2_point: &WPoint,
        end_point: &WPoint,
        snap_distance: f64,
        handles_size: f64,
        highlight_size: f64,
    ) -> Self {
        let id = COUNTER_SHAPE.fetch_add(1, Ordering::Relaxed);
        let (end_point, ctrl1_point, ctrl2_point) =
            if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
                (
                    *start_point + 3. * snap_distance,
                    *start_point + snap_distance,
                    *start_point + 2. * snap_distance,
                )
            } else {
                (*end_point, *ctrl1_point, *ctrl2_point)
            };
        Self {
            id,
            shape: ShapeType::CubicBezier(
                (Handle::Start, *start_point),
                (Handle::Ctrl1, ctrl1_point),
                (Handle::Ctrl2, ctrl2_point),
                (Handle::End, end_point),
            ),
            snap_distance,
            handles_size: WPoint::default() + handles_size,
            highlight_size: WPoint::default() + highlight_size,
            selection: Some(Handle::End),
            highlight: None,
            tmp: WPoint::default(),
            init: true,
        }
    }
    pub fn new_rectangle(
        bl: &WPoint,
        w: f64,
        h: f64,
        snap_distance: f64,
        handles_size: f64,
        highlight_size: f64,
    ) -> Self {
        let id = COUNTER_SHAPE.fetch_add(1, Ordering::Relaxed);
        let (w, h) = if w == 0. || h == 0. {
            (5. * snap_distance, 5. * snap_distance)
        } else {
            (w, h)
        };
        Self {
            id,
            shape: ShapeType::Rectangle(
                (Handle::BottomLeft, *bl),
                (Handle::TopLeft, bl.addxy(0., -h)),
                (Handle::TopRight, bl.addxy(w, -h)),
                (Handle::BottomRight, bl.addxy(w, 0.)),
            ),
            snap_distance,
            handles_size: WPoint::default() + handles_size,
            highlight_size: WPoint::default() + highlight_size,
            selection: Some(Handle::TopRight),
            highlight: None,
            tmp: WPoint::default(),
            init: true,
        }
    }
    pub fn new_ellipse(
        center: &WPoint,
        radius: &WPoint,
        rotation: f64,
        start_angle: f64,
        end_angle: f64,
        snap_distance: f64,
        handles_size: f64,
        highlight_size: f64,
    ) -> Self {
        let id = COUNTER_SHAPE.fetch_add(1, Ordering::Relaxed);
        let radius = if radius.wx == 0. || radius.wy == 0. {
            WPoint {
                wx: 5. * snap_distance,
                wy: -5. * snap_distance,
            }
        } else {
            *radius
        };
        let h_start_angle = get_point_from_angle(center, &radius, rotation, -start_angle);
        let h_end_angle = get_point_from_angle(center, &radius, rotation, -end_angle);
        console::log_1(
            &format!(
                "radius.wx:{:?} radius.wy:{:?} h_start_angle:{:?} h_end_angle:{:?}",
                radius.wx, radius.wy, h_start_angle, h_end_angle
            )
            .into(),
        );
        Self {
            id,
            shape: ShapeType::Ellipse(
                (Handle::Center, *center),
                (Handle::Radius, *center + radius),
                (Handle::StartAngle, h_start_angle.addxy(center.wx, 0.)),
                (Handle::EndAngle, h_end_angle.addxy(center.wx, 0.)),
                (rotation, start_angle, end_angle),
            ),
            snap_distance,
            handles_size: WPoint::default() + handles_size,
            highlight_size: WPoint::default() + highlight_size,
            selection: Some(Handle::Radius),
            highlight: None,
            tmp: WPoint::default(),
            init: true,
        }
    }
    pub fn get_id(&self) -> usize {
        self.id
    }
    pub fn has_selection(&self) -> bool {
        if let Some(_) = self.selection {
            true
        } else {
            false
        }
    }
    pub fn get_selection(&self) -> Option<Handle> {
        self.selection
    }
    pub fn set_selection(&mut self, handle_selection: Option<Handle>) {
        self.selection = handle_selection;
    }
    pub fn get_selection_from_position(&mut self, p: &WPoint, precision: f64) -> Option<Handle> {
        use ShapeType::*;
        // Get a list of all handle-point pairs in the current shape
        let handle_pairs = match &self.shape {
            Line(pair1, pair2) => vec![pair1, pair2],
            QuadBezier(pair1, pair2, pair3) => vec![pair1, pair2, pair3],
            CubicBezier(pair1, pair2, pair3, pair4) => vec![pair1, pair2, pair3, pair4],
            Rectangle(pair1, pair2, pair3, pair4) => vec![pair1, pair2, pair3, pair4],
            Ellipse(pair1, pair2, pair3, pair4, _) => vec![pair1, pair2, pair3, pair4],
        };
        for (handle, point) in handle_pairs.iter() {
            if is_point_on_point(p, &point, precision) {
                return Some(*handle);
            }
        }
        match &self.shape {
            Line(_, _) => {
                if is_point_on_line(&p, &self.shape, precision) {
                    return Some(Handle::All);
                }
            }
            QuadBezier(_, _, _) => {
                if is_point_on_quadbezier(&p, &self.shape, precision) {
                    return Some(Handle::All);
                }
            }
            CubicBezier(_, _, _, _) => {
                if is_point_on_cubicbezier(&p, &self.shape, precision) {
                    return Some(Handle::All);
                }
            }
            Rectangle(_, _, _, _) => {
                if is_point_on_line(&p, &self.shape, precision)
                    || is_point_on_line(&p, &self.shape, precision)
                    || is_point_on_line(&p, &self.shape, precision)
                    || is_point_on_line(&p, &self.shape, precision)
                {
                    return Some(Handle::All);
                }
            }
            Ellipse(_, _, _, _, _) => {
                if is_point_on_ellipse(&p, &self.shape, precision) {
                    return Some(Handle::All);
                }
            }
        };

        None
    }
    pub fn move_selection(&mut self, p: &WPoint, dp: &WPoint) {
        use Handle::*;
        if let Some(selection) = self.selection {
            let mut new_point = *p;

            match &mut self.shape {
                ShapeType::Line(pair1, pair2) => match selection {
                    Start => {
                        magnet_geometry(&pair2.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        pair1.1 = new_point;
                        // Avoid degenerescence
                        if pair1.1 == pair2.1 {
                            pair1.1 += self.snap_distance;
                        }
                    }
                    End => {
                        magnet_geometry(&pair1.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        pair2.1 = new_point;
                        // Avoid degenerescence
                        if pair1.1 == pair2.1 {
                            pair2.1 += self.snap_distance;
                        }
                    }
                    All => {
                        self.tmp += *dp;
                        if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
                            snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
                            pair1.1.wx += self.tmp.wx;
                            pair2.1.wx += self.tmp.wx;
                            self.tmp.wx = 0.;
                        }
                        if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
                            snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
                            pair1.1.wy += self.tmp.wy;
                            pair2.1.wy += self.tmp.wy;
                            self.tmp.wy = 0.;
                        }
                    }
                    _ => {}
                },
                ShapeType::QuadBezier(start, ctrl, end) => match selection {
                    Start => {
                        magnet_geometry(&end.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        start.1 = new_point;
                    }
                    Ctrl => {
                        if !magnet_geometry(&end.1, &mut new_point, self.snap_distance) {
                            magnet_geometry(&start.1, &mut new_point, self.snap_distance);
                        }
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        ctrl.1 = new_point;
                    }
                    End => {
                        magnet_geometry(&start.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        end.1 = new_point;
                        if self.init {
                            ctrl.1 = (start.1 + end.1) / 2.;
                        }
                    }
                    All => {
                        self.tmp += *dp;
                        if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
                            snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
                            start.1.wx += self.tmp.wx;
                            ctrl.1.wx += self.tmp.wx;
                            end.1.wx += self.tmp.wx;
                            self.tmp.wx = 0.;
                        }
                        if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
                            snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
                            start.1.wy += self.tmp.wy;
                            ctrl.1.wy += self.tmp.wy;
                            end.1.wy += self.tmp.wy;
                            self.tmp.wy = 0.;
                        }
                    }
                    _ => {}
                },
                ShapeType::CubicBezier(start, ctrl1, ctrl2, end) => match selection {
                    Start => {
                        magnet_geometry(&end.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        start.1 = new_point;
                    }
                    Ctrl1 => {
                        if !magnet_geometry(&end.1, &mut new_point, self.snap_distance) {
                            if !magnet_geometry(&start.1, &mut new_point, self.snap_distance) {
                                magnet_geometry(&ctrl2.1, &mut new_point, self.snap_distance);
                            }
                        }
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        ctrl1.1 = new_point;
                    }
                    Ctrl2 => {
                        if !magnet_geometry(&end.1, &mut new_point, self.snap_distance) {
                            if !magnet_geometry(&start.1, &mut new_point, self.snap_distance) {
                                magnet_geometry(&ctrl1.1, &mut new_point, self.snap_distance);
                            }
                        }
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        ctrl2.1 = new_point;
                    }
                    End => {
                        magnet_geometry(&start.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        end.1 = new_point;
                        if self.init {
                            ctrl1.1 = start.1 + (end.1 - start.1) / 3.;
                            ctrl2.1 = start.1 + (end.1 - start.1) / 3. * 2.;
                        }
                    }
                    All => {
                        self.tmp += *dp;
                        if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
                            snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
                            start.1.wx += self.tmp.wx;
                            ctrl1.1.wx += self.tmp.wx;
                            ctrl2.1.wx += self.tmp.wx;
                            end.1.wx += self.tmp.wx;
                            self.tmp.wx = 0.;
                        }
                        if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
                            snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
                            start.1.wy += self.tmp.wy;
                            ctrl1.1.wy += self.tmp.wy;
                            ctrl2.1.wy += self.tmp.wy;
                            end.1.wy += self.tmp.wy;
                            self.tmp.wy = 0.;
                        }
                    }
                    _ => {}
                },
                ShapeType::Rectangle(bl, tl, tr, br) => match selection {
                    BottomLeft => {
                        magnet_geometry(&tr.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        bl.1 = new_point;
                        if bl.1.wx >= tr.1.wx {
                            bl.1.wx = tr.1.wx - self.snap_distance;
                        }
                        if bl.1.wy <= tr.1.wy {
                            bl.1.wy = tr.1.wy + self.snap_distance;
                        }
                        tl.1.wx = bl.1.wx;
                        br.1.wy = bl.1.wy;
                    }
                    TopLeft => {
                        magnet_geometry(&br.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        tl.1 = new_point;
                        if tl.1.wx >= br.1.wx {
                            tl.1.wx = br.1.wx - self.snap_distance;
                        }
                        if tl.1.wy >= br.1.wy {
                            tl.1.wy = br.1.wy - self.snap_distance;
                        }
                        tr.1.wy = tl.1.wy;
                        bl.1.wx = tl.1.wx;
                    }
                    TopRight => {
                        magnet_geometry(&bl.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        tr.1 = new_point;
                        if tr.1.wx <= bl.1.wx {
                            tr.1.wx = bl.1.wx + self.snap_distance;
                        }
                        if tr.1.wy >= bl.1.wy {
                            tr.1.wy = bl.1.wy - self.snap_distance;
                        }
                        tl.1.wy = tr.1.wy;
                        br.1.wx = tr.1.wx;
                    }
                    BottomRight => {
                        magnet_geometry(&tl.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        br.1 = new_point;
                        if br.1.wx <= tl.1.wx {
                            br.1.wx = tl.1.wx + self.snap_distance;
                        }
                        if br.1.wy <= tl.1.wy {
                            br.1.wy = tl.1.wy + self.snap_distance;
                        }
                        tr.1.wx = br.1.wx;
                        bl.1.wy = br.1.wy;
                    }
                    All => {
                        self.tmp += *dp;
                        if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
                            snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
                            bl.1.wx += self.tmp.wx;
                            tl.1.wx += self.tmp.wx;
                            tr.1.wx += self.tmp.wx;
                            br.1.wx += self.tmp.wx;
                            self.tmp.wx = 0.;
                        }
                        if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
                            snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
                            bl.1.wy += self.tmp.wy;
                            tl.1.wy += self.tmp.wy;
                            tr.1.wy += self.tmp.wy;
                            br.1.wy += self.tmp.wy;
                            self.tmp.wy = 0.;
                        }
                    }
                    _ => {}
                },
                ShapeType::Ellipse(
                    center,
                    radius,
                    h_start_angle,
                    h_end_angle,
                    (rotation, start_angle, end_angle),
                ) => match selection {
                    Center => {
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        let delta_pos = new_point - center.1;
                        center.1 = new_point;
                        radius.1 += delta_pos;
                        h_start_angle.1 =
                            get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
                        h_end_angle.1 =
                            get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
                    }
                    Radius => {
                        magnet_geometry(&center.1, &mut new_point, self.snap_distance);
                        snap_to_snap_grid(&mut new_point, self.snap_distance);
                        radius.1 = new_point;
                        if radius.1.wx <= center.1.wx {
                            radius.1.wx = center.1.wx + self.snap_distance;
                        }
                        if radius.1.wy >= center.1.wy {
                            radius.1.wy = center.1.wy - self.snap_distance;
                        }
                        h_start_angle.1 =
                            get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
                        h_end_angle.1 =
                            get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
                    }
                    StartAngle => {
                        let mut angle = get_angle_from_point(&new_point, &center.1, 0.);
                        h_start_angle.1 =
                            get_point_from_angle(&center.1, &radius.1, *rotation, -angle);
                        magnet_geometry(&center.1, &mut h_start_angle.1, self.snap_distance);
                        angle = get_angle_from_point(&h_start_angle.1, &center.1, 0.);
                        *start_angle = angle;
                        if *start_angle > *end_angle {
                            *start_angle -= 2. * PI;
                        }
                        h_start_angle.1 =
                            get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
                    }
                    EndAngle => {
                        let mut angle = get_angle_from_point(&new_point, &center.1, 0.);
                        h_end_angle.1 =
                            get_point_from_angle(&center.1, &radius.1, *rotation, -angle);
                        magnet_geometry(&center.1, &mut h_end_angle.1, self.snap_distance);
                        angle = get_angle_from_point(&h_end_angle.1, &center.1, 0.);
                        *end_angle = angle;
                        if *end_angle < *start_angle {
                            *end_angle += 2. * PI;
                        }
                        h_end_angle.1 =
                            get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
                    }
                    All => {
                        self.tmp += *dp;
                        if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
                            snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
                            center.1.wx += self.tmp.wx;
                            radius.1.wx += self.tmp.wx;
                            h_start_angle.1 = get_point_from_angle(
                                &center.1,
                                &radius.1,
                                *rotation,
                                -*start_angle,
                            );
                            h_end_angle.1 =
                                get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
                            self.tmp.wx = 0.;
                        }
                        if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
                            snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
                            center.1.wy += self.tmp.wy;
                            radius.1.wy += self.tmp.wy;
                            h_start_angle.1 = get_point_from_angle(
                                &center.1,
                                &radius.1,
                                *rotation,
                                -*start_angle,
                            );
                            h_end_angle.1 =
                                get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
                            self.tmp.wy = 0.;
                        }
                    }
                    _ => {}
                },
            }
        }
    }
    pub fn remove_highlight(&mut self) {
        self.highlight = None;
    }
    pub fn get_highlight(&self) -> Option<Handle> {
        self.highlight
    }
    pub fn set_highlight_from_position(&mut self, p: &WPoint) -> bool {
        // Reset any existing highlight
        self.highlight = None;

        let mut highlight_found = false;

        // Get a list of all highlightable handle-point pairs in the current shape
        use ShapeType::*;
        let handle_pairs = match &self.shape {
            Line(pair1, pair2) => vec![pair1, pair2],
            QuadBezier(pair1, _, pair3) => vec![pair1, pair3],
            CubicBezier(pair1, _, _, pair4) => vec![pair1, pair4],
            Rectangle(pair1, _, _, pair4) => vec![pair1, pair4],
            Ellipse(pair1, _, _, pair4, _) => vec![pair1, pair4],
        };

        for (handle, point) in handle_pairs.iter() {
            if is_point_on_point(p, point, self.snap_distance) {
                self.highlight = Some(*handle);
                highlight_found = true;
                break; // Exit loop once a match is found
            }
        }

        highlight_found
    }
    pub fn get_highlightable_handles_positions(&self) -> Vec<(Handle, WPoint)> {
        let mut v = Vec::new();

        // Get a list of all highlightable handle-point pairs in the current shape
        use ShapeType::*;
        let handle_pairs = match &self.shape {
            Line(pair1, pair2) => vec![pair1, pair2],
            QuadBezier(pair1, _, pair3) => vec![pair1, pair3],
            CubicBezier(pair1, _, _, pair4) => vec![pair1, pair4],
            Rectangle(pair1, _, _, pair4) => vec![pair1, pair4],
            Ellipse(pair1, _, _, pair4, _) => vec![pair1, pair4],
        };
        v.push(handle_pairs[0].clone());
        v.push(handle_pairs[1].clone());
        v
    }
    pub fn get_shape_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        if let None = self.selection {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        use ShapeType::*;
        match &self.shape {
            Line(start, end) => {
                cst.push(ConstructionType::Move(start.1));
                cst.push(ConstructionType::Line(end.1));
            }
            QuadBezier(pair1, pair2, pair3) => {
                cst.push(ConstructionType::Move(pair1.1));
                cst.push(ConstructionType::QuadBezier(pair2.1, pair3.1));
            }
            CubicBezier(pair1, pair2, pair3, pair4) => {
                cst.push(ConstructionType::Move(pair1.1));
                cst.push(ConstructionType::CubicBezier(pair2.1, pair3.1, pair4.1));
            }
            Rectangle(bl, tl, tr, br) => {
                cst.push(ConstructionType::Move(bl.1));
                cst.push(ConstructionType::Line(tl.1));
                cst.push(ConstructionType::Line(tr.1));
                cst.push(ConstructionType::Line(br.1));
                cst.push(ConstructionType::Line(bl.1));
            }
            Ellipse(
                center,
                radius,
                h_start_angle,
                _h_end_angle,
                (rotation, start_angle, end_angle),
            ) => {
                cst.push(ConstructionType::Move(h_start_angle.1));
                cst.push(ConstructionType::Ellipse(
                    center.1,
                    (radius.1 - center.1).abs(),
                    *rotation,
                    *start_angle,
                    *end_angle,
                    false,
                ));
            }
        };
        cst
    }
    pub fn get_handles_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        use Handle::*;
        if let Some(selection) = self.selection {
            use ShapeType::*;
            let handles_pairs = match &self.shape {
                Line(pair1, pair2) => match selection {
                    Start => vec![(pair1.1, true), (pair2.1, false)],
                    End => vec![(pair1.1, false), (pair2.1, true)],
                    _ => vec![(pair1.1, false), (pair2.1, false)],
                },
                QuadBezier(pair1, pair2, pair3) => match selection {
                    Start => vec![(pair1.1, true), (pair2.1, false), (pair3.1, false)],
                    Ctrl => vec![(pair1.1, false), (pair2.1, true), (pair3.1, false)],
                    End => vec![(pair1.1, false), (pair2.1, false), (pair3.1, true)],
                    _ => vec![(pair1.1, false), (pair2.1, false), (pair3.1, false)],
                },

                CubicBezier(pair1, pair2, pair3, pair4) => match selection {
                    Start => vec![
                        (pair1.1, true),
                        (pair2.1, false),
                        (pair3.1, false),
                        (pair4.1, false),
                    ],
                    Ctrl1 => vec![
                        (pair1.1, false),
                        (pair2.1, true),
                        (pair3.1, false),
                        (pair4.1, false),
                    ],
                    Ctrl2 => vec![
                        (pair1.1, false),
                        (pair2.1, false),
                        (pair3.1, true),
                        (pair4.1, false),
                    ],
                    End => vec![
                        (pair1.1, false),
                        (pair2.1, false),
                        (pair3.1, false),
                        (pair4.1, true),
                    ],
                    _ => vec![
                        (pair1.1, false),
                        (pair2.1, false),
                        (pair3.1, false),
                        (pair4.1, false),
                    ],
                },
                Rectangle(bl, tl, tr, br) => match selection {
                    BottomLeft => vec![(bl.1, true), (tl.1, false), (tr.1, false), (br.1, false)],
                    TopLeft => vec![(bl.1, false), (tl.1, true), (tr.1, false), (br.1, false)],
                    TopRight => vec![(bl.1, false), (tl.1, false), (tr.1, true), (br.1, false)],
                    BottomRight => vec![(bl.1, false), (tl.1, false), (tr.1, false), (br.1, true)],
                    _ => vec![(bl.1, false), (tl.1, false), (tr.1, false), (br.1, false)],
                },
                Ellipse(center, radius, h_start_angle, h_end_angle, _) => match selection {
                    Center => vec![
                        (center.1, true),
                        (radius.1, false),
                        (h_start_angle.1, false),
                        (h_end_angle.1, false),
                    ],
                    Radius => vec![
                        (center.1, false),
                        (radius.1, true),
                        (h_start_angle.1, false),
                        (h_end_angle.1, false),
                    ],
                    StartAngle => vec![
                        (center.1, false),
                        (radius.1, false),
                        (h_start_angle.1, true),
                        (h_end_angle.1, false),
                    ],
                    EndAngle => vec![
                        (center.1, false),
                        (radius.1, false),
                        (h_start_angle.1, false),
                        (h_end_angle.1, true),
                    ],
                    _ => vec![
                        (center.1, false),
                        (radius.1, false),
                        (h_start_angle.1, false),
                        (h_end_angle.1, false),
                    ],
                },
            };
            for (point, fill) in handles_pairs.iter() {
                push_handle(&point, &self.handles_size, *fill, &mut cst);
            }
        }
        cst
    }
    pub fn get_highlight_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        // Get a list of all highlightable handle-point pairs in the current shape
        use ShapeType::*;
        let handle_pairs = match &self.shape {
            Line(pair1, pair2) => vec![pair1.1, pair2.1],
            QuadBezier(pair1, _, pair3) => vec![pair1.1, pair3.1],
            CubicBezier(pair1, _, _, pair4) => vec![pair1.1, pair4.1],
            Rectangle(pair1, _, _, pair4) => vec![pair1.1, pair4.1],
            Ellipse(pair1, _, _, pair4, _) => vec![pair1.1, pair4.1],
        };

        if let Some(highlight) = self.highlight {
            match highlight {
                Handle::Start => {
                    cst.push(ConstructionType::Layer(LayerType::Highlight));
                    push_handle(&handle_pairs[0], &self.highlight_size, false, &mut cst);
                }
                Handle::End => {
                    cst.push(ConstructionType::Layer(LayerType::Highlight));
                    push_handle(&handle_pairs[1], &self.highlight_size, false, &mut cst);
                }
                _ => (),
            };
        }
        cst
    }
    pub fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        // Get a list of all highlightable handle-point pairs in the current shape
        use Handle::*;
        use ShapeType::*;
        match &self.shape {
            Line(pair1, pair2) => {
                if let Some(selection) = self.selection {
                    match selection {
                        Start | End => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&pair1.1, &pair2.1, true, &mut cst);
                            push_horizontal(&pair1.1, &pair2.1, true, &mut cst);
                            push_45_135(&pair1.1, &pair2.1, true, &mut cst);
                        }
                        _ => (),
                    }
                }
            }
            QuadBezier(pair1, pair2, pair3) => {
                if let Some(selection) = self.selection {
                    match selection {
                        Start | End => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&pair1.1, &pair3.1, true, &mut cst);
                            push_horizontal(&pair1.1, &pair3.1, true, &mut cst);
                            push_45_135(&pair1.1, &pair3.1, true, &mut cst);
                        }
                        Ctrl => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&pair2.1, &pair1.1, true, &mut cst);
                            push_horizontal(&pair2.1, &pair1.1, true, &mut cst);
                            push_45_135(&pair2.1, &pair1.1, true, &mut cst);
                            push_vertical(&pair2.1, &pair3.1, true, &mut cst);
                            push_horizontal(&pair2.1, &pair3.1, true, &mut cst);
                            push_45_135(&pair2.1, &pair3.1, true, &mut cst);
                        }
                        _ => (),
                    }
                }
            }
            CubicBezier(pair1, pair2, pair3, pair4) => {
                if let Some(selection) = self.selection {
                    match selection {
                        Start | End => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&pair1.1, &pair4.1, true, &mut cst);
                            push_horizontal(&pair1.1, &pair4.1, true, &mut cst);
                            push_45_135(&pair1.1, &pair4.1, true, &mut cst);
                        }
                        Ctrl1 => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&pair2.1, &pair1.1, true, &mut cst);
                            push_horizontal(&pair2.1, &pair1.1, true, &mut cst);
                            push_45_135(&pair2.1, &pair1.1, true, &mut cst);
                            push_vertical(&pair2.1, &pair3.1, true, &mut cst);
                            push_horizontal(&pair2.1, &pair3.1, true, &mut cst);
                            push_45_135(&pair2.1, &pair3.1, true, &mut cst);
                            push_vertical(&pair2.1, &pair4.1, true, &mut cst);
                            push_horizontal(&pair2.1, &pair4.1, true, &mut cst);
                            push_45_135(&pair2.1, &pair4.1, true, &mut cst);
                        }
                        Ctrl2 => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&pair3.1, &pair1.1, true, &mut cst);
                            push_horizontal(&pair3.1, &pair1.1, true, &mut cst);
                            push_45_135(&pair3.1, &pair1.1, true, &mut cst);
                            push_vertical(&pair3.1, &pair2.1, true, &mut cst);
                            push_horizontal(&pair3.1, &pair2.1, true, &mut cst);
                            push_45_135(&pair3.1, &pair2.1, true, &mut cst);
                            push_vertical(&pair3.1, &pair4.1, true, &mut cst);
                            push_horizontal(&pair3.1, &pair4.1, true, &mut cst);
                            push_45_135(&pair3.1, &pair4.1, true, &mut cst);
                        }
                        _ => (),
                    }
                }
            }
            Rectangle(bl, tl, tr, br) => {
                if let Some(selection) = self.selection {
                    match selection {
                        BottomLeft => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_45_135(&bl.1, &tr.1, true, &mut cst);
                        }
                        TopLeft => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_45_135(&tl.1, &br.1, true, &mut cst);
                        }
                        TopRight => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_45_135(&tr.1, &bl.1, true, &mut cst);
                        }
                        BottomRight => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_45_135(&br.1, &tl.1, true, &mut cst);
                        }
                        _ => (),
                    }
                }
            }
            Ellipse(center, radius, h_start_angle, h_end_angle, _) => {
                if let Some(selection) = self.selection {
                    match selection {
                        Center => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&center.1, &h_start_angle.1, true, &mut cst);
                            push_horizontal(&center.1, &h_start_angle.1, true, &mut cst);
                            push_45_135(&center.1, &h_start_angle.1, true, &mut cst);
                            push_vertical(&center.1, &h_end_angle.1, true, &mut cst);
                            push_horizontal(&center.1, &h_end_angle.1, true, &mut cst);
                            push_45_135(&center.1, &h_end_angle.1, true, &mut cst);
                            push_vertical(&center.1, &radius.1, true, &mut cst);
                            push_horizontal(&center.1, &radius.1, true, &mut cst);
                            push_45_135(&center.1, &radius.1, true, &mut cst);
                        }
                        Radius => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&center.1, &radius.1, true, &mut cst);
                            push_horizontal(&center.1, &radius.1, true, &mut cst);
                            push_45_135(&center.1, &radius.1, true, &mut cst);
                        }
                        StartAngle => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&h_start_angle.1, &center.1, true, &mut cst);
                            push_horizontal(&h_start_angle.1, &center.1, true, &mut cst);
                            push_45_135(&h_start_angle.1, &center.1, true, &mut cst);
                            push_vertical(&h_start_angle.1, &h_end_angle.1, true, &mut cst);
                            push_horizontal(&h_start_angle.1, &h_end_angle.1, true, &mut cst);
                            push_45_135(&h_start_angle.1, &h_end_angle.1, true, &mut cst);
                        }
                        EndAngle => {
                            cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
                            push_vertical(&h_end_angle.1, &center.1, true, &mut cst);
                            push_horizontal(&h_end_angle.1, &center.1, true, &mut cst);
                            push_45_135(&h_end_angle.1, &center.1, true, &mut cst);
                            push_vertical(&h_end_angle.1, &h_start_angle.1, true, &mut cst);
                            push_horizontal(&h_end_angle.1, &h_start_angle.1, true, &mut cst);
                            push_45_135(&h_end_angle.1, &h_start_angle.1, true, &mut cst);
                        }
                        _ => (),
                    }
                }
            }
        };
        cst
    }
    pub fn get_bounding_box(&self) -> [WPoint; 2] {
        // Get a list of all highlightable handle-point pairs in the current shape
        use ShapeType::*;
        let pairs = match &self.shape {
            Line(pair1, pair2) => vec![pair1.1, pair2.1],
            QuadBezier(pair1, _, pair3) => vec![pair1.1, pair3.1],
            CubicBezier(pair1, _, _, pair4) => vec![pair1.1, pair4.1],
            Rectangle(pair1, _, _, pair4) => vec![pair1.1, pair4.1],
            Ellipse(pair1, _, _, pair4, _) => vec![pair1.1, pair4.1],
        };
        [pairs[0], pairs[1]]
    }
    pub fn init_done(&mut self) {
        self.init = false;
    }
}

pub struct ShapesGroups {
    // Key is the shape id and the link handle, second usize is the group id
    groups: HashMap<(usize, Handle), usize>,
}

static COUNTER_SHAPES_GROUPS: AtomicUsize = AtomicUsize::new(0);

impl ShapesGroups {
    pub fn new() -> ShapesGroups {
        ShapesGroups {
            groups: HashMap::new(),
        }
    }
    pub fn add(&mut self, shape1: (usize, Handle), shape2: (usize, Handle)) {
        let id = COUNTER_SHAPES_GROUPS.fetch_add(1, Ordering::Relaxed);
        self.groups.insert(shape1, id);
        self.groups.insert(shape2, id);
    }
}
