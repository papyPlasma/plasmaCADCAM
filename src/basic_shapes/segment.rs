// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[cfg(not(test))]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use crate::math::*;
use crate::types::*;

#[derive(Copy, Clone)]
pub struct Segment;
impl Segment {
    // pub fn save_current_position(&mut self) {
    //     self.common.save_pos()
    // }
    // pub fn get_bounded_rectangle(&self) -> [WPos; 2] {
    //     [
    //         *self.common.get_pos() + self.start_point.wpos,
    //         *self.common.get_pos() + self.end_point.wpos,
    //     ]
    // }
    // pub fn s_dist(pick_pos: &WPos, start_pos: WPos, end_pos: WPos) -> f64 {
    //     let s = self.start_point.wpos + *self.common.get_pos();
    //     let e = self.end_point.wpos + *self.common.get_pos();
    //     pick_pos.s_dist_seg(&s, &e)
    // }
    // pub fn get_point_under_pos(
    //     &self,
    //     pick_pos: &WPos,
    //     grab_handle_precision: f64,
    // ) -> Option<PointType> {
    //     // The first point found is returned
    //     let pick_pos = *pick_pos - *self.common.get_pos();
    //     if pick_pos.dist(&self.end_point.wpos) < grab_handle_precision {
    //         return Some(PointType::End);
    //     }
    //     if pick_pos.dist(&self.start_point.wpos) < grab_handle_precision {
    //         return Some(PointType::Start);
    //     }
    //     None
    // }

    // pub fn clear_selections(&mut self) {
    //     self.common.set_selection(false);
    //     self.start_point.selected = false;
    //     self.end_point.selected = false;
    // }
    // pub fn set_selection(&mut self, selection: bool) {
    //     self.common.set_selection(selection);
    // }
    // pub fn set_selection_point(&mut self, pt_typ: &PointType, selection: bool) {
    //     match pt_typ {
    //         PointType::Start => self.start_point.selected = selection,
    //         PointType::End => self.end_point.selected = selection,
    //         _ => (),
    //     }
    // }

    // fn move_selection(&mut self, pick_pos: &WPos, pick_pos_ms_dwn: &WPos, _magnet_distance: f64) {
    //     let pick_pos = *pick_pos;
    //     let pick_pos_ms_dwn = *pick_pos_ms_dwn;

    //     // if self.common.init {
    //     //     self.start_point.selected = false;
    //     //     self.end_point.selected = true;
    //     // }
    //     if self.common.selected {
    //         match (self.start_point.selected, self.end_point.selected) {
    //             (true, false) => {
    //                 let pos = pick_pos - self.common.position;
    //                 if pos != self.end_point.wpos {
    //                     self.start_point.wpos = pos;
    //                 }
    //             }
    //             (false, true) => {
    //                 let pos = pick_pos - self.common.position;
    //                 if pos != self.start_point.wpos {
    //                     self.end_point.wpos = pos;
    //                 }
    //             }
    //             (false, false) => {
    //                 self.common.position = self.common.saved_position + pick_pos - pick_pos_ms_dwn;
    //             }
    //             _ => (),
    //         }
    //     }
    // }

    // fn get_handles_construction(&self, size_handle: f64) -> Vec<ConstructionType> {
    //     let mut cst = Vec::new();

    //     let mut start_point = self.start_point;
    //     start_point.wpos += self.position;

    //     let mut end_point = self.end_point;
    //     end_point.wpos += self.position;

    //     push_handle(&mut cst, &start_point, size_handle);
    //     push_handle(&mut cst, &end_point, size_handle);
    //     cst
    // }
    // fn get_helpers_construction(&self) -> Vec<ConstructionType> {
    //     let mut cst: Vec<ConstructionType> = vec![];

    //     cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //     if is_aligned_vert(&self.start_point.wpos, &self.end_point.wpos) {
    //         helper_vertical(
    //             &(self.position + self.start_point.wpos),
    //             &(self.position + self.end_point.wpos),
    //             true,
    //             &mut cst,
    //         );
    //     }
    //     if is_aligned_hori(&self.start_point.wpos, &self.end_point.wpos) {
    //         helper_horizontal(
    //             &(self.position + self.start_point.wpos),
    //             &(self.position + self.end_point.wpos),
    //             true,
    //             &mut cst,
    //         );
    //     }
    //     if is_aligned_45_or_135(&self.start_point.wpos, &self.end_point.wpos) {
    //         helper_45_135(
    //             &(self.position + self.start_point.wpos),
    //             &(self.position + self.end_point.wpos),
    //             true,
    //             &mut cst,
    //         );
    //     }
    //     cst
    // }
}

// impl Shape for Line {

//     fn get_pos(&self) -> WPos {
//         self.position
//     }
//     fn get_step_r(&self, step: f64) -> f64 {
//         let dist = self.start_point.wpos.dist(&self.end_point.wpos);
//         if dist > 0. {
//             step / dist
//         } else {
//             step
//         }
//     }
//     fn get_pos_from_ratio(&self, r: f64) -> WPos {
//         let s = self.start_point.wpos;
//         let e = self.end_point.wpos;
//         (e - s) * r + s + self.position
//     }
//     fn get_ratio_from_pos(&self, pos: &WPos) -> f64 {
//         (*pos - self.position).ratio(&self.start_point.wpos, &self.end_point.wpos)
//     }
//     fn get_projected_pos(&self, pick_pos: &WPos) -> WPos {
//         (*pick_pos - self.position).project_to_seg(&self.start_point.wpos, &self.end_point.wpos)
//             + self.position
//     }
//     fn split(&self, pos: &WPos) -> (Option<Box<dyn Shape>>, Option<Box<dyn Shape>>) {
//         let start_pos = self.start_point.wpos + self.position;
//         let end_pos = self.end_point.wpos + self.position;
//         (
//             if let Some(mut line1) = Line::new(&start_pos, pos) {
//                 line1.init_done();
//                 Some(Box::new(line1))
//             } else {
//                 None
//             },
//             if let Some(mut line2) = Line::new(pos, &end_pos) {
//                 line2.init_done();
//                 Some(Box::new(line2))
//             } else {
//                 None
//             },
//         )
//     }

//     fn set_selected(&mut self, selected: bool) {
//         self.selected = selected;
//     }
//     fn deselect_all_points(&mut self) {
//         self.start_point.selected = false;
//         self.end_point.selected = false;
//     }
//     fn is_selected(&self) -> bool {
//         self.selected
//     }

//     fn save_current_position(&mut self) {
//         self.saved_position = self.position;
//     }
//     fn get_saved_position(&self) -> WPos {
//         self.saved_position
//     }
//     fn magnet_to_point(&self, pick_pos: &mut WPos, magnet_distance: f64) {
//         let start_pos = self.start_point.wpos;
//         let end_pos = self.start_point.wpos;

//         if pick_pos.dist(&(start_pos + self.position)) < magnet_distance {
//             *pick_pos = self.position + start_pos;
//         }
//         if pick_pos.dist(&(end_pos + self.position)) < magnet_distance {
//             *pick_pos = self.position + end_pos;
//         }
//     }

//     fn get_bounded_rectangle(&self) -> [WPos; 2] {
//         [
//             self.position + self.start_point.wpos,
//             self.position + self.end_point.wpos,
//         ]
//     }
// }
// // impl ShapePool for Line {}
