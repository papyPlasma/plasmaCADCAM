use super::types::{ConstructionType, LayerType, Point, PointType, Shape, WPos};
use crate::{datapool::ShapePool, math::*};

#[derive(Clone)]
pub struct Ellipse {
    center_point: Point,
    radius_point: Point,
    sa_point: Point,
    ea_point: Point,
    position: WPos,
    saved_position: WPos,
    selected: bool,
    init: bool,
}
impl Ellipse {
    pub fn new(
        center_pos: &WPos,
        radius_pos: &WPos,
        start_angle: f64,
        end_angle: f64,
        snap_distance: f64,
    ) -> Ellipse {
        let position = *center_pos;
        let mut center_pos = WPos::zero();
        let mut radius_pos = *radius_pos - position;

        center_pos = snap_to_snap_grid(&center_pos, snap_distance);
        radius_pos = snap_to_snap_grid(&radius_pos, snap_distance);

        if radius_pos.wx == center_pos.wx {
            radius_pos.wx += snap_distance;
        }
        if radius_pos.wy == center_pos.wy {
            radius_pos.wy += snap_distance;
        }

        let sa_pos = get_point_from_angle(&radius_pos, start_angle);
        let ea_pos = get_point_from_angle(&radius_pos, end_angle);

        let center_point = Point::new(&(center_pos), true, true, false);
        let radius_point = Point::new(&(radius_pos), true, true, false);
        let sa_point = Point::new(&(sa_pos), true, true, false);
        let ea_point = Point::new(&(ea_pos), true, true, false);

        Ellipse {
            center_point,
            radius_point,
            sa_point,
            ea_point,
            position,
            saved_position: position,
            selected: false,
            init: true,
        }
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
            (pos.wy - center_pos.wy) / radius_pos.wy,
            (pos.wx - center_pos.wx) / radius_pos.wx,
        )
    }
    fn angle_on_ellipse_abs(&self, pos: &WPos) -> f64 {
        let center_pos = self.center_point.wpos;
        let radius_pos = self.radius_point.wpos.abs();
        f64::atan2(
            (pos.wy - center_pos.wy) / radius_pos.wy,
            (pos.wx - center_pos.wx) / radius_pos.wx,
        )
    }
    fn get_point_from_angle(&self, angle: f64) -> WPos {
        let x = self.radius_point.wpos.wx * angle.cos();
        let y = self.radius_point.wpos.wy * angle.sin();
        WPos { wx: x, wy: y }
    }
    fn is_point_on_ellipse(&self, pos: &WPos, precision: f64) -> bool {
        let pt_int = self.ellipse_line_intersection(&pos);
        if pt_int.dist(&pos) > precision {
            return false;
        }
        if self.sa_point.wpos.dist(&self.ea_point.wpos) < 1. {
            return true;
        }
        let start_angle = self.angle_on_ellipse(&self.sa_point.wpos);
        let end_angle = self.angle_on_ellipse(&self.ea_point.wpos);
        let angle = self.angle_on_ellipse_abs(&pos);

        if end_angle > start_angle {
            angle >= start_angle && angle <= end_angle
        } else {
            !(angle >= end_angle && angle <= start_angle)
        }
    }
}
impl Shape for Ellipse {
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
        self.is_point_on_ellipse(&pick_pos, grab_handle_precision / 2.)
    }
    fn get_shape_point_under_pick_pos(
        &mut self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<PointType> {
        let radius = self.radius_point.wpos;
        // The first point found is returned
        let pick_pos = *pick_pos - self.position;
        if is_point_on_point(&pick_pos, &self.center_point.wpos, grab_handle_precision) {
            return Some(PointType::Center);
        }
        if is_point_on_point(&pick_pos, &radius, grab_handle_precision) {
            return Some(PointType::Radius);
        }
        let (sa_wpos, ea_wpos) = match (
            self.radius_point.wpos.wx < 0.,
            self.radius_point.wpos.wy < 0.,
        ) {
            (false, false) => (self.sa_point.wpos, self.ea_point.wpos),
            (false, true) => (
                WPos::new(self.sa_point.wpos.wx, -self.sa_point.wpos.wy),
                WPos::new(self.ea_point.wpos.wx, -self.ea_point.wpos.wy),
            ),
            (true, false) => (
                WPos::new(-self.sa_point.wpos.wx, self.sa_point.wpos.wy),
                WPos::new(-self.ea_point.wpos.wx, self.ea_point.wpos.wy),
            ),
            (true, true) => (
                WPos::new(-self.sa_point.wpos.wx, -self.sa_point.wpos.wy),
                WPos::new(-self.ea_point.wpos.wx, -self.ea_point.wpos.wy),
            ),
        };
        if is_point_on_point(&pick_pos, &sa_wpos, grab_handle_precision) {
            return Some(PointType::StartAngle);
        }
        if is_point_on_point(&pick_pos, &ea_wpos, grab_handle_precision) {
            return Some(PointType::EndAngle);
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
        self.center_point.selected = false;
        self.radius_point.selected = false;
        self.sa_point.selected = false;
        self.ea_point.selected = false;
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
        let rel_pick_pos = snap_to_snap_grid(&(*pick_pos - self.position), snap_distance);

        if self.init {
            self.center_point.selected = false;
            self.radius_point.selected = true;
            self.sa_point.selected = false;
            self.ea_point.selected = false;
        }
        if self.selected {
            match (
                self.center_point.selected,
                self.radius_point.selected,
                self.sa_point.selected,
                self.ea_point.selected,
            ) {
                (true, false, false, false) => {
                    self.position = self.saved_position + *pick_pos - *pick_pos_ms_dwn;
                }
                (false, true, false, false) => {
                    self.radius_point.wpos = rel_pick_pos;
                    if self.radius_point.wpos.wx == self.center_point.wpos.wx {
                        self.radius_point.wpos.wx += snap_distance;
                    }
                    if self.radius_point.wpos.wy == self.center_point.wpos.wy {
                        self.radius_point.wpos.wy += snap_distance;
                    }

                    let start_angle = self.angle_on_ellipse(&self.sa_point.wpos);
                    self.sa_point.wpos = self.get_point_from_angle(start_angle);

                    let end_angle = self.angle_on_ellipse(&self.ea_point.wpos);
                    self.ea_point.wpos = self.get_point_from_angle(end_angle);
                }
                (false, false, true, false) => {
                    let pos = rel_pick_pos - self.center_point.wpos;
                    let angle = match (
                        self.radius_point.wpos.wx < 0.,
                        self.radius_point.wpos.wy < 0.,
                    ) {
                        (false, false) => get_atan2(&pos),
                        (false, true) => get_atan2(&WPos::new(pos.wx, -pos.wy)),
                        (true, false) => get_atan2(&WPos::new(-pos.wx, pos.wy)),
                        (true, true) => get_atan2(&WPos::new(-pos.wx, -pos.wy)),
                    };
                    self.sa_point.wpos = get_point_from_angle(&self.radius_point.wpos, angle);
                }
                (false, false, false, true) => {
                    let pos = rel_pick_pos - self.center_point.wpos;
                    let angle = match (
                        self.radius_point.wpos.wx < 0.,
                        self.radius_point.wpos.wy < 0.,
                    ) {
                        (false, false) => get_atan2(&pos),
                        (false, true) => get_atan2(&WPos::new(pos.wx, -pos.wy)),
                        (true, false) => get_atan2(&WPos::new(-pos.wx, pos.wy)),
                        (true, true) => get_atan2(&WPos::new(-pos.wx, -pos.wy)),
                    };
                    self.ea_point.wpos = get_point_from_angle(&self.radius_point.wpos, angle);
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
            self.center_point.selected,
            self.radius_point.selected,
            self.sa_point.selected,
            self.ea_point.selected,
        ) = match point_type {
            PointType::Center => (true, false, false, false),
            PointType::Radius => (false, true, false, false),
            PointType::StartAngle => (false, false, true, false),
            PointType::EndAngle => (false, false, false, true),
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
        let sa_pos = self.sa_point.wpos;
        let ea_pos = self.ea_point.wpos;

        if pick_pos.dist(&(sa_pos + self.position)) < magnet_distance {
            *pick_pos = self.position + sa_pos;
        }
        if pick_pos.dist(&(ea_pos + self.position)) < magnet_distance {
            *pick_pos = self.position + ea_pos;
        }
    }

    fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        if !self.selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }

        let start_angle = self.angle_on_ellipse(&self.sa_point.wpos);
        let end_angle = self.angle_on_ellipse(&self.ea_point.wpos);

        let mut sa_point = self.sa_point;

        sa_point.wpos = match (
            self.radius_point.wpos.wx < 0.,
            self.radius_point.wpos.wy < 0.,
        ) {
            (false, false) => sa_point.wpos,
            (false, true) => WPos::new(sa_point.wpos.wx, -sa_point.wpos.wy),
            (true, false) => WPos::new(-sa_point.wpos.wx, sa_point.wpos.wy),
            (true, true) => WPos::new(-sa_point.wpos.wx, -sa_point.wpos.wy),
        };

        cst.push(ConstructionType::Move(self.position + sa_point.wpos));
        cst.push(ConstructionType::Ellipse(
            self.position + self.center_point.wpos,
            WPos::new(
                self.radius_point.wpos.wx.abs(),
                self.radius_point.wpos.wy.abs(),
            ),
            0.,
            start_angle,
            end_angle,
            false,
        ));
        cst
    }
    fn get_handles_construction(&self, size_handle: f64) -> Vec<ConstructionType> {
        let mut cst = Vec::new();

        let mut center_point = self.center_point;
        center_point.wpos += self.position;

        let mut radius_point = self.radius_point;
        radius_point.wpos += self.position;

        let mut sa_point = self.sa_point;
        let mut ea_point = self.ea_point;
        (sa_point.wpos, ea_point.wpos) = match (
            self.radius_point.wpos.wx < 0.,
            self.radius_point.wpos.wy < 0.,
        ) {
            (false, false) => (sa_point.wpos, ea_point.wpos),
            (false, true) => (
                WPos::new(sa_point.wpos.wx, -sa_point.wpos.wy),
                WPos::new(ea_point.wpos.wx, -ea_point.wpos.wy),
            ),
            (true, false) => (
                WPos::new(-sa_point.wpos.wx, sa_point.wpos.wy),
                WPos::new(-ea_point.wpos.wx, ea_point.wpos.wy),
            ),
            (true, true) => (
                WPos::new(-sa_point.wpos.wx, -sa_point.wpos.wy),
                WPos::new(-ea_point.wpos.wx, -ea_point.wpos.wy),
            ),
        };
        sa_point.wpos += self.position;
        ea_point.wpos += self.position;

        push_handle(&mut cst, &center_point, size_handle);
        push_handle(&mut cst, &radius_point, size_handle);
        push_handle(&mut cst, &sa_point, size_handle);
        push_handle(&mut cst, &ea_point, size_handle);
        cst
    }
    fn get_helpers_construction(&self) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        let position = self.position;
        let center = self.center_point.wpos;
        let radius = self.radius_point.wpos;
        let sa = self.sa_point.wpos;
        let ea = self.ea_point.wpos;
        cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
        if is_aligned_45_or_135(&center, &radius) {
            helper_45_135(&(position + center), &(position + radius), true, &mut cst);
            helper_45_135(
                &(position + center),
                &(position + WPos::new(radius.wx, -radius.wy)),
                true,
                &mut cst,
            );
        }
        if is_aligned_45_or_135(&center, &sa) {
            helper_45_135(&(position + center), &(position + sa), true, &mut cst);
        }
        if is_aligned_vert(&center, &sa) {
            helper_vertical(&(position + center), &(position + sa), true, &mut cst);
        }
        if is_aligned_hori(&center, &sa) {
            helper_horizontal(&(position + center), &(position + sa), true, &mut cst);
        }
        if is_aligned_45_or_135(&center, &ea) {
            helper_45_135(&(position + center), &(position + ea), true, &mut cst);
        }
        if is_aligned_vert(&center, &ea) {
            helper_vertical(&(position + center), &(position + ea), true, &mut cst);
        }
        if is_aligned_hori(&center, &ea) {
            helper_horizontal(&(position + center), &(position + ea), true, &mut cst);
        }
        cst
    }
    fn get_bounded_rectangle(&self) -> [WPos; 2] {
        [
            self.position + self.center_point.wpos - self.radius_point.wpos,
            self.position + self.center_point.wpos + self.radius_point.wpos,
        ]
    }
}
impl ShapePool for Ellipse {}
