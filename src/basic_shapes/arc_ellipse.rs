use std::f64::consts::PI;
#[cfg(not(test))]
use web_sys::console;

use super::common::*;
use crate::math::*;
use crate::types::*;

#[derive(Clone)]
pub struct EllipticArc {
    pub center_point: Point,
    pub radius_point: Point,
    pub start_angle_point: Point,
    pub end_angle_point: Point,
    pub common: CommonVars,
}
impl EllipticArc {
    pub fn builder(
        center_pos: &WPos,
        radius_pos: &WPos,
        start_angle: f64,
        end_angle: f64,
    ) -> BasicShapeTypes {
        let mut center_pos = *center_pos;
        let position = center_pos;
        let mut radius_pos = *radius_pos - position;
        let center_pos = WPos::zero();

        let sa_pos = get_point_from_angle(&radius_pos, start_angle);
        let ea_pos = get_point_from_angle(&radius_pos, end_angle);

        let center_point = Point::new(&(center_pos), true, true, false);
        let radius_point = Point::new(&(radius_pos), true, true, false);
        let start_angle_point = Point::new(&(sa_pos), true, true, false);
        let end_angle_point = Point::new(&(ea_pos), true, true, false);

        let earc = EllipticArc {
            center_point,
            radius_point,
            start_angle_point,
            end_angle_point,
            common: CommonVars {
                position,
                saved_position: position,
                selected: false,
                // init: true,
            },
        };
        BasicShapeTypes::ArcEllipse(Box::new(earc))
    }
    pub fn deselect_all_points(&mut self) {
        self.center_point.selected = false;
        self.radius_point.selected = false;
        self.start_angle_point.selected = false;
        self.end_angle_point.selected = false;
    }
    pub fn set_selected(&mut self) {
        self.common.selected = false;
    }
    pub fn is_selected(&self) -> bool {
        self.common.selected == true
    }
    pub fn save_current_position(&mut self) {
        self.common.saved_position = self.common.position
    }
    pub fn s_dist(&self, pick_pos: &WPos) -> f64 {
        0.
    }
    pub fn get_shape_point_type_under_pick_pos(
        &self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<PointType> {
        let radius = self.radius_point.wpos;
        // The first point found is returned
        let pick_pos = *pick_pos - self.common.position;
        if pick_pos.dist(&self.center_point.wpos) < grab_handle_precision {
            return Some(PointType::Center);
        }
        if pick_pos.dist(&radius) < grab_handle_precision {
            return Some(PointType::Radius);
        }
        if pick_pos.dist(&self.start_angle_point.wpos) < grab_handle_precision {
            return Some(PointType::StartAngle);
        }
        if pick_pos.dist(&self.end_angle_point.wpos) < grab_handle_precision {
            return Some(PointType::EndAngle);
        }
        None
    }

    fn ellipse_line_intersection(&self, pt: &WPos) -> WPos {
        let center_pos = self.center_point.wpos;
        let mut radius_pos = self.radius_point.wpos;
        radius_pos.wy = -radius_pos.wy;

        let m = (pt.wy - center_pos.wy) / (pt.wx - center_pos.wx);
        let h = center_pos.wx;
        let k = center_pos.wy;
        let a = radius_pos.wx;
        let b = radius_pos.wy;
        // Calculating y-intercept using the center of the ellipse
        let c = k - m * h;
        let a_coef = m.powi(2) / b.powi(2) + 1.0 / a.powi(2);
        let b_coef = 2.0 * m * (c - k) / b.powi(2) - 2.0 * h / a.powi(2);
        let c_coef = h.powi(2) / a.powi(2) + (c - k).powi(2) / b.powi(2) - 1.0;
        let discriminant = b_coef.powi(2) - 4.0 * a_coef * c_coef;
        let x1 = (-b_coef + discriminant.sqrt()) / (2.0 * a_coef);
        let y1 = m * x1 + c;
        let pt1 = WPos::new(x1, y1);
        let x2 = (-b_coef - discriminant.sqrt()) / (2.0 * a_coef);
        let y2 = m * x2 + c;
        let pt2 = WPos::new(x2, y2);
        if pt.dist(&pt1) < pt.dist(&pt2) {
            pt1
        } else {
            pt2
        }
    }
    fn angle_on_ellipse(&self, pos: &WPos) -> f64 {
        let center_pos = self.center_point.wpos;
        let radius_pos = self.radius_point.wpos;
        f64::atan2(
            (pos.wy - center_pos.wy) / radius_pos.wy.abs(),
            (pos.wx - center_pos.wx) / radius_pos.wx.abs(),
        )
    }
    fn get_point_from_angle(&self, angle: f64) -> WPos {
        let x = self.radius_point.wpos.wx.abs() * angle.cos();
        let y = self.radius_point.wpos.wy.abs() * angle.sin();
        WPos { wx: x, wy: y }
    }
    fn is_point_on_ellipse(&self, pos: &WPos, precision: f64) -> bool {
        let pt_int = self.ellipse_line_intersection(&pos);
        if pt_int.dist(&pos) > precision {
            return false;
        }
        if self.start_angle_point.wpos.dist(&self.end_angle_point.wpos) < 1. {
            return true;
        }
        let start_angle = self.angle_on_ellipse(&self.start_angle_point.wpos);
        let end_angle = self.angle_on_ellipse(&self.end_angle_point.wpos);
        let angle = self.angle_on_ellipse(&pos);

        if end_angle > start_angle {
            angle >= start_angle && angle <= end_angle
        } else {
            !(angle >= end_angle && angle <= start_angle)
        }
    }
    pub fn get_angle_from_pos(&self, pos: &WPos) -> f64 {
        pos.wy.atan2(pos.wx)
    }
}
// impl BasicShape for EllipticArc {
//     fn is_init(&self) -> bool {
//         self.init
//     }
//     fn init_done(&mut self) {
//         self.init = false;
//     }
//     fn get_pos(&self) -> WPos {
//         self.position
//     }
//     fn get_step_r(&self, grab_handle_precision: f64) -> f64 {
//         //
//         0.
//     }

//     fn get_pos_from_ratio(&self, r: f64) -> WPos {
//         // TODO
//         WPos::zero()
//     }
//     fn get_ratio_from_pos(&self, rpos: &WPos) -> f64 {
//         // TODO
//         0.
//     }
//     fn get_projected_pos(&self, pick_pos: &WPos) -> WPos {
//         // TODO
//         WPos::zero()
//     }
//     fn split(&self, pos: &WPos) -> (Option<Box<dyn BasicShape>>, Option<Box<dyn BasicShape>>) {
//         // TODO
//         (None, None)
//     }

//     fn dist(&self, pick_pos: &WPos) -> f64 {
//         // TODO
//         0.
//     }
//     // fn is_shape_under_pick_pos(&self, pick_pos: &WPos, grab_handle_precision: f64) -> bool {
//     //     let pick_pos = *pick_pos - self.position;
//     //     self.is_point_on_ellipse(&pick_pos, grab_handle_precision / 2.)
//     // }

//     fn clear_selection(&mut self) {
//         self.selected = false
//     }
//     fn set_selected(&mut self, selected: bool) {
//         self.selected = selected;
//     }
//     fn deselect_all_points(&mut self) {
//         self.center_point.selected = false;
//         self.radius_point.selected = false;
//         self.sa_point.selected = false;
//         self.ea_point.selected = false;
//     }
//     fn is_selected(&self) -> bool {
//         self.selected
//     }
//     fn move_selection(&mut self, pick_pos: &WPos, pick_pos_ms_dwn: &WPos, magnet_distance: f64) {
//         let pick_pos = *pick_pos;

//         if self.init {
//             self.center_point.selected = false;
//             self.radius_point.selected = true;
//             self.sa_point.selected = false;
//             self.ea_point.selected = false;
//         }
//         if self.selected {
//             match (
//                 self.center_point.selected,
//                 self.radius_point.selected,
//                 self.sa_point.selected,
//                 self.ea_point.selected,
//             ) {
//                 (false, false, false, false) | (true, false, false, false) => {
//                     let mut pick_pos_ms_dwn = *pick_pos_ms_dwn;
//                     self.position = self.saved_position + pick_pos - pick_pos_ms_dwn;
//                 }
//                 (false, true, false, false) => {
//                     self.radius_point.wpos = pick_pos - self.position;
//                     let pos = pick_pos - self.position;
//                     if pos.wx != self.center_point.wpos.wx || pos.wy != self.center_point.wpos.wy {
//                         self.radius_point.wpos = pos;
//                     }

//                     let start_angle = self.angle_on_ellipse(&self.sa_point.wpos);
//                     self.sa_point.wpos = self.get_point_from_angle(start_angle);

//                     let end_angle = self.angle_on_ellipse(&self.ea_point.wpos);
//                     self.ea_point.wpos = self.get_point_from_angle(end_angle);
//                 }
//                 (false, false, true, false) => {
//                     let pos = pick_pos - self.position - self.center_point.wpos;
//                     let mut angle = self.get_angle_from_pos(&pos);

//                     magnet_to_45(&mut angle, &self.radius_point.wpos, magnet_distance);
//                     self.sa_point.wpos = get_point_from_angle(&self.radius_point.wpos, angle);

//                     magnet_to_x(
//                         &mut self.sa_point.wpos,
//                         &self.center_point.wpos,
//                         magnet_distance,
//                     );
//                     magnet_to_y(
//                         &mut self.sa_point.wpos,
//                         &self.center_point.wpos,
//                         magnet_distance,
//                     );
//                     magnet_to_xy(
//                         &mut self.sa_point.wpos,
//                         &self.center_point.wpos,
//                         magnet_distance,
//                     );
//                 }
//                 (false, false, false, true) => {
//                     let pos = pick_pos - self.position - self.center_point.wpos;
//                     let mut angle = self.get_angle_from_pos(&pos);

//                     magnet_to_45(&mut angle, &self.radius_point.wpos, magnet_distance);
//                     self.ea_point.wpos = get_point_from_angle(&self.radius_point.wpos, angle);

//                     magnet_to_x(
//                         &mut self.ea_point.wpos,
//                         &self.center_point.wpos,
//                         magnet_distance,
//                     );
//                     magnet_to_y(
//                         &mut self.ea_point.wpos,
//                         &self.center_point.wpos,
//                         magnet_distance,
//                     );
//                     magnet_to_xy(
//                         &mut self.ea_point.wpos,
//                         &self.center_point.wpos,
//                         magnet_distance,
//                     );
//                 }
//                 _ => (),
//             }
//         }
//     }
//     fn select_point_type(&mut self, point_type: &PointType) {
//         (
//             self.center_point.selected,
//             self.radius_point.selected,
//             self.sa_point.selected,
//             self.ea_point.selected,
//         ) = match point_type {
//             PointType::Center => (true, false, false, false),
//             PointType::Radius => (false, true, false, false),
//             PointType::StartAngle => (false, false, true, false),
//             PointType::EndAngle => (false, false, false, true),
//             _ => (false, false, false, false),
//         }
//     }

//     fn save_current_position(&mut self) {
//         self.saved_position = self.position;
//     }
//     fn get_saved_position(&self) -> WPos {
//         self.saved_position
//     }
//     fn magnet_to_point(&self, pick_pos: &mut WPos, magnet_distance: f64) {
//         let sa_pos = self.sa_point.wpos;
//         let ea_pos = self.ea_point.wpos;

//         if pick_pos.dist(&(sa_pos + self.position)) < magnet_distance {
//             *pick_pos = self.position + sa_pos;
//         }
//         if pick_pos.dist(&(ea_pos + self.position)) < magnet_distance {
//             *pick_pos = self.position + ea_pos;
//         }
//     }

//     fn get_construction(&self) -> Vec<ConstructionType> {
//         let mut cst: Vec<ConstructionType> = vec![];
//         if !self.selected {
//             cst.push(ConstructionType::Layer(LayerType::Worksheet));
//         } else {
//             cst.push(ConstructionType::Layer(LayerType::Selected));
//         }

//         let start_angle = self.angle_on_ellipse(&self.sa_point.wpos);
//         let end_angle = self.angle_on_ellipse(&self.ea_point.wpos);

//         let mut sa_point = self.sa_point;

//         // sa_point.wpos = match (
//         //     self.radius_point.wpos.wx < 0.,
//         //     self.radius_point.wpos.wy < 0.,
//         // ) {
//         //     (false, false) => sa_point.wpos,
//         //     (false, true) => WPos::new(sa_point.wpos.wx, -sa_point.wpos.wy),
//         //     (true, false) => WPos::new(-sa_point.wpos.wx, sa_point.wpos.wy),
//         //     (true, true) => WPos::new(-sa_point.wpos.wx, -sa_point.wpos.wy),
//         // };

//         cst.push(ConstructionType::Move(self.position + sa_point.wpos));
//         cst.push(ConstructionType::Ellipse(
//             self.position + self.center_point.wpos,
//             WPos::new(
//                 self.radius_point.wpos.wx.abs(),
//                 self.radius_point.wpos.wy.abs(),
//             ),
//             0.,
//             start_angle,
//             end_angle,
//             false,
//         ));
//         cst
//     }
//     fn get_handles_construction(&self, size_handle: f64) -> Vec<ConstructionType> {
//         let mut cst = Vec::new();

//         let mut center_point = self.center_point;
//         center_point.wpos += self.position;

//         let mut radius_point = self.radius_point;
//         radius_point.wpos += self.position;

//         let mut sa_point = self.sa_point;
//         let mut ea_point = self.ea_point;
//         (sa_point.wpos, ea_point.wpos) = (sa_point.wpos, ea_point.wpos);

//         // (sa_point.wpos, ea_point.wpos) = match (
//         //     self.radius_point.wpos.wx < 0.,
//         //     self.radius_point.wpos.wy < 0.,
//         // ) {
//         //     (false, false) => (sa_point.wpos, ea_point.wpos),
//         //     (false, true) => (
//         //         WPos::new(sa_point.wpos.wx, -sa_point.wpos.wy),
//         //         WPos::new(ea_point.wpos.wx, -ea_point.wpos.wy),
//         //     ),
//         //     (true, false) => (
//         //         WPos::new(-sa_point.wpos.wx, sa_point.wpos.wy),
//         //         WPos::new(-ea_point.wpos.wx, ea_point.wpos.wy),
//         //     ),
//         //     (true, true) => (
//         //         WPos::new(-sa_point.wpos.wx, -sa_point.wpos.wy),
//         //         WPos::new(-ea_point.wpos.wx, -ea_point.wpos.wy),
//         //     ),
//         // };
//         sa_point.wpos += self.position;
//         ea_point.wpos += self.position;

//         push_handle(&mut cst, &center_point, size_handle);
//         push_handle(&mut cst, &radius_point, size_handle);
//         push_handle(&mut cst, &sa_point, size_handle);
//         push_handle(&mut cst, &ea_point, size_handle);
//         cst
//     }
//     fn get_helpers_construction(&self) -> Vec<ConstructionType> {
//         let mut cst: Vec<ConstructionType> = vec![];
//         let position = self.position;
//         let center = self.center_point.wpos;
//         let radius = self.radius_point.wpos;
//         let sa = self.sa_point.wpos;
//         let ea = self.ea_point.wpos;
//         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
//         if is_aligned_45_or_135(&center, &radius) {
//             helper_45_135(&(position + center), &(position + radius), true, &mut cst);
//             helper_45_135(
//                 &(position + center),
//                 &(position + WPos::new(radius.wx, -radius.wy)),
//                 true,
//                 &mut cst,
//             );
//         }
//         if is_aligned_45_or_135(&center, &sa) {
//             helper_45_135(&(position + center), &(position + sa), true, &mut cst);
//         }
//         if is_aligned_vert(&center, &sa) {
//             helper_vertical(&(position + center), &(position + sa), true, &mut cst);
//         }
//         if is_aligned_hori(&center, &sa) {
//             helper_horizontal(&(position + center), &(position + sa), true, &mut cst);
//         }
//         if is_aligned_45_or_135(&center, &ea) {
//             helper_45_135(&(position + center), &(position + ea), true, &mut cst);
//         }
//         if is_aligned_vert(&center, &ea) {
//             helper_vertical(&(position + center), &(position + ea), true, &mut cst);
//         }
//         if is_aligned_hori(&center, &ea) {
//             helper_horizontal(&(position + center), &(position + ea), true, &mut cst);
//         }
//         cst
//     }
//     fn get_bounded_rectangle(&self) -> [WPos; 2] {
//         [
//             self.position + self.center_point.wpos - self.radius_point.wpos,
//             self.position + self.center_point.wpos + self.radius_point.wpos,
//         ]
//     }
// }
// // impl ShapePool for Ellipse {}
