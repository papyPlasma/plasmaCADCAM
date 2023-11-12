use super::types::{ConstructionType, LayerType, Point, PointType, Shape, WPos};
use crate::{datapool::ShapePool, math::*};

#[derive(Clone)]
pub struct Line {
    start_point: Point,
    end_point: Point,
    position: WPos,
    saved_position: WPos,
    selected: bool,
    init: bool,
}
impl Line {
    pub fn new(start: &WPos, end: &WPos, snap_distance: f64) -> Line {
        let mut start = *start;
        let mut end = *end;
        start = snap_to_snap_grid(&start, snap_distance);
        end = snap_to_snap_grid(&end, snap_distance);

        let end = if start.wx == end.wx || start.wy == end.wy {
            start + snap_distance
        } else {
            end
        };

        let position = start;
        let start_point = Point::new(&(start - position), false, false, false);
        let end_point = Point::new(&(end - position), false, false, false);

        Line {
            start_point,
            end_point,
            position,
            saved_position: position,
            selected: false,
            init: true,
        }
    }
    pub fn is_point_on_line(&self, pos: &WPos, precision: f64) -> bool {
        is_point_on_segment(&self.start_point, &self.end_point, pos, precision)
    }
}
impl Shape for Line {
    fn is_init(&self) -> bool {
        self.init
    }
    fn init_done(&mut self) {
        self.init = false;
    }
    fn get_pos(&self) -> WPos {
        self.position
    }
    fn is_shape_under_pick_pos(&self, pick_pos: &WPos, grab_handle_precision: f64) -> bool {
        let pick_pos = *pick_pos - self.position;
        self.is_point_on_line(&pick_pos, grab_handle_precision / 2.)
    }
    fn get_shape_point_under_pick_pos(
        &mut self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<PointType> {
        // The first point found is returned
        let pick_pos = *pick_pos - self.position;
        if is_point_on_point(&pick_pos, &self.end_point.wpos, grab_handle_precision) {
            return Some(PointType::End);
        }
        if is_point_on_point(&pick_pos, &self.start_point.wpos, grab_handle_precision) {
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
        self.end_point.selected = false;
    }
    fn is_selected(&self) -> bool {
        self.selected
    }
    fn move_selection(
        &mut self,
        pick_pos: &WPos,
        pick_pos_ms_dwn: &WPos,
        snap_distance: f64,
        _magnet_distance: f64,
    ) {
        if self.selected {
            match (self.start_point.selected, self.end_point.selected) {
                (true, false) => {
                    self.start_point.wpos = *pick_pos - self.position;
                    if self.start_point.wpos == self.end_point.wpos {
                        self.start_point.wpos += snap_distance;
                    }
                }
                (false, true) => {
                    self.end_point.wpos = *pick_pos - self.position;
                    if self.end_point.wpos == self.start_point.wpos {
                        self.end_point.wpos += 2. * snap_distance;
                    }
                }
                (false, false) => {
                    self.position = self.saved_position + *pick_pos - *pick_pos_ms_dwn;
                }
                _ => (),
            }
        }
    }
    fn select_point(&mut self, point_type: &PointType) {
        (self.start_point.selected, self.end_point.selected) = match point_type {
            PointType::Start => (true, false),
            PointType::End => (false, true),
            _ => (false, false),
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
        // let ctrl_pos = self.ctrl_point.wpos;
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
        cst.push(ConstructionType::Line(self.position + self.end_point.wpos));
        cst
    }
    fn get_handles_construction(&self, size_handle: f64) -> Vec<ConstructionType> {
        let mut cst = Vec::new();

        let mut start_point = self.start_point;
        start_point.wpos += self.position;

        let mut end_point = self.end_point;
        end_point.wpos += self.position;

        push_handle(&mut cst, &start_point, size_handle);
        push_handle(&mut cst, &end_point, size_handle);
        cst
    }
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];

        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
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
        cst
    }
    fn get_bounded_rectangle(&self) -> [WPos; 2] {
        [
            self.position + self.start_point.wpos,
            self.position + self.end_point.wpos,
        ]
    }
}
impl ShapePool for Line {}
