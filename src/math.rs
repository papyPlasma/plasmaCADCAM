use crate::{
    datapool::PointId,
    shapes::shapes::{ConstructionType, WPoint},
};
use std::{
    f64::consts::PI,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

pub const EPSILON: f64 = 1e-5; // Some small value
pub const MAX_ITERATIONS: usize = 100; // Or some other reasonable upper bound

pub fn is_aligned_vert(pt1: &WPoint, pt2: &WPoint) -> bool {
    (pt1.wx - pt2.wx).abs() < 0.001
}
pub fn helper_vertical(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    use ConstructionType::*;
    if full {
        cst.push(Move(*pt1));
        cst.push(Line(WPoint {
            wx: pt1.wx,
            wy: 2. * pt1.wy - pt2.wy,
        }));
        cst.push(Move(*pt1));
        cst.push(Line(*pt2));
        cst.push(Line(WPoint {
            wx: pt1.wx,
            wy: 2. * pt2.wy - pt1.wy,
        }));
    } else {
        cst.push(Move(*pt1));
        cst.push(Line(WPoint {
            wx: pt1.wx,
            wy: 2. * pt1.wy - pt2.wy,
        }));
        cst.push(Move(*pt2));
        cst.push(Line(WPoint {
            wx: pt1.wx,
            wy: 2. * pt2.wy - pt1.wy,
        }));
    }
}
pub fn is_aligned_hori(pt1: &WPoint, pt2: &WPoint) -> bool {
    (pt1.wy - pt2.wy).abs() < 0.001
}
pub fn helper_horizontal(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    use ConstructionType::*;
    if full {
        cst.push(Move(*pt1));
        cst.push(Line(WPoint {
            wx: 2. * pt1.wx - pt2.wx,
            wy: pt1.wy,
        }));
        cst.push(Move(*pt1));
        cst.push(Line(*pt2));
        cst.push(Line(WPoint {
            wx: 2. * pt2.wx - pt1.wx,
            wy: pt1.wy,
        }));
    } else {
        cst.push(Move(*pt1));
        cst.push(Line(WPoint {
            wx: 2. * pt1.wx - pt2.wx,
            wy: pt1.wy,
        }));
        cst.push(Move(*pt2));
        cst.push(Line(WPoint {
            wx: 2. * pt2.wx - pt1.wx,
            wy: pt1.wy,
        }));
    }
}
pub fn is_aligned_45_or_135(pt1: &WPoint, pt2: &WPoint) -> bool {
    let dy = pt2.wy - pt1.wy;
    let dx = pt2.wx - pt1.wx;
    if dx != 0. {
        (dy / dx).abs() > 1. / 1.01 && (dy / dx).abs() < 1.01
    } else {
        false
    }
}
pub fn helper_45_135(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    if full {
        use ConstructionType::*;
        cst.push(Move(*pt1));
        cst.push(Line(WPoint {
            wx: 2. * pt1.wx - pt2.wx,
            wy: 2. * pt1.wy - pt2.wy,
        }));
        cst.push(Move(*pt1));
        cst.push(Line(*pt2));
        cst.push(Line(WPoint {
            wx: 2. * pt2.wx - pt1.wx,
            wy: 2. * pt2.wy - pt1.wy,
        }));
    } else {
        use ConstructionType::*;
        cst.push(Move(*pt1));
        cst.push(Line(WPoint {
            wx: 2. * pt1.wx - pt2.wx,
            wy: 2. * pt1.wy - pt2.wy,
        }));
        cst.push(Move(*pt2));
        cst.push(Line(WPoint {
            wx: 2. * pt2.wx - pt1.wx,
            wy: 2. * pt2.wy - pt1.wy,
        }));
    }
}

pub fn is_point_on_point(pt: &WPoint, pt1: &WPoint, precision: f64) -> bool {
    pt.dist(pt1) < precision
}
fn is_between(pt: &WPoint, pt1: &WPoint, pt2: &WPoint) -> bool {
    let dot_product = (pt.wx - pt1.wx) * (pt2.wx - pt1.wx) + (pt.wy - pt1.wy) * (pt2.wy - pt1.wy);
    if dot_product < 0. {
        return false;
    }
    let length2 = (pt2.wx - pt1.wx).powf(2.) + (pt2.wy - pt1.wy).powf(2.);
    if dot_product > length2 {
        return false;
    }
    return true;
}
pub fn point_on_segment(pt1: &WPoint, pt2: &WPoint, pt: &WPoint, precision: f64) -> bool {
    let denominator = ((pt2.wy - pt1.wy).powf(2.) + (pt2.wx - pt1.wx).powf(2.)).sqrt();
    if denominator == 0. {
        return is_point_on_point(pt, &pt1, precision);
    }
    let numerator = ((pt2.wy - pt1.wy) * pt.wx - (pt2.wx - pt1.wx) * pt.wy + pt2.wx * pt1.wy
        - pt2.wy * pt1.wx)
        .abs();

    if numerator / denominator > precision {
        return false;
    }
    is_between(pt, &pt1, &pt2)
}

pub fn _is_box_inside(box_outer: &[WPoint; 2], box_inner: &[WPoint; 2]) -> bool {
    let bl_outer = box_outer[0];
    let tr_outer = box_outer[1];
    let bl_inner = box_inner[0];
    let tr_inner = box_inner[1];
    bl_inner.wx >= bl_outer.wx
        && bl_inner.wy >= bl_outer.wy
        && tr_inner.wx <= tr_outer.wx
        && tr_inner.wy <= tr_outer.wy
}
pub fn reorder_corners(bb: &mut [WPoint; 2]) {
    let pt1 = bb[0];
    let pt2 = bb[1];
    if pt1.wx < pt2.wx {
        if pt1.wy < pt2.wy {
            let bl = WPoint {
                wx: pt1.wx,
                wy: pt1.wy,
            };
            let tr = WPoint {
                wx: pt2.wx,
                wy: pt2.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        } else {
            let bl = WPoint {
                wx: pt1.wx,
                wy: pt2.wy,
            };
            let tr = WPoint {
                wx: pt2.wx,
                wy: pt1.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        }
    } else {
        if pt1.wy < pt2.wy {
            let bl = WPoint {
                wx: pt2.wx,
                wy: pt1.wy,
            };
            let tr = WPoint {
                wx: pt1.wx,
                wy: pt2.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        } else {
            let bl = WPoint {
                wx: pt2.wx,
                wy: pt2.wy,
            };
            let tr = WPoint {
                wx: pt1.wx,
                wy: pt1.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        }
    }
}

pub fn snap_to_snap_grid(pos: &mut WPoint, snap_distance: f64) {
    *pos = (*pos / snap_distance).round() * snap_distance;
}
pub fn _snap_to_snap_grid_y(pos: &mut WPoint, grid_spacing: f64) {
    pos.wy = (pos.wy / grid_spacing).round() * grid_spacing;
}
pub fn _snap_to_snap_grid_x(pos: &mut WPoint, grid_spacing: f64) {
    pos.wx = (pos.wx / grid_spacing).round() * grid_spacing;
}

pub fn push_handles(
    cst: &mut Vec<ConstructionType>,
    hdles: &Vec<(PointId, WPoint)>,
    opt_sel_id: &Option<PointId>,
    size_handle: f64,
) {
    for (pt_id, point) in hdles.iter() {
        let fill = matched_point(pt_id, opt_sel_id);
        push_handle(cst, &point, fill, size_handle);
    }
}
pub fn push_handle(cst: &mut Vec<ConstructionType>, pt: &WPoint, fill: bool, size_handle: f64) {
    let radius = WPoint::default() + size_handle / 2.;
    use ConstructionType::*;
    cst.push(Move(*pt + WPoint::default().addxy(size_handle / 2., 0.)));
    cst.push(Ellipse(*pt, radius, 0., 0., 2. * PI, fill));
}

fn matched_point(pt_id: &PointId, opt_sel_id: &Option<PointId>) -> bool {
    if let Some(pt_sel_id) = opt_sel_id {
        if *pt_sel_id == *pt_id {
            true
        } else {
            false
        }
    } else {
        false
    }
}
//     magnet_geometry(&br, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     tl = p;
//     if tl.wx >= br.wx {
//         tl.wx = br.wx - self.snap_distance;
//     }
//     if tl.wy >= br.wy {
//         tl.wy = br.wy - self.snap_distance;
//     }
//     tr.wy = tl.wy;
//     bl.wx = tl.wx;
// }
// TopRight => {
//     magnet_geometry(&bl, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     tr = p;
//     if tr.wx <= bl.wx {
//         tr.wx = bl.wx + self.snap_distance;
//     }
//     if tr.wy >= bl.wy {
//         tr.wy = bl.wy - self.snap_distance;
//     }
//     tl.wy = tr.wy;
//     br.wx = tr.wx;
// }
// BottomRight => {
//     magnet_geometry(&tl, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     br = p;
//     if br.wx <= tl.wx {
//         br.wx = tl.wx + self.snap_distance;
//     }
//     if br.wy <= tl.wy {
//         br.wy = tl.wy + self.snap_distance;
//     }
//     tr.wx = br.wx;
//     bl.wy = br.wy;

// // Line spliting (1 pt)
// #[allow(dead_code)]
// pub fn split_line(
//     pt: &WPoint,
//     shape: &SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if let SimpleShape::Line(start, end) = shape {
//         if is_point_on_point(pt, &start.1, EPSILON) || is_point_on_point(pt, &end.1, EPSILON) {
//             return None;
//         };
//         if is_point_on_line(pt, shape, EPSILON) {
//             Some((
//                 SimpleShape::Line((Handle::Start, start.1.clone()), (Handle::End, pt.clone())),
//                 SimpleShape::Line((Handle::Start, pt.clone()), (Handle::End, end.1.clone())),
//             ))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
// // Quad Bezier curve spliting (1 pt)
// #[allow(dead_code)]
// pub fn split_quad_bezier(
//     pt: &WPoint,
//     shape: &SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if let SimpleShape::QuadBezier(start, ctrl, end) = shape {
//         if let Some(t) = find_t_for_point_on_quad_bezier(pt, &start.1, &ctrl.1, &end.1) {
//             let ctrl1 = start.1.lerp(&ctrl.1, t);
//             let ctrl2 = ctrl.1.lerp(&end.1, t);
//             let split = ctrl1.lerp(&ctrl2, t);
//             Some((
//                 SimpleShape::QuadBezier(
//                     (Handle::Start, start.1.clone()),
//                     (Handle::Ctrl, ctrl1),
//                     (Handle::End, split),
//                 ),
//                 SimpleShape::QuadBezier(
//                     (Handle::Start, split),
//                     (Handle::Ctrl, ctrl2),
//                     (Handle::End, end.1.clone()),
//                 ),
//             ))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
// // Cubic Bezier curve spliting (1 pt)
// #[allow(dead_code)]
// pub fn split_cubic_bezier(
//     pt: &WPoint,
//     shape: SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if let SimpleShape::CubicBezier(start, ctrl1, ctrl2, end) = shape {
//         if let Some(t) = find_t_for_point_on_cubic_bezier(pt, &start.1, &ctrl1.1, &ctrl2.1, &end.1)
//         {
//             let p0_prime = start.1.lerp(&ctrl1.1, t);
//             let p1_prime = ctrl1.1.lerp(&ctrl2.1, t);
//             let p2_prime = ctrl2.1.lerp(&end.1, t);
//             let q0 = p0_prime.lerp(&p1_prime, t);
//             let q1 = p1_prime.lerp(&p2_prime, t);
//             let r = q0.lerp(&q1, t);
//             Some((
//                 SimpleShape::CubicBezier(
//                     (Handle::Start, start.1.clone()),
//                     (Handle::Ctrl, p0_prime),
//                     (Handle::End, q0),
//                     (Handle::End, r.clone()),
//                 ),
//                 SimpleShape::CubicBezier(
//                     (Handle::Start, r),
//                     (Handle::Ctrl, q1),
//                     (Handle::End, p2_prime),
//                     (Handle::End, end.1.clone()),
//                 ),
//             ))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
// // Rectangle spliting (2 pts)
// #[allow(dead_code)]
// pub fn split_rectangle(
//     pt1: &WPoint,
//     pt2: &WPoint,
//     shape: SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if pt1.dist(pt2) > EPSILON {
//         if let SimpleShape::Rectangle(bl, tl, tr, br) = shape {
//             let lines = vec![
//                 SimpleShape::Line((Handle::Start, bl), (Handle::End, tl)),
//                 SimpleShape::Line((Handle::Start, tl), (Handle::End, tr)),
//                 SimpleShape::Line((Handle::Start, tr), (Handle::End, br)),
//                 SimpleShape::Line((Handle::Start, br), (Handle::End, bl)),
//             ];
//             let mut oidx1 = None;
//             let mut oidx2 = None;
//             for (idx, line) in lines.iter().enumerate() {
//                 if let Some(v) = split_line(pt1, &line) {
//                     oidx1 = Some(idx);
//                     break;
//                 }
//             }
//             for (idx, line) in lines.iter().enumerate() {
//                 if let Some(v) = split_line(pt2, &line) {
//                     oidx2 = Some(idx);
//                     break;
//                 }
//             }
//             if let Some(idx1) = oidx1 {
//                 if let Some(idx2) = oidx2 {
//                     // TBD
//                     None
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             }
//             //
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }
// // Ellipse curve splitting (2 pts)
// #[allow(dead_code)]
// pub fn split_ellipse(
//     pt1: &WPoint,
//     pt2: &WPoint,
//     shape: SimpleShape,
// ) -> Option<(SimpleShape, SimpleShape)> {
//     if let SimpleShape::Ellipse(
//         center,
//         radius,
//         h_start_angle,
//         h_end_angle,
//         (rotation, start_angle, end_angle),
//     ) = shape
//     {
//         if pt1.dist(pt2) > EPSILON {
//             // Getting the angles for pt1 and pt2
//             let angle_pt1 = get_angle_from_point(pt1, &center.1, rotation);
//             let angle_pt2 = get_angle_from_point(pt2, &center.1, rotation);
//             // Ensuring angle_pt1 is smaller than angle_pt2
//             let (min_angle, max_angle) = if angle_pt1 < angle_pt2 {
//                 (angle_pt1, angle_pt2)
//             } else {
//                 (angle_pt2, angle_pt1)
//             };
//             let h_start_angle = get_point_from_angle(&center.1, &radius.1, rotation, -start_angle);
//             let h_min_angle = get_point_from_angle(&center.1, &radius.1, rotation, -min_angle);
//             let h_max_angle = get_point_from_angle(&center.1, &radius.1, rotation, -max_angle);
//             let h_end_angle = get_point_from_angle(&center.1, &radius.1, rotation, -end_angle);
//             Some((
//                 SimpleShape::Ellipse(
//                     (Handle::Center, center.1.clone()),
//                     (Handle::End, center.1.clone() + radius.1.clone()),
//                     (Handle::StartAngle, h_start_angle.addxy(center.1.wx, 0.)),
//                     (Handle::EndAngle, h_min_angle.addxy(center.1.wx, 0.)),
//                     (rotation, start_angle, min_angle),
//                 ),
//                 SimpleShape::Ellipse(
//                     (Handle::Center, center.1.clone()),
//                     (Handle::End, center.1.clone() + radius.1.clone()),
//                     (Handle::StartAngle, h_max_angle.addxy(center.1.wx, 0.)),
//                     (Handle::EndAngle, h_end_angle.addxy(center.1.wx, 0.)),
//                     (rotation, max_angle, end_angle),
//                 ),
//             ))
//         } else {
//             None
//         }
//     } else {
//         None
//     }
// }

pub fn get_point_on_quad_bezier(t: f64, start: &WPoint, ctrl: &WPoint, end: &WPoint) -> WPoint {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;

    let mut result = WPoint::default();
    result.wx = uu * start.wx + 2.0 * u * t * ctrl.wx + tt * end.wx;
    result.wy = uu * start.wy + 2.0 * u * t * ctrl.wy + tt * end.wy;

    result
}
pub fn get_point_on_cubic_bezier(
    t: f64,
    start: &WPoint,
    ctrl1: &WPoint,
    ctrl2: &WPoint,
    end: &WPoint,
) -> WPoint {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;

    let mut result = *start * uuu; // (1-t)^3 * start
    result += *ctrl1 * 3.0 * uu * t; // 3(1-t)^2 * t * ctrl1
    result += *ctrl2 * 3.0 * u * tt; // 3(1-t) * t^2 * ctrl2
    result += *end * ttt; // t^3 * end

    result
}
#[inline]
pub fn get_atan2(point: &WPoint) -> f64 {
    point.wy.atan2(point.wx)
}
#[inline]
pub fn get_point_from_angle(radius: &WPoint, angle: f64) -> WPoint {
    let x = radius.wx.abs() * angle.cos();
    let y = radius.wy.abs() * angle.sin();
    WPoint { wx: x, wy: y }
}

#[allow(dead_code)]
fn find_t_for_point_on_quad_bezier(
    p: &WPoint,
    start: &WPoint,
    ctrl: &WPoint,
    end: &WPoint,
) -> Option<f64> {
    let mut t_min = 0.0;
    let mut t_max = 1.0;

    for _ in 0..MAX_ITERATIONS {
        let t_mid = (t_min + t_max) / 2.0;
        let mid_point = get_point_on_quad_bezier(t_mid, start, ctrl, end);

        let dist = mid_point.dist(p);
        if dist < EPSILON {
            return Some(t_mid);
        }

        if get_point_on_quad_bezier((t_min + t_mid) / 2.0, start, ctrl, end).dist(p) < dist {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }

    None
}

#[allow(dead_code)]
fn find_t_for_point_on_cubic_bezier(
    p: &WPoint,
    start: &WPoint,
    ctrl1: &WPoint,
    ctrl2: &WPoint,
    end: &WPoint,
) -> Option<f64> {
    let mut t_min = 0.0;
    let mut t_max = 1.0;

    for _ in 0..MAX_ITERATIONS {
        let t_mid = (t_min + t_max) / 2.0;
        let mid_point = get_point_on_cubic_bezier(t_mid, start, ctrl1, ctrl2, end);

        let dist = mid_point.dist(p);
        if dist < EPSILON {
            return Some(t_mid);
        }

        if get_point_on_cubic_bezier((t_min + t_mid) / 2.0, start, ctrl1, ctrl2, end).dist(p) < dist
        {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }

    None
}
