use crate::types::*;

pub enum BasicShapeType {
    Segment(bool),
    // QBezier(bool),
    // CBezier(bool),
    // ArcEllipse(bool),
}
// impl BasicShapeType {

//     pub fn save_current_position(&mut self) {
//         use BasicShapeType::*;
//         match self {
//             Segment(segment) => segment.save_current_position(),
//             // QBezier(qbezier) => qbezier.save_current_position(),
//             // CBezier(cbezier) => cbezier.save_current_position(),
//             // ArcEllipse(aellipse) => aellipse.save_current_position(),
//         }
//     }

//     pub fn s_dist(&self, pos: &WPos) -> f64 {
//         use BasicShapeType::*;
//         match self {
//             Segment(segment) => segment.s_dist(pos),
//             // QBezier(qbezier) => qbezier.s_dist(pos),
//             // CBezier(cbezier) => cbezier.s_dist(pos),
//             // ArcEllipse(aellipse) => aellipse.s_dist(pos),
//         }
//     }
//     pub fn get_point_under_pos(
//         &self,
//         pick_pos: &WPos,
//         grab_handle_precision: f64,
//     ) -> Option<PointType> {
//         use BasicShapeType::*;
//         match self {
//             Segment(segment) => segment.get_point_under_pos(pick_pos, grab_handle_precision),
//             // QBezier(qbezier) => {
//             //     qbezier.get_shape_point_type_under_pick_pos(pick_pos, grab_handle_precision)
//             // }
//             // CBezier(cbezier) => {
//             //     cbezier.get_shape_point_type_under_pick_pos(pick_pos, grab_handle_precision)
//             // }
//             // ArcEllipse(aellipse) => {
//             //     aellipse.get_shape_point_type_under_pick_pos(pick_pos, grab_handle_precision)
//             // }
//         }
//     }
//     pub fn clear_selections(&mut self) {
//         use BasicShapeType::*;
//         match self {
//             Segment(segment) => segment.clear_selections(),
//             // QBezier(qbezier) => qbezier.s_dist(pos),
//             // CBezier(cbezier) => cbezier.s_dist(pos),
//             // ArcEllipse(aellipse) => aellipse.s_dist(pos),
//         }
//     }
//     pub fn select(&mut self, selection: bool) {
//         use BasicShapeType::*;
//         match self {
//             Segment(segment) => segment.set_selection(selection),
//             // QBezier(qbezier) => qbezier.s_dist(pos),
//             // CBezier(cbezier) => cbezier.s_dist(pos),
//             // ArcEllipse(aellipse) => aellipse.s_dist(pos),
//         }
//     }
//     pub fn select_point(&mut self, pt_typ: &PointType, selection: bool) {
//         use BasicShapeType::*;
//         match self {
//             Segment(segment) => segment.set_selection_point(pt_typ, selection),
//             // QBezier(qbezier) => qbezier.s_dist(pos),
//             // CBezier(cbezier) => cbezier.s_dist(pos),
//             // ArcEllipse(aellipse) => aellipse.s_dist(pos),
//         }
//     }
// }
