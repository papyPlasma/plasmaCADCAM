use crate::math::*;
use crate::shapes::shapes::{HandleType, Shape, ShapeType};
use std::hash::{Hash, Hasher};
use std::ops::{Deref, DerefMut};

use std::{
    collections::HashMap,
    f64::consts::PI,
    sync::atomic::{AtomicUsize, Ordering},
};

static COUNTER_POINTS: AtomicUsize = AtomicUsize::new(0);
static COUNTER_SHAPES: AtomicUsize = AtomicUsize::new(0);

#[derive(Copy, Clone, Debug)]
pub struct PointId {
    pt_id: usize,
}
impl Deref for PointId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.pt_id
    }
}
impl DerefMut for PointId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pt_id
    }
}
impl Hash for PointId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pt_id.hash(state);
    }
}
impl PartialEq for PointId {
    fn eq(&self, other: &Self) -> bool {
        self.pt_id == other.pt_id
    }
}
impl Eq for PointId {}

#[derive(Copy, Clone, Debug)]
pub struct ShapeId {
    sh_id: usize,
}
impl Deref for ShapeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.sh_id
    }
}
impl DerefMut for ShapeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sh_id
    }
}
impl Hash for ShapeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.sh_id.hash(state);
    }
}
impl PartialEq for ShapeId {
    fn eq(&self, other: &Self) -> bool {
        self.sh_id == other.sh_id
    }
}
impl Eq for ShapeId {}

pub struct DataPool {
    pts_ids: HashMap<PointId, WPoint>,
    shs_ids: HashMap<ShapeId, Shape>,
}
impl DataPool {
    pub fn new() -> DataPool {
        DataPool {
            pts_ids: HashMap::new(),
            shs_ids: HashMap::new(),
        }
    }
    // Points
    pub fn insert_point(&mut self, pt: &WPoint) -> PointId {
        let pt_id = PointId {
            pt_id: COUNTER_POINTS.fetch_add(1, Ordering::Relaxed),
        };
        self.pts_ids.insert(pt_id.clone(), *pt);
        pt_id
    }
    pub fn modify_point(&mut self, pt_id: PointId, pt: &WPoint) {
        self.pts_ids.insert(pt_id.clone(), *pt);
    }
    pub fn delete_point(&mut self, pt_id: PointId) {
        self.pts_ids.remove(&pt_id);
    }
    pub fn get_point(&self, pt_id: PointId) -> Option<&WPoint> {
        self.pts_ids.get(&pt_id)
    }
    // Shapes
    pub fn is_point_on_line(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::Line = shape.get_type() {
            let pts = shape.get_handles_bundles();
            let start = self
                .get_point(*pts.get(&HandleType::Start).unwrap())
                .unwrap();
            let end = self.get_point(*pts.get(&HandleType::End).unwrap()).unwrap();
            point_on_segment(&start, &end, &pt, precision)
        } else {
            false
        }
    }
    pub fn is_point_on_quadbezier(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::QuadBezier = shape.get_type() {
            let mut t_min = 0.;
            let mut t_max = 1.;
            let mut min_dist = f64::MAX;
            let pts = shape.get_handles_bundles();
            let start = self
                .get_point(*pts.get(&HandleType::Start).unwrap())
                .unwrap();
            let ctrl = self
                .get_point(*pts.get(&HandleType::Ctrl).unwrap())
                .unwrap();
            let end = self.get_point(*pts.get(&HandleType::End).unwrap()).unwrap();
            for _i in 0..MAX_ITERATIONS {
                // max iterations can be adjusted
                let t_mid = (t_min + t_max) / 2.;
                let bt = get_point_on_quad_bezier(t_mid, &start, &ctrl, &end);
                let dist = bt.dist(pt);
                if dist < min_dist {
                    min_dist = dist;
                }
                if dist < precision {
                    return true; // We found a sufficiently close point
                }
                // Using gradient to decide the next tMid for the next iteration.
                let gradient =
                    (bt.wx - pt.wx) * (end.wx - start.wx) + (bt.wy - pt.wy) * (end.wy - start.wy);
                if gradient > 0. {
                    t_max = t_mid;
                } else {
                    t_min = t_mid;
                }
            }
            min_dist <= precision
        } else {
            false
        }
    }
    pub fn is_point_on_cubicbezier(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::CubicBezier = shape.get_type() {
            let mut t_min = 0.;
            let mut t_max = 1.;
            let mut min_dist = f64::MAX;
            let pts = shape.get_handles_bundles();
            let start = self
                .get_point(*pts.get(&HandleType::Start).unwrap())
                .unwrap();
            let ctrl1 = self
                .get_point(*pts.get(&HandleType::Ctrl1).unwrap())
                .unwrap();
            let ctrl2 = self
                .get_point(*pts.get(&HandleType::Ctrl2).unwrap())
                .unwrap();
            let end = self.get_point(*pts.get(&HandleType::End).unwrap()).unwrap();
            for _i in 0..MAX_ITERATIONS {
                let t_mid = (t_min + t_max) / 2.;
                let bt = get_point_on_cubic_bezier(t_mid, &start, &ctrl1, &ctrl2, &end);
                let dist = bt.dist(pt);
                if dist < min_dist {
                    min_dist = dist;
                }
                if dist < precision {
                    return true; // We found a sufficiently close point
                }
                // Using gradient to decide the next tMid for the next iteration.
                let gradient =
                    (bt.wx - pt.wx) * (end.wx - start.wx) + (bt.wy - pt.wy) * (end.wy - start.wy);
                if gradient > 0. {
                    t_max = t_mid;
                } else {
                    t_min = t_mid;
                }
            }
            min_dist <= precision
        } else {
            false
        }
    }
    pub fn is_point_on_rectangle(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::Rectangle = shape.get_type() {
            let pts = shape.get_handles_bundles();
            let bl = self.get_point(*pts.get(&HandleType::BL).unwrap()).unwrap();
            let tl = self.get_point(*pts.get(&HandleType::TL).unwrap()).unwrap();
            let tr = self.get_point(*pts.get(&HandleType::TR).unwrap()).unwrap();
            let br = self.get_point(*pts.get(&HandleType::BR).unwrap()).unwrap();
            return point_on_segment(&bl, &tl, &pt, precision)
                || point_on_segment(&tl, &tr, &pt, precision)
                || point_on_segment(&tr, &br, &pt, precision)
                || point_on_segment(&br, &bl, &pt, precision);
        } else {
            false
        }
    }
    fn ellipse_line_intersection(&self, center: &WPoint, radius: &WPoint, pt: &WPoint) -> WPoint {
        let m = (pt.wy - center.wy) / (pt.wx - center.wx);
        let h = center.wx;
        let k = center.wy;
        let a = radius.wx;
        let b = radius.wy;

        // Calculating y-intercept using the center of the ellipse
        let c = k - m * h;

        let a_coef = m.powi(2) / b.powi(2) + 1.0 / a.powi(2);
        let b_coef = 2.0 * m * (c - k) / b.powi(2) - 2.0 * h / a.powi(2);
        let c_coef = h.powi(2) / a.powi(2) + (c - k).powi(2) / b.powi(2) - 1.0;

        let discriminant = b_coef.powi(2) - 4.0 * a_coef * c_coef;

        let x1 = (-b_coef + discriminant.sqrt()) / (2.0 * a_coef);
        let y1 = m * x1 + c;
        let pt1 = WPoint::new(x1, y1);

        let x2 = (-b_coef - discriminant.sqrt()) / (2.0 * a_coef);
        let y2 = m * x2 + c;
        let pt2 = WPoint::new(x2, y2);

        if pt.dist(&pt1) < pt.dist(&pt2) {
            pt1
        } else {
            pt2
        }
    }
    fn ellipse_angle_intersection(&self, center: &WPoint, radius: &WPoint, angle: f64) -> WPoint {
        if angle == -PI / 2. {
            return WPoint {
                wx: 0.,
                wy: radius.wy,
            };
        }
        if angle == PI / 2. {
            return WPoint {
                wx: 0.,
                wy: -radius.wy,
            };
        }
        let m = angle.tan();
        let h = center.wx;
        let k = center.wy;
        let a = radius.wx;
        let b = radius.wy;

        // Calculating y-intercept using the center of the ellipse
        let c = k - m * h;

        let a_coef = m.powi(2) / b.powi(2) + 1.0 / a.powi(2);
        let b_coef = 2.0 * m * (c - k) / b.powi(2) - 2.0 * h / a.powi(2);
        let c_coef = h.powi(2) / a.powi(2) + (c - k).powi(2) / b.powi(2) - 1.0;

        let discriminant = b_coef.powi(2) - 4.0 * a_coef * c_coef;

        let x1 = (-b_coef + discriminant.sqrt()) / (2.0 * a_coef);
        let y1 = m * x1 + c;
        let pt1 = WPoint::new(x1, y1);

        let x2 = (-b_coef - discriminant.sqrt()) / (2.0 * a_coef);
        let y2 = m * x2 + c;
        let pt2 = WPoint::new(x2, y2);

        if angle >= 0. {
            if m >= 0. {
                pt1
            } else {
                pt2
            }
        } else {
            if m >= 0. {
                pt2
            } else {
                pt1
            }
        }
    }
    pub fn is_point_on_ellipse(&self, pt: &WPoint, shape: &Shape, precision: f64) -> bool {
        if let ShapeType::Ellipse = shape.get_type() {
            let pts = shape.get_handles_bundles();
            let center = self
                .get_point(*pts.get(&HandleType::Center).unwrap())
                .unwrap();
            let radius = self
                .get_point(*pts.get(&HandleType::Radius).unwrap())
                .unwrap();
            let h_start_angle = self
                .get_point(*pts.get(&HandleType::StartAngle).unwrap())
                .unwrap();
            let h_end_angle = self
                .get_point(*pts.get(&HandleType::EndAngle).unwrap())
                .unwrap();

            // True radius
            let radius_pt = *radius - *center;
            let pt_int = self.ellipse_line_intersection(center, &radius_pt, pt);
            if pt_int.dist(pt) > precision {
                return false;
            }

            if h_start_angle.dist(h_end_angle) < 1. {
                return true;
            }
            let start_angle = -center.angle_on_ellipse(&h_start_angle, &radius);
            let end_angle = -center.angle_on_ellipse(&h_end_angle, &radius);
            let angle = -center.angle_on_ellipse(&pt, &radius);

            if end_angle > start_angle {
                angle >= start_angle && angle <= end_angle
            } else {
                !(angle >= end_angle && angle <= start_angle)
            }
        } else {
            false
        }
    }
    pub fn move_line_point(
        &mut self,
        shape_id: ShapeId,
        point_id: PointId,
        pick_point: &WPoint,
        snap_distance: f64,
    ) {
        let shape = self.get_shape(&shape_id).unwrap();
        self.modify_point(point_id, &(*pick_point - shape.coord));
    }
    pub fn move_quadbezier_point(
        &mut self,
        shape_id: ShapeId,
        point_id: PointId,
        pick_point: &WPoint,
        _snap_distance: f64,
    ) {
        let shape = self.get_shape(&shape_id).cloned().unwrap();
        let init = shape.is_init();
        let handles_bdls = shape.get_handles_bundles().clone();
        self.modify_point(point_id, &(*pick_point - shape.coord));
        if init {
            let start_pt = handles_bdls
                .iter()
                .find(|&h_bdl| *h_bdl.0 == HandleType::Start)
                .unwrap()
                .1;
            let ctrl_pt = handles_bdls
                .iter()
                .find(|&h_bdl| *h_bdl.0 == HandleType::Ctrl)
                .unwrap()
                .1;
            // If on init, then the selected point is the end point
            // Set the control point to half the distance start-end
            let start = self.get_point(*start_pt).unwrap();
            let ctrl_pos = ((*pick_point - shape.coord) + *start) / 2.;
            self.modify_point(*ctrl_pt, &ctrl_pos);
        }
    }
    pub fn move_cubicbezier_point(
        &mut self,
        shape_id: ShapeId,
        point_id: PointId,
        pick_point: &WPoint,
        _snap_distance: f64,
    ) {
        let shape = self.get_shape(&shape_id).cloned().unwrap();
        let init = shape.is_init();
        // snap_to_snap_grid(&mut mouse_pos_world, snap_distance);
        let handles_bdls = shape.get_handles_bundles().clone();
        self.modify_point(point_id, &(*pick_point - shape.coord));
        if init {
            let start_pt = handles_bdls
                .iter()
                .find(|&h_bdl| *h_bdl.0 == HandleType::Start)
                .unwrap()
                .1;
            let ctrl1_pt = handles_bdls
                .iter()
                .find(|&h_bdl| *h_bdl.0 == HandleType::Ctrl1)
                .unwrap()
                .1;
            let ctrl2_pt = handles_bdls
                .iter()
                .find(|&h_bdl| *h_bdl.0 == HandleType::Ctrl2)
                .unwrap()
                .1;
            // If on init, then the selected point is the end point
            // Set the ctrl 1 and ctrl 2 points to 1/3 2/3 the distance start-end
            let start = self.get_point(*start_pt).unwrap();
            let ctrl1_pos = ((*pick_point - shape.coord) + *start) / 3.;
            let ctrl2_pos = ((*pick_point - shape.coord) + *start) / 3. * 2.;
            self.modify_point(*ctrl1_pt, &ctrl1_pos);
            self.modify_point(*ctrl2_pt, &ctrl2_pos);
        }
    }
    pub fn move_rectangle_point(
        &mut self,
        shape_id: ShapeId,
        point_id: PointId,
        pick_point: &WPoint,
        snap_distance: f64,
    ) {
        let shape = self.get_shape(&shape_id).cloned().unwrap();
        let pick_point = *pick_point - *shape.get_coord();
        let pts_ids = shape.get_handles_bundles();
        let bl_id = pts_ids.get(&HandleType::BL).unwrap();
        let tl_id = pts_ids.get(&HandleType::TL).unwrap();
        let tr_id = pts_ids.get(&HandleType::TR).unwrap();
        let br_id = pts_ids.get(&HandleType::BR).unwrap();
        let mut bl = self.get_point(*bl_id).cloned().unwrap();
        let mut tl = self.get_point(*tl_id).cloned().unwrap();
        let mut tr = self.get_point(*tr_id).cloned().unwrap();
        let mut br = self.get_point(*br_id).cloned().unwrap();
        if point_id == *bl_id {
            bl = pick_point;
            if bl.wx >= tr.wx {
                bl.wx = tr.wx - snap_distance;
            }
            if bl.wy <= tr.wy {
                bl.wy = tr.wy + snap_distance;
            }
            tl.wx = bl.wx;
            br.wy = bl.wy;
        }
        if point_id == *tl_id {
            tl = pick_point;
            if tl.wx >= br.wx {
                tl.wx = br.wx - snap_distance;
            }
            if tl.wy >= br.wy {
                tl.wy = br.wy - snap_distance;
            }
            tr.wy = tl.wy;
            bl.wx = tl.wx;
        }
        if point_id == *tr_id {
            tr = pick_point;
            if tr.wx <= bl.wx {
                tr.wx = bl.wx + snap_distance;
            }
            if tr.wy >= bl.wy {
                tr.wy = bl.wy - snap_distance;
            }
            tl.wy = tr.wy;
            br.wx = tr.wx;
        }
        if point_id == *br_id {
            br = pick_point;
            if br.wx <= tl.wx {
                br.wx = tl.wx + snap_distance;
            }
            if br.wy <= tl.wy {
                br.wy = tl.wy + snap_distance;
            }
            tr.wx = br.wx;
            bl.wy = br.wy;
        }
        self.modify_point(*bl_id, &bl);
        self.modify_point(*tl_id, &tl);
        self.modify_point(*tr_id, &tr);
        self.modify_point(*br_id, &br);
    }
    pub fn move_ellipse_point(
        &mut self,
        shape_id: ShapeId,
        point_id: PointId,
        pick_point: &WPoint,
        snap_distance: f64,
    ) {
        let shape = self.get_shape(&shape_id).unwrap();
        let pick_point = *pick_point - *shape.get_coord();
        let pts_ids = shape.get_handles_bundles().clone();

        let center_id = pts_ids.get(&HandleType::Center).unwrap();
        let radius_id = pts_ids.get(&HandleType::Radius).unwrap();
        let h_sa_id = pts_ids.get(&HandleType::StartAngle).unwrap();
        let h_ea_id = pts_ids.get(&HandleType::EndAngle).unwrap();

        let center = self.get_point(*center_id).cloned().unwrap();
        let mut radius = self.get_point(*radius_id).cloned().unwrap();
        let mut h_sa = self.get_point(*h_sa_id).cloned().unwrap();
        let mut h_ea = self.get_point(*h_ea_id).cloned().unwrap();

        let start_angle = get_atan2(&(h_sa - center));
        let end_angle = get_atan2(&(h_ea - center));

        let coord = self.get_shape_mut(&shape_id).unwrap().get_coord_mut();
        if point_id == *center_id {
            *coord = *coord + pick_point;
        }

        if point_id == *radius_id {
            radius = pick_point;
            if radius.wx <= center.wx {
                radius.wx = center.wx + snap_distance;
            }
            if radius.wy >= center.wy {
                radius.wy = center.wy - snap_distance;
            }
            h_sa = self.ellipse_angle_intersection(&center, &radius, start_angle);
            h_ea = self.ellipse_angle_intersection(&center, &radius, end_angle);
        }
        if point_id == *h_sa_id {
            let angle = get_atan2(&(pick_point - center));
            h_sa = get_point_from_angle(&radius, angle);
        }
        if point_id == *h_ea_id {
            let angle = get_atan2(&(pick_point - center));
            h_ea = get_point_from_angle(&radius, angle);
        }

        self.modify_point(*center_id, &center);
        self.modify_point(*radius_id, &radius);
        self.modify_point(*h_sa_id, &h_sa);
        self.modify_point(*h_ea_id, &h_ea);
    }
    pub fn get_shape_id_from_mouse_pos_world(
        &self,
        pos: &WPoint,
        grab_handle_precision: f64,
    ) -> Option<ShapeId> {
        for (shape_id, shape) in self.shs_ids.iter() {
            use ShapeType::*;
            let coord = shape.get_coord();
            match shape.get_type() {
                Line => {
                    if self.is_point_on_line(&(*pos - *coord), shape, grab_handle_precision) {
                        return Some(*shape_id);
                    }
                }
                QuadBezier => {
                    if self.is_point_on_quadbezier(&(*pos - *coord), shape, grab_handle_precision) {
                        return Some(*shape_id);
                    }
                }
                CubicBezier => {
                    if self.is_point_on_cubicbezier(&(*pos - *coord), shape, grab_handle_precision)
                    {
                        return Some(*shape_id);
                    }
                }
                Rectangle => {
                    if self.is_point_on_rectangle(&(*pos - *coord), shape, grab_handle_precision) {
                        return Some(*shape_id);
                    }
                }
                Ellipse => {
                    if self.is_point_on_ellipse(&(*pos - *coord), shape, grab_handle_precision) {
                        return Some(*shape_id);
                    }
                }
                Group => (),
            }
        }
        None
    }
    pub fn get_point_id_from_position(
        &self,
        pos: &WPoint,
        grab_handle_precision: f64,
    ) -> Option<PointId> {
        for shape in self.shs_ids.values() {
            let coord = shape.get_coord();
            for (_, pt_id) in shape.get_handles_bundles().iter() {
                let pt = self.get_point(*pt_id).unwrap();
                if is_point_on_point(pos, &(*pt + *coord), grab_handle_precision) {
                    return Some(pt_id.clone());
                }
            }
        }
        None
    }
    pub fn insert_shape(&mut self, shape: Shape) -> ShapeId {
        let shape_id = ShapeId {
            sh_id: COUNTER_SHAPES.fetch_add(1, Ordering::Relaxed),
        };
        self.shs_ids.insert(shape_id, shape);
        shape_id
    }
    pub fn modify_shape(&mut self, shape_id: &ShapeId, shape: Shape) {
        self.shs_ids.insert(*shape_id, shape);
    }
    pub fn delete_shape(&mut self, shape_id: &ShapeId) {
        self.shs_ids.remove(shape_id);
    }
    pub fn move_shape(&mut self, shape_selected_id: ShapeId, pos_point: &WPoint) {
        let shape = self.get_shape_mut(&shape_selected_id).unwrap();
        shape.move_shape(pos_point);
    }
    pub fn move_shape_point(
        &mut self,
        point_id: PointId,
        shape_id: ShapeId,
        pick_point: &WPoint,
        snap_distance: f64,
    ) {
        use ShapeType::*;
        let shape = self.get_shape(&shape_id).cloned().unwrap();
        match shape.shape_type {
            Line => self.move_line_point(shape_id, point_id, pick_point, snap_distance),
            QuadBezier => self.move_quadbezier_point(shape_id, point_id, pick_point, snap_distance),
            CubicBezier => {
                self.move_cubicbezier_point(shape_id, point_id, pick_point, snap_distance)
            }
            Rectangle => self.move_rectangle_point(shape_id, point_id, pick_point, snap_distance),
            Ellipse => self.move_ellipse_point(shape_id, point_id, pick_point, snap_distance),
            ShapeType::Group => (),
        }
    }
    pub fn select_shapes_bounded_by_rectangle(&mut self, _bb: [WPoint; 2]) {
        // for shape in &mut pa_ref.pool.get_pool_shapes().values() {
        //     let bb_inner: [WPoint; 2] = shape.get_bounding_box();
        //     if is_box_inside(&bb_outer, &bb_inner) {
        //         shape.clear_any_shape_selection(Some(usize::All));
        //     } else {
        //         shape.clear_any_shape_selection(None);
        //     }
        // }
    }
    pub fn get_all_shapes_ids(&self) -> Vec<ShapeId> {
        self.shs_ids.keys().cloned().collect()
    }
    pub fn get_shape(&self, shape_id: &ShapeId) -> Option<&Shape> {
        self.shs_ids.get(shape_id)
    }
    pub fn shape_has_point(&self, shape_id: &ShapeId, point_id: PointId) -> bool {
        let shape = self.shs_ids.get(shape_id).unwrap();
        for sh_point_id in shape.handles_bundles.values() {
            if *sh_point_id == point_id {
                return true;
            }
        }
        false
    }
    pub fn get_shape_id_from_point_id(&self, point_id: PointId) -> ShapeId {
        for shape_id in self.shs_ids.keys() {
            if self.shape_has_point(shape_id, point_id) {
                return *shape_id;
            }
        }
        unreachable!()
    }
    pub fn get_shape_mut(&mut self, shape_id: &ShapeId) -> Option<&mut Shape> {
        self.shs_ids.get_mut(&shape_id)
    }
    pub fn get_shapes(&self) -> &HashMap<ShapeId, Shape> {
        &self.shs_ids
    }
    pub fn get_shapes_mut(&mut self) -> &mut HashMap<ShapeId, Shape> {
        &mut self.shs_ids
    }
}
