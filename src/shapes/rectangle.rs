use super::types::{ConstructionType, LayerType, Point, PointType, Shape, WPos};
use crate::{datapool::ShapePool, math::*};

#[derive(Clone)]
pub struct Rectangle {
    bl_pt: Point,
    tl_pt: Point,
    tr_pt: Point,
    br_pt: Point,
    position: WPos,
    saved_position: WPos,
    selected: bool,
    init: bool,
}
impl Rectangle {
    pub fn new(position: &WPos, w: f64, h: f64, snap_distance: f64) -> Rectangle {
        let position = snap_to_snap_grid(position, snap_distance);
        let w = snap_to_positive_value(w, snap_distance);
        let h = snap_to_positive_value(h, snap_distance);

        // let w = 30.;
        // let h = 40.;
        let tl_pt = Point::new(&WPos::zero(), true, true, false);
        let tr_pt = Point::new(&WPos::zero().addxy(w, 0.), true, true, false);
        let br_pt = Point::new(&WPos::zero().addxy(w, h), true, true, false);
        let bl_pt = Point::new(&WPos::zero().addxy(0., h), true, true, false);

        let rectangle = Rectangle {
            bl_pt,
            tl_pt,
            tr_pt,
            br_pt,
            position,
            saved_position: position,
            selected: false,
            init: true,
        };
        rectangle
    }
    pub fn is_point_on_rectangle(&self, pos: &WPos, precision: f64) -> bool {
        is_point_on_segment(&self.bl_pt, &self.tl_pt, pos, precision)
            || is_point_on_segment(&self.tl_pt, &self.tr_pt, pos, precision)
            || is_point_on_segment(&self.tr_pt, &self.br_pt, pos, precision)
            || is_point_on_segment(&self.br_pt, &self.bl_pt, pos, precision)
    }
    // pub fn reorder_pts(&mut self) {
    //     let min_wx = self.tl_pt.wpos.wx.min(
    //         self.tr_pt
    //             .wpos
    //             .wx
    //             .min(self.br_pt.wpos.wx.min(self.bl_pt.wpos.wx)),
    //     );
    //     let min_wy = self.tl_pt.wpos.wy.min(
    //         self.tr_pt
    //             .wpos
    //             .wy
    //             .min(self.br_pt.wpos.wy.min(self.bl_pt.wpos.wy)),
    //     );
    //     let max_wx = self.tl_pt.wpos.wx.max(
    //         self.tr_pt
    //             .wpos
    //             .wx
    //             .max(self.br_pt.wpos.wx.max(self.bl_pt.wpos.wx)),
    //     );
    //     let max_wy = self.tl_pt.wpos.wy.max(
    //         self.tr_pt
    //             .wpos
    //             .wy
    //             .max(self.br_pt.wpos.wy.max(self.bl_pt.wpos.wy)),
    //     );
    //     self.tl_pt.wpos.wx = min_wx;
    //     self.tl_pt.wpos.wy = min_wy;
    //     self.br_pt.wpos.wx = max_wx;
    //     self.br_pt.wpos.wy = max_wy;
    //     self.tr_pt.wpos.wx = max_wx;
    //     self.tr_pt.wpos.wy = min_wy;
    //     self.bl_pt.wpos.wx = min_wx;
    //     self.bl_pt.wpos.wy = max_wy;
    // }
}
impl Shape for Rectangle {
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
        self.is_point_on_rectangle(&pick_pos, grab_handle_precision / 2.)
    }
    fn get_shape_point_under_pick_pos(
        &mut self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<PointType> {
        // The first point found is returned
        let pick_pos = *pick_pos - self.position;
        if is_point_on_point(&pick_pos, &self.bl_pt.wpos, grab_handle_precision) {
            return Some(PointType::BL);
        }
        if is_point_on_point(&pick_pos, &self.tl_pt.wpos, grab_handle_precision) {
            return Some(PointType::TL);
        }
        if is_point_on_point(&pick_pos, &self.tr_pt.wpos, grab_handle_precision) {
            return Some(PointType::TR);
        }
        if is_point_on_point(&pick_pos, &self.br_pt.wpos, grab_handle_precision) {
            return Some(PointType::BR);
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
        self.bl_pt.selected = false;
        self.tl_pt.selected = false;
        self.tr_pt.selected = false;
        self.br_pt.selected = false;
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
        if self.init {
            self.tl_pt.selected = false;
            self.tr_pt.selected = false;
            self.br_pt.selected = true;
            self.bl_pt.selected = false;
        }
        if self.selected {
            match (
                self.tl_pt.selected,
                self.tr_pt.selected,
                self.br_pt.selected,
                self.bl_pt.selected,
            ) {
                (true, false, false, false) => {
                    let new_tl_pos = snap_to_snap_grid(&(*pick_pos - self.position), snap_distance);
                    self.tl_pt.wpos = new_tl_pos;
                    self.bl_pt.wpos.wx = self.tl_pt.wpos.wx;
                    self.tr_pt.wpos.wy = self.tl_pt.wpos.wy;
                }
                (false, true, false, false) => {
                    let new_tr_pos = snap_to_snap_grid(&(*pick_pos - self.position), snap_distance);
                    self.tr_pt.wpos = new_tr_pos;
                    self.br_pt.wpos.wx = self.tr_pt.wpos.wx;
                    self.tl_pt.wpos.wy = self.tr_pt.wpos.wy;
                }
                (false, false, true, false) => {
                    let new_br_pos = snap_to_snap_grid(&(*pick_pos - self.position), snap_distance);
                    self.br_pt.wpos = new_br_pos;
                    self.tr_pt.wpos.wx = self.br_pt.wpos.wx;
                    self.bl_pt.wpos.wy = self.br_pt.wpos.wy;
                }
                (false, false, false, true) => {
                    let new_bl_pos = snap_to_snap_grid(&(*pick_pos - self.position), snap_distance);
                    self.bl_pt.wpos = new_bl_pos;
                    self.tl_pt.wpos.wx = self.bl_pt.wpos.wx;
                    self.br_pt.wpos.wy = self.bl_pt.wpos.wy;
                }
                (false, false, false, false) => {
                    self.position = self.saved_position + *pick_pos - *pick_pos_ms_dwn;
                }
                _ => (),
            }
        }
    }
    fn select_point(&mut self, point_type: &PointType) {
        (
            self.bl_pt.selected,
            self.tl_pt.selected,
            self.tr_pt.selected,
            self.br_pt.selected,
        ) = match point_type {
            PointType::BL => (true, false, false, false),
            PointType::TL => (false, true, false, false),
            PointType::TR => (false, false, true, false),
            PointType::BR => (false, false, false, true),
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
        let bl_pos = self.bl_pt.wpos;
        let tl_pos = self.tl_pt.wpos;
        let tr_pos = self.tr_pt.wpos;
        let br_pos = self.br_pt.wpos;

        if pick_pos.dist(&(bl_pos + self.position)) < magnet_distance {
            *pick_pos = self.position + bl_pos;
        }
        if pick_pos.dist(&(tl_pos + self.position)) < magnet_distance {
            *pick_pos = self.position + tl_pos;
        }
        if pick_pos.dist(&(tr_pos + self.position)) < magnet_distance {
            *pick_pos = self.position + tr_pos;
        }
        if pick_pos.dist(&(br_pos + self.position)) < magnet_distance {
            *pick_pos = self.position + br_pos;
        }
    }

    fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        if !self.selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        cst.push(ConstructionType::Move(self.position + self.bl_pt.wpos));
        cst.push(ConstructionType::Line(self.position + self.tl_pt.wpos));
        cst.push(ConstructionType::Line(self.position + self.tr_pt.wpos));
        cst.push(ConstructionType::Line(self.position + self.br_pt.wpos));
        cst.push(ConstructionType::Line(self.position + self.bl_pt.wpos));
        cst
    }
    fn get_handles_construction(&self, size_handle: f64) -> Vec<ConstructionType> {
        let mut cst = Vec::new();

        let mut bl_point = self.bl_pt;
        bl_point.wpos += self.position;
        let mut tl_point = self.tl_pt;
        tl_point.wpos += self.position;
        let mut tr_point = self.tr_pt;
        tr_point.wpos += self.position;
        let mut br_point = self.br_pt;
        br_point.wpos += self.position;

        push_handle(&mut cst, &bl_point, size_handle);
        push_handle(&mut cst, &tl_point, size_handle);
        push_handle(&mut cst, &tr_point, size_handle);
        push_handle(&mut cst, &br_point, size_handle);
        cst
    }
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];

        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));

        if is_aligned_45_or_135(&self.tl_pt.wpos, &self.br_pt.wpos) {
            helper_45_135(
                &(self.position + self.tl_pt.wpos),
                &(self.position + self.br_pt.wpos),
                true,
                &mut cst,
            );
        }

        if is_aligned_45_or_135(&self.bl_pt.wpos, &self.tr_pt.wpos) {
            helper_45_135(
                &(self.position + self.bl_pt.wpos),
                &(self.position + self.tr_pt.wpos),
                true,
                &mut cst,
            );
        }

        cst
    }
    fn get_bounded_rectangle(&self) -> [WPos; 2] {
        [
            self.position + self.bl_pt.wpos,
            self.position + self.tl_pt.wpos,
        ]
    }
}
impl ShapePool for Rectangle {}
