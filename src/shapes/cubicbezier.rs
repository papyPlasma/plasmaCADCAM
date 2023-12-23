use super::types::{ConstructionType, LayerType, Point, PointType, Shape, WPos};
use crate::math::*;

#[derive(Clone)]
pub struct CubicBezier {
    start_point: Point,
    ctrl1_point: Point,
    ctrl2_point: Point,
    end_point: Point,
    position: WPos,
    saved_position: WPos,
    selected: bool,
    init: bool,
}
impl CubicBezier {
    pub fn new(start: &WPos, ctrl1: &WPos, ctrl2: &WPos, end: &WPos) -> Option<CubicBezier> {
        let start = *start;
        let ctrl1 = *ctrl1;
        let ctrl2 = *ctrl2;
        let end = *end;

        if start == end || start == ctrl1 || start == ctrl2 || ctrl1 == end || ctrl2 == end {
            return None;
        };

        let position = start;
        let start_point = Point::new(&(start - position), false, false, false);
        let ctrl1_point = Point::new(&(ctrl1 - position), false, false, false);
        let ctrl2_point = Point::new(&(ctrl2 - position), false, false, false);
        let end_point = Point::new(&(end - position), false, false, false);

        Some(CubicBezier {
            start_point,
            ctrl1_point,
            ctrl2_point,
            end_point,
            position,
            saved_position: position,
            selected: false,
            init: true,
        })
    }
    pub fn get_point_on_cubic_bezier(&self, t: f64) -> WPos {
        let u = 1.0 - t;
        let tt = t * t;
        let uu = u * u;
        let uuu = uu * u;
        let ttt = tt * t;

        let mut result = self.start_point.wpos * uuu; // (1-t)^3 * start
        result += self.ctrl1_point.wpos * 3.0 * uu * t; // 3(1-t)^2 * t * ctrl1
        result += self.ctrl2_point.wpos * 3.0 * u * tt; // 3(1-t) * t^2 * ctrl2
        result += self.end_point.wpos * ttt; // t^3 * end

        result
    }
    pub fn is_point_on_cubicbezier(&self, pos: &WPos, precision: f64) -> bool {
        let mut t_min = 0.;
        let mut t_max = 1.;
        let mut min_dist = f64::MAX;
        for _i in 0..MAX_ITERATIONS {
            let t_mid = (t_min + t_max) / 2.;
            let bt = self.get_point_on_cubic_bezier(t_mid);
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
impl Shape for CubicBezier {
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
        //
        0.
    }
    fn get_pos_from_ratio(&self, r: f64) -> WPos {
        // TODO
        WPos::zero()
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
    }
    // fn is_shape_under_pick_pos(&self, pick_pos: &WPos, grab_handle_precision: f64) -> bool {
    //     let pick_pos = *pick_pos - self.position;
    //     self.is_point_on_cubicbezier(&pick_pos, grab_handle_precision / 2.)
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
        if pick_pos.dist(&self.ctrl1_point.wpos) < grab_handle_precision {
            return Some(PointType::Ctrl1);
        }
        if pick_pos.dist(&self.ctrl2_point.wpos) < grab_handle_precision {
            return Some(PointType::Ctrl2);
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
        self.ctrl1_point.selected = false;
        self.ctrl2_point.selected = false;
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
            self.ctrl1_point.selected = false;
            self.ctrl2_point.selected = false;
            self.end_point.selected = true;
        }
        if self.selected {
            match (
                self.start_point.selected,
                self.ctrl1_point.selected,
                self.ctrl2_point.selected,
                self.end_point.selected,
            ) {
                (true, false, false, false) => {
                    let pos = pick_pos - self.position;
                    if pos != self.ctrl1_point.wpos
                        && pos != self.ctrl2_point.wpos
                        && pos != self.end_point.wpos
                    {
                        self.start_point.wpos = pos;
                    }
                }
                (false, true, false, false) => {
                    let pos = pick_pos - self.position;
                    if pos != self.start_point.wpos
                        && pos != self.ctrl2_point.wpos
                        && pos != self.end_point.wpos
                    {
                        self.ctrl1_point.wpos = pos;
                    }
                }
                (false, false, true, false) => {
                    let pos = pick_pos - self.position;
                    if pos != self.start_point.wpos
                        && pos != self.ctrl1_point.wpos
                        && pos != self.end_point.wpos
                    {
                        self.ctrl2_point.wpos = pos;
                    }
                }
                (false, false, false, true) => {
                    let pos = pick_pos - self.position;
                    if pos != self.start_point.wpos
                        && pos != self.ctrl1_point.wpos
                        && pos != self.ctrl2_point.wpos
                    {
                        self.end_point.wpos = pos;
                        if self.init {
                            self.ctrl1_point.wpos =
                                (self.start_point.wpos + self.end_point.wpos) / 3.;
                            self.ctrl2_point.wpos =
                                (self.start_point.wpos + self.end_point.wpos) * 2. / 3.;
                        }
                    }
                }
                (false, false, false, false) => {
                    self.position = self.saved_position + pick_pos - pick_pos_ms_dwn;
                }
                _ => (),
            }
        }
    }
    fn select_point_type(&mut self, point_type: &PointType) {
        (
            self.start_point.selected,
            self.ctrl1_point.selected,
            self.ctrl2_point.selected,
            self.end_point.selected,
        ) = match point_type {
            PointType::Start => (true, false, false, false),
            PointType::Ctrl1 => (false, true, false, false),
            PointType::Ctrl2 => (false, false, true, false),
            PointType::End => (false, false, false, true),
            _ => (false, false, false, false),
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
        cst.push(ConstructionType::CubicBezier(
            self.position + self.ctrl1_point.wpos,
            self.position + self.ctrl2_point.wpos,
            self.position + self.end_point.wpos,
        ));
        cst
    }
    fn get_handles_construction(&self, size_handle: f64) -> Vec<ConstructionType> {
        let mut cst = Vec::new();

        let mut start_point = self.start_point;
        start_point.wpos += self.position;

        let mut ctrl1_point = self.ctrl1_point;
        ctrl1_point.wpos += self.position;

        let mut ctrl2_point = self.ctrl2_point;
        ctrl2_point.wpos += self.position;

        let mut end_point = self.end_point;
        end_point.wpos += self.position;

        push_handle(&mut cst, &start_point, size_handle);
        push_handle(&mut cst, &ctrl1_point, size_handle);
        push_handle(&mut cst, &ctrl2_point, size_handle);
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
        // ctrl1 - end
        if is_aligned_vert(&self.ctrl1_point.wpos, &self.end_point.wpos) {
            helper_vertical(
                &(self.position + self.ctrl1_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&self.ctrl1_point.wpos, &self.end_point.wpos) {
            helper_horizontal(
                &(self.position + self.ctrl1_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&self.ctrl1_point.wpos, &self.end_point.wpos) {
            helper_45_135(
                &(self.position + self.ctrl1_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        // ctrl1 - start
        if is_aligned_vert(&self.ctrl1_point.wpos, &self.start_point.wpos) {
            helper_vertical(
                &(self.position + self.ctrl1_point.wpos),
                &(self.position + self.start_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&self.ctrl1_point.wpos, &self.start_point.wpos) {
            helper_horizontal(
                &(self.position + self.ctrl1_point.wpos),
                &(self.position + self.start_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&self.ctrl1_point.wpos, &self.start_point.wpos) {
            helper_45_135(
                &(self.position + self.ctrl1_point.wpos),
                &(self.position + self.start_point.wpos),
                true,
                &mut cst,
            );
        }
        // ctrl2 - end
        if is_aligned_vert(&self.ctrl2_point.wpos, &self.end_point.wpos) {
            helper_vertical(
                &(self.position + self.ctrl2_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&self.ctrl2_point.wpos, &self.end_point.wpos) {
            helper_horizontal(
                &(self.position + self.ctrl2_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&self.ctrl2_point.wpos, &self.end_point.wpos) {
            helper_45_135(
                &(self.position + self.ctrl2_point.wpos),
                &(self.position + self.end_point.wpos),
                true,
                &mut cst,
            );
        }
        // ctrl2 - start
        if is_aligned_vert(&self.ctrl2_point.wpos, &self.start_point.wpos) {
            helper_vertical(
                &(self.position + self.ctrl2_point.wpos),
                &(self.position + self.start_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_hori(&self.ctrl2_point.wpos, &self.start_point.wpos) {
            helper_horizontal(
                &(self.position + self.ctrl2_point.wpos),
                &(self.position + self.start_point.wpos),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&self.ctrl2_point.wpos, &self.start_point.wpos) {
            helper_45_135(
                &(self.position + self.ctrl2_point.wpos),
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
// impl ShapePool for CubicBezier {}
