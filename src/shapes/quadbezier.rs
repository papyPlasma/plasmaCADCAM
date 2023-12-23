use super::types::{ConstructionType, LayerType, Point, PointType, Shape, WPos};
use crate::math::*;

#[derive(Clone)]
pub struct QuadBezier {
    start_point: Point,
    ctrl_point: Point,
    end_point: Point,
    position: WPos,
    saved_position: WPos,
    selected: bool,
    init: bool,
}
impl QuadBezier {
    pub fn new(start: &WPos, ctrl: &WPos, end: &WPos) -> Option<QuadBezier> {
        let start = *start;
        let ctrl = *ctrl;
        let end = *end;

        if start == end || start == ctrl || ctrl == end {
            return None;
        };

        let position = start;
        let start_point = Point::new(&(start - position), false, false, false);
        let ctrl_point = Point::new(&(ctrl - position), false, false, false);
        let end_point = Point::new(&(end - position), false, false, false);

        Some(QuadBezier {
            start_point,
            ctrl_point,
            end_point,
            position,
            saved_position: position,
            selected: false,
            init: true,
        })
    }
    fn get_point_on_quad_bezier(&self, t: f64) -> WPos {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;

        let mut result = WPos::default();
        result.wx = uu * self.start_point.wpos.wx
            + 2.0 * u * t * self.ctrl_point.wpos.wx
            + tt * self.end_point.wpos.wx;
        result.wy = uu * self.start_point.wpos.wy
            + 2.0 * u * t * self.ctrl_point.wpos.wy
            + tt * self.end_point.wpos.wy;

        result
    }
    pub fn is_point_on_quadbezier(&self, pos: &WPos, precision: f64) -> bool {
        let mut t_min = 0.;
        let mut t_max = 1.;
        let mut min_dist = f64::MAX;

        for _i in 0..MAX_ITERATIONS {
            // max iterations can be adjusted
            let t_mid = (t_min + t_max) / 2.;
            let bt = self.get_point_on_quad_bezier(t_mid);
            let dist = bt.dist(pos);
            if dist < min_dist {
                min_dist = dist;
            }
            if dist < precision {
                return true; // We found a sufficiently close point
            }
            // Using gradient to decide the next tMid for the next iteration.
            let gradient = (bt.wx - pos.wx) * (self.end_point.wpos.wx - self.start_point.wpos.wx)
                + (bt.wy - pos.wy) * (self.end_point.wpos.wy - self.start_point.wpos.wy);
            if gradient > 0. {
                t_max = t_mid;
            } else {
                t_min = t_mid;
            }
        }
        min_dist <= precision
    }
}

impl Shape for QuadBezier {
    fn is_init(&self) -> bool {
        self.init
    }
    fn init_done(&mut self) {
        self.init = false;
    }
    fn get_pos(&self) -> WPos {
        self.position
    }
    fn get_step_r(&self, grab_handle_precision: f64) -> f64 {
        0.
    }

    fn get_pos_from_ratio(&self, r: f64) -> WPos {
        let s = self.start_point.wpos;
        let c = self.ctrl_point.wpos;
        let e = self.end_point.wpos;
        let r1 = 1.0 - r;
        let wx = r1.powi(2) * s.wx + 2.0 * r1 * r * c.wx + r.powi(2) * e.wx;
        let wy = r1.powi(2) * s.wy + 2.0 * r1 * r * c.wy + r.powi(2) * e.wy;
        WPos { wx, wy }
    }

    fn get_ratio_from_pos(&self, rpos: &WPos) -> f64 {
        // TODO
        0.
    }
    fn get_projected_pos(&self, pick_pos: &WPos) -> WPos {
        // TODO
        WPos::zero()
    }
    fn split(&self, pos: &WPos) -> (Option<Box<dyn Shape>>, Option<Box<dyn Shape>>) {
        // TODO
        (None, None)
    }

    fn dist(&self, pick_pos: &WPos) -> f64 {
        // TODO
        0.

        // TODO
        // let steps = 100; // Increase for better precision
        // let mut min_distance = f64::MAX;

        // for i in 0..steps {
        //     let r1 = i as f64 / steps as f64;
        //     let r2 = (i + 1) as f64 / steps as f64;
        //     let start_pos = self.get_pos_from_ratio(r1);
        //     let end_pos = self.get_pos_from_ratio(r2);
        //     let distance = pos.sign_dist_to_seg(&start_pos, &end_pos);
        //     min_distance = min_distance.min(distance);
        // }
        // min_distance
    }

    // fn is_shape_under_pick_pos(&self, pick_pos: &WPos, grab_handle_precision: f64) -> bool {
    //     let pick_pos = *pick_pos - self.position;
    //     self.is_point_on_quadbezier(&pick_pos, grab_handle_precision / 2.)
    // }
    fn get_shape_point_type_under_pick_pos(
        &mut self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<PointType> {
        // The first point found is returned
        let pick_pos = *pick_pos - self.position;
        if pick_pos.dist(&self.end_point.wpos) < grab_handle_precision {
            return Some(PointType::End);
        }
        if pick_pos.dist(&self.ctrl_point.wpos) < grab_handle_precision {
            return Some(PointType::Ctrl);
        }
        if pick_pos.dist(&self.start_point.wpos) < grab_handle_precision {
            return Some(PointType::Start);
        }
        None
    }
    fn clear_selection(&mut self) {
        self.selected = false
    }
    fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }
    fn deselect_all_points(&mut self) {
        self.start_point.selected = false;
        self.ctrl_point.selected = false;
        self.end_point.selected = false;
    }
    fn is_selected(&self) -> bool {
        self.selected
    }
    fn move_selection(&mut self, pick_pos: &WPos, pick_pos_ms_dwn: &WPos, _magnet_distance: f64) {
        let pick_pos = *pick_pos;
        let pick_pos_ms_dwn = *pick_pos_ms_dwn;

        if self.init {
            self.start_point.selected = false;
            self.ctrl_point.selected = false;
            self.end_point.selected = true;
        }
        if self.selected {
            match (
                self.start_point.selected,
                self.ctrl_point.selected,
                self.end_point.selected,
            ) {
                (true, false, false) => {
                    let pos = pick_pos - self.position;
                    if pos != self.end_point.wpos && pos != self.ctrl_point.wpos {
                        self.start_point.wpos = pos;
                    }
                }
                (false, true, false) => {
                    let pos = pick_pos - self.position;
                    if pos != self.start_point.wpos && pos != self.end_point.wpos {
                        self.ctrl_point.wpos = pos;
                    }
                }
                (false, false, true) => {
                    let pos = pick_pos - self.position;
                    if pos != self.start_point.wpos && pos != self.ctrl_point.wpos {
                        self.end_point.wpos = pos;
                        if self.init {
                            self.ctrl_point.wpos =
                                (self.start_point.wpos + self.end_point.wpos) / 2.;
                        }
                    }
                }
                (false, false, false) => {
                    self.position = self.saved_position + pick_pos - pick_pos_ms_dwn;
                }
                _ => (),
            }
        }
    }
    fn select_point_type(&mut self, point_type: &PointType) {
        (
            self.start_point.selected,
            self.ctrl_point.selected,
            self.end_point.selected,
        ) = match point_type {
            PointType::Start => (true, false, false),
            PointType::Ctrl => (false, true, false),
            PointType::End => (false, false, true),
            _ => (false, false, false),
        }
    }

    fn save_current_position(&mut self) {
        self.saved_position = self.position;
    }
    fn get_saved_position(&self) -> WPos {
        self.saved_position
    }
    fn magnet_to_point(&self, pick_pos: &mut WPos, magnet_distance: f64) {
        let start_pos = self.start_point.wpos;
        let end_pos = self.start_point.wpos;

        if pick_pos.dist(&(start_pos + self.position)) < magnet_distance {
            *pick_pos = self.position + start_pos;
        }
        if pick_pos.dist(&(end_pos + self.position)) < magnet_distance {
            *pick_pos = self.position + end_pos;
        }
    }

    fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        if !self.selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        cst.push(ConstructionType::Move(
            self.position + self.start_point.wpos,
        ));
        cst.push(ConstructionType::QuadBezier(
            self.position + self.ctrl_point.wpos,
            self.position + self.end_point.wpos,
        ));
        cst
    }
    fn get_handles_construction(&self, size_handle: f64) -> Vec<ConstructionType> {
        let mut cst = Vec::new();

        let mut start_point = self.start_point;
        start_point.wpos += self.position;

        let mut ctrl_point = self.ctrl_point;
        ctrl_point.wpos += self.position;

        let mut end_point = self.end_point;
        end_point.wpos += self.position;

        push_handle(&mut cst, &start_point, size_handle);
        push_handle(&mut cst, &ctrl_point, size_handle);
        push_handle(&mut cst, &end_point, size_handle);
        cst
    }
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        // start - end
        if is_aligned_vert(&self.start_point.wpos, &self.end_point.wpos) {
            helper_vertical(
                &(self.position + self.start_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&self.start_point.wpos, &self.end_point.wpos) {
            helper_horizontal(
                &(self.position + self.start_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&self.start_point.wpos, &self.end_point.wpos) {
            helper_45_135(
                &(self.position + self.start_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        // ctrl - end
        if is_aligned_vert(&self.ctrl_point.wpos, &self.end_point.wpos) {
            helper_vertical(
                &(self.position + self.ctrl_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&self.ctrl_point.wpos, &self.end_point.wpos) {
            helper_horizontal(
                &(self.position + self.ctrl_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&self.ctrl_point.wpos, &self.end_point.wpos) {
            helper_45_135(
                &(self.position + self.ctrl_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        // ctrl - start
        if is_aligned_vert(&self.ctrl_point.wpos, &self.start_point.wpos) {
            helper_vertical(
                &(self.position + self.ctrl_point.wpos),
                &(self.position + self.start_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&self.ctrl_point.wpos, &self.start_point.wpos) {
            helper_horizontal(
                &(self.position + self.ctrl_point.wpos),
                &(self.position + self.start_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&self.ctrl_point.wpos, &self.start_point.wpos) {
            helper_45_135(
                &(self.position + self.ctrl_point.wpos),
                &(self.position + self.start_point.wpos),
                true,
                &mut cst,
            );
        }
        cst
    }
    fn get_bounded_rectangle(&self) -> [WPos; 2] {
        [
            self.position + self.start_point.wpos,
            self.position + self.end_point.wpos,
        ]
    }
}
// impl ShapePool for QuadBezier {}
