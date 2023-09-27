use crate::shapes::{ConstructionType, Handle, ShapeType};
use std::{
    f64::consts::PI,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

const EPSILON: f64 = 1e-5; // Some small value
const MAX_ITERATIONS: usize = 100; // Or some other reasonable upper bound

pub fn magnet_geometry(pt1: &WPoint, pt2: &mut WPoint, snap_distance: f64) -> bool {
    let dx = (pt2.wx - pt1.wx).abs();
    let dy = (pt2.wy - pt1.wy).abs();

    let snap_distance = 2. * snap_distance;

    if dy < snap_distance {
        pt2.wy = pt1.wy;
        pt2.wx = pt2.wx;
        true
    } else {
        if dx < snap_distance {
            pt2.wx = pt1.wx;
            pt2.wy = pt2.wy;
            true
        } else {
            let x1 = pt1.wx;
            let y1 = pt1.wy;
            let x2 = pt2.wx;
            let y2 = pt2.wy;
            // Projection of p on (pt1, m=1)
            let p_proj = WPoint {
                wx: (x1 + x2 + y2 - y1) / 2.,
                wy: (-x1 + x2 + y2 + y1) / 2.,
            };
            if p_proj.dist(pt2) < snap_distance {
                *pt2 = p_proj;
                true
            } else {
                // Projection of p on (pt1, m=-1)
                let p_proj = WPoint {
                    wx: (x1 + x2 - y2 + y1) / 2.,
                    wy: (x1 - x2 + y2 + y1) / 2.,
                };

                if p_proj.dist(pt2) < snap_distance {
                    *pt2 = p_proj;
                    true
                } else {
                    *pt2 = *pt2;
                    false
                }
            }
        }
    }
}

fn is_vert(pt1: &WPoint, pt2: &WPoint) -> bool {
    (pt1.wx - pt2.wx).abs() < 0.001
}
fn is_hori(pt1: &WPoint, pt2: &WPoint) -> bool {
    (pt1.wy - pt2.wy).abs() < 0.001
}
fn is_45_135(pt1: &WPoint, pt2: &WPoint) -> bool {
    let dy = pt2.wy - pt1.wy;
    let dx = pt2.wx - pt1.wx;
    if dx != 0. {
        (dy / dx).abs() > 1. / 1.01 && (dy / dx).abs() < 1.01
    } else {
        false
    }
}

pub fn push_45_135(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    if is_45_135(pt1, pt2) {
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
}
pub fn push_vertical(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    use ConstructionType::*;
    if is_vert(pt1, pt2) {
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
}
pub fn push_horizontal(pt1: &WPoint, pt2: &WPoint, full: bool, cst: &mut Vec<ConstructionType>) {
    use ConstructionType::*;
    if is_hori(pt1, pt2) {
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
}
pub fn push_handle(pt: &WPoint, size_handle: &WPoint, fill: bool, cst: &mut Vec<ConstructionType>) {
    let radius = *size_handle / 2.;
    use ConstructionType::*;
    cst.push(Move(
        *pt + WPoint {
            wx: radius.wx,
            wy: 0.,
        },
    ));
    cst.push(Ellipse(*pt, radius, 0., 0., 2. * PI, fill));
}

pub fn snap_to_snap_grid(pos: &mut WPoint, snap_distance: f64) {
    pos.wx = (pos.wx / snap_distance).round() * snap_distance;
    pos.wy = (pos.wy / snap_distance).round() * snap_distance;
}
pub fn snap_to_snap_grid_y(pos: &mut WPoint, grid_spacing: f64) {
    pos.wy = (pos.wy / grid_spacing).round() * grid_spacing;
}
pub fn snap_to_snap_grid_x(pos: &mut WPoint, grid_spacing: f64) {
    pos.wx = (pos.wx / grid_spacing).round() * grid_spacing;
}

// pub fn snap_equidistant(handles: &mut Vec<WXY>, idx: &usize, idxs: &[usize; 2], snap_val: f64) {
//     let pt = handles[*idx];
//     let pt1 = handles[idxs[0]];
//     let pt2 = handles[idxs[1]];
//     let mid = (pt1 + pt2) / 2.0;
//     let dx = pt2.wx - pt1.wx;
//     let dy = pt2.wy - pt1.wy;
//     if dx == 0. && dy == 0. {
//         return;
//     }
//     let proj = if dx == 0. {
//         WXY {
//             wx: pt.wx,
//             wy: (pt2.wy + pt1.wy) / 2.,
//         }
//     } else {
//         if dy == 0. {
//             WXY {
//                 wx: (pt2.wx + pt1.wx) / 2.,
//                 wy: pt.wy,
//             }
//         } else {
//             let slope = dy / dx;
//             let perp_slope = -1. / slope;
//             let x_p = (perp_slope * mid.wx - slope * pt.wx + pt.wy - mid.wy) / (perp_slope - slope);
//             let y_p = perp_slope * (x_p - mid.wx) + mid.wy;
//             WXY { wx: x_p, wy: y_p }
//         }
//     };
//     if pt.dist(&proj) < snap_val {
//         handles[*idx] = proj;
//     }
// }

pub fn is_point_on_point(pt: &WPoint, pt1: &WPoint, precision: f64) -> bool {
    pt.dist(pt1) < precision
}
pub fn is_point_on_line(pt: &WPoint, shape: &ShapeType, precision: f64) -> bool {
    if let ShapeType::Line(start, end) = shape {
        let denominator =
            ((end.1.wy - start.1.wy).powf(2.) + (end.1.wx - start.1.wx).powf(2.)).sqrt();
        if denominator == 0. {
            return is_point_on_point(pt, &start.1, precision);
        }
        let numerator = ((end.1.wy - start.1.wy) * pt.wx - (end.1.wx - start.1.wx) * pt.wy
            + end.1.wx * start.1.wy
            - end.1.wy * start.1.wx)
            .abs();

        if numerator / denominator > precision {
            return false;
        }
        is_between(pt, &start.1, &end.1)
    } else {
        false
    }
}
pub fn is_point_on_quadbezier(pt: &WPoint, shape: &ShapeType, precision: f64) -> bool {
    if let ShapeType::QuadBezier(start, ctrl, end) = shape {
        let mut t_min = 0.;
        let mut t_max = 1.;
        let mut min_dist = f64::MAX;
        for _i in 0..MAX_ITERATIONS {
            // max iterations can be adjusted
            let t_mid = (t_min + t_max) / 2.;
            let bt = get_point_on_quad_bezier(t_mid, &start.1, &ctrl.1, &end.1);
            let dist = bt.dist(pt);
            if dist < min_dist {
                min_dist = dist;
            }
            if dist < precision {
                return true; // We found a sufficiently close point
            }
            // Using gradient to decide the next tMid for the next iteration.
            let gradient = (bt.wx - pt.wx) * (end.1.wx - start.1.wx)
                + (bt.wy - pt.wy) * (end.1.wy - start.1.wy);
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
pub fn is_point_on_cubicbezier(pt: &WPoint, shape: &ShapeType, precision: f64) -> bool {
    if let ShapeType::CubicBezier(start, ctrl1, ctrl2, end) = shape {
        let mut t_min = 0.;
        let mut t_max = 1.;
        let mut min_dist = f64::MAX;

        for _i in 0..MAX_ITERATIONS {
            let t_mid = (t_min + t_max) / 2.;
            let bt = get_point_on_cubic_bezier(t_mid, &start.1, &ctrl1.1, &ctrl2.1, &end.1);
            let dist = bt.dist(pt);
            if dist < min_dist {
                min_dist = dist;
            }
            if dist < precision {
                return true; // We found a sufficiently close point
            }
            // Using gradient to decide the next tMid for the next iteration.
            let gradient = (bt.wx - pt.wx) * (end.1.wx - start.1.wx)
                + (bt.wy - pt.wy) * (end.1.wy - start.1.wy);
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
// pub fn is_point_on_ellipse(
//     pt: &WPoint,
//     center: &WPoint,
//     radius: &WPoint,
//     mut precision: f64,
// ) -> bool {
//     precision /= radius.norm();
//     precision *= 2.;
//     let value = (pt.wx - center.wx).powf(2.) / (radius.wx * radius.wx)
//         + (pt.wy - center.wy).powf(2.) / (radius.wy * radius.wy);
//     value < 1. + precision && value > 1. - precision
// }
pub fn is_point_on_ellipse(pt: &WPoint, shape: &ShapeType, precision: f64) -> bool {
    if let ShapeType::Ellipse(
        center,
        radius,
        h_start_angle,
        h_end_angle,
        (rotation, start_angle, end_angle),
    ) = shape
    {
        // Translate point
        let translated_pt = *pt - center.1;
        // Rotate the point
        let cos_rotation = rotation.cos();
        let sin_rotation = rotation.sin();
        let rotated_pt = WPoint {
            wx: cos_rotation * translated_pt.wx + sin_rotation * translated_pt.wy,
            wy: -sin_rotation * translated_pt.wx + cos_rotation * translated_pt.wy,
        };
        // Check if point is on the axis-aligned ellipse
        let on_ellipse = (rotated_pt.wx * rotated_pt.wx) / (radius.1.wx * radius.1.wx)
            + (rotated_pt.wy * rotated_pt.wy) / (radius.1.wy * radius.1.wy);
        if (on_ellipse - 1.0).abs() > precision {
            return false;
        }
        // Check if point is within the angle range
        let angle = rotated_pt.wy.atan2(rotated_pt.wx);
        angle >= *start_angle && angle <= *end_angle
    } else {
        false
    }
}

pub fn is_box_inside(box_outer: &[WPoint; 2], box_inner: &[WPoint; 2]) -> bool {
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

// Line spliting (1 pt)
#[allow(dead_code)]
pub fn split_line(pt: &WPoint, shape: &ShapeType) -> Option<(ShapeType, ShapeType)> {
    if let ShapeType::Line(start, end) = shape {
        if is_point_on_point(pt, &start.1, EPSILON) || is_point_on_point(pt, &end.1, EPSILON) {
            return None;
        };
        if is_point_on_line(pt, shape, EPSILON) {
            Some((
                ShapeType::Line((Handle::Start, start.1.clone()), (Handle::End, pt.clone())),
                ShapeType::Line((Handle::Start, pt.clone()), (Handle::End, end.1.clone())),
            ))
        } else {
            None
        }
    } else {
        None
    }
}
// Quad Bezier curve spliting (1 pt)
#[allow(dead_code)]
pub fn split_quad_bezier(pt: &WPoint, shape: &ShapeType) -> Option<(ShapeType, ShapeType)> {
    if let ShapeType::QuadBezier(start, ctrl, end) = shape {
        if let Some(t) = find_t_for_point_on_quad_bezier(pt, &start.1, &ctrl.1, &end.1) {
            let ctrl1 = start.1.lerp(&ctrl.1, t);
            let ctrl2 = ctrl.1.lerp(&end.1, t);
            let split = ctrl1.lerp(&ctrl2, t);
            Some((
                ShapeType::QuadBezier(
                    (Handle::Start, start.1.clone()),
                    (Handle::Ctrl, ctrl1),
                    (Handle::End, split),
                ),
                ShapeType::QuadBezier(
                    (Handle::Start, split),
                    (Handle::Ctrl, ctrl2),
                    (Handle::End, end.1.clone()),
                ),
            ))
        } else {
            None
        }
    } else {
        None
    }
}
// Cubic Bezier curve spliting (1 pt)
#[allow(dead_code)]
pub fn split_cubic_bezier(pt: &WPoint, shape: ShapeType) -> Option<(ShapeType, ShapeType)> {
    if let ShapeType::CubicBezier(start, ctrl1, ctrl2, end) = shape {
        if let Some(t) = find_t_for_point_on_cubic_bezier(pt, &start.1, &ctrl1.1, &ctrl2.1, &end.1)
        {
            let p0_prime = start.1.lerp(&ctrl1.1, t);
            let p1_prime = ctrl1.1.lerp(&ctrl2.1, t);
            let p2_prime = ctrl2.1.lerp(&end.1, t);
            let q0 = p0_prime.lerp(&p1_prime, t);
            let q1 = p1_prime.lerp(&p2_prime, t);
            let r = q0.lerp(&q1, t);
            Some((
                ShapeType::CubicBezier(
                    (Handle::Start, start.1.clone()),
                    (Handle::Ctrl, p0_prime),
                    (Handle::End, q0),
                    (Handle::End, r.clone()),
                ),
                ShapeType::CubicBezier(
                    (Handle::Start, r),
                    (Handle::Ctrl, q1),
                    (Handle::End, p2_prime),
                    (Handle::End, end.1.clone()),
                ),
            ))
        } else {
            None
        }
    } else {
        None
    }
}
// Rectangle spliting (2 pts)
#[allow(dead_code)]
pub fn split_rectangle(
    pt1: &WPoint,
    pt2: &WPoint,
    shape: ShapeType,
) -> Option<(ShapeType, ShapeType)> {
    if pt1.dist(pt2) > EPSILON {
        if let ShapeType::Rectangle(bl, tl, tr, br) = shape {
            let lines = vec![
                ShapeType::Line((Handle::Start, bl.1), (Handle::End, tl.1)),
                ShapeType::Line((Handle::Start, tl.1), (Handle::End, tr.1)),
                ShapeType::Line((Handle::Start, tr.1), (Handle::End, br.1)),
                ShapeType::Line((Handle::Start, br.1), (Handle::End, bl.1)),
            ];
            let mut oidx1 = None;
            let mut oidx2 = None;
            for (idx, line) in lines.iter().enumerate() {
                if let Some(v) = split_line(pt1, &line) {
                    oidx1 = Some(idx);
                    break;
                }
            }
            for (idx, line) in lines.iter().enumerate() {
                if let Some(v) = split_line(pt2, &line) {
                    oidx2 = Some(idx);
                    break;
                }
            }
            if let Some(idx1) = oidx1 {
                if let Some(idx2) = oidx2 {
                    // TBD
                    None
                } else {
                    None
                }
            } else {
                None
            }
            //
        } else {
            None
        }
    } else {
        None
    }
}
// Ellipse curve splitting (2 pts)
#[allow(dead_code)]
pub fn split_ellipse(
    pt1: &WPoint,
    pt2: &WPoint,
    shape: ShapeType,
) -> Option<(ShapeType, ShapeType)> {
    if let ShapeType::Ellipse(
        center,
        radius,
        h_start_angle,
        h_end_angle,
        (rotation, start_angle, end_angle),
    ) = shape
    {
        if pt1.dist(pt2) > EPSILON {
            // Getting the angles for pt1 and pt2
            let angle_pt1 = get_angle_from_point(pt1, &center.1, rotation);
            let angle_pt2 = get_angle_from_point(pt2, &center.1, rotation);

            // Ensuring angle_pt1 is smaller than angle_pt2
            let (min_angle, max_angle) = if angle_pt1 < angle_pt2 {
                (angle_pt1, angle_pt2)
            } else {
                (angle_pt2, angle_pt1)
            };
            let h_start_angle = get_point_from_angle(&center.1, &radius.1, rotation, -start_angle);
            let h_min_angle = get_point_from_angle(&center.1, &radius.1, rotation, -min_angle);
            let h_max_angle = get_point_from_angle(&center.1, &radius.1, rotation, -max_angle);
            let h_end_angle = get_point_from_angle(&center.1, &radius.1, rotation, -end_angle);
            Some((
                ShapeType::Ellipse(
                    (Handle::Center, center.1.clone()),
                    (Handle::End, center.1.clone() + radius.1.clone()),
                    (Handle::StartAngle, h_start_angle.addxy(center.1.wx, 0.)),
                    (Handle::EndAngle, h_min_angle.addxy(center.1.wx, 0.)),
                    (rotation, start_angle, min_angle),
                ),
                ShapeType::Ellipse(
                    (Handle::Center, center.1.clone()),
                    (Handle::End, center.1.clone() + radius.1.clone()),
                    (Handle::StartAngle, h_max_angle.addxy(center.1.wx, 0.)),
                    (Handle::EndAngle, h_end_angle.addxy(center.1.wx, 0.)),
                    (rotation, max_angle, end_angle),
                ),
            ))
        } else {
            None
        }
    } else {
        None
    }
}

fn get_point_on_quad_bezier(t: f64, start: &WPoint, ctrl: &WPoint, end: &WPoint) -> WPoint {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;

    let mut result = WPoint { wx: 0.0, wy: 0.0 };
    result.wx = uu * start.wx + 2.0 * u * t * ctrl.wx + tt * end.wx;
    result.wy = uu * start.wy + 2.0 * u * t * ctrl.wy + tt * end.wy;

    result
}
fn get_point_on_cubic_bezier(
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
pub fn get_angle_from_point(point: &WPoint, center: &WPoint, rotation: f64) -> f64 {
    // Computing the angle without rotation
    let angle = (point.wy - center.wy).atan2(point.wx - center.wx);
    // Factoring in the rotation
    let adjusted_angle = angle - rotation;
    adjusted_angle
}
pub fn get_point_from_angle(center: &WPoint, radius: &WPoint, rotation: f64, angle: f64) -> WPoint {
    let x = center.wx + (radius.wx.abs() - center.wx) * (angle + rotation).cos();
    let y = center.wy + (radius.wy.abs() - center.wy) * (angle + rotation).sin();
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

#[derive(Copy, Clone, Debug)]
pub struct WPoint {
    pub wx: f64,
    pub wy: f64,
}
impl WPoint {
    pub fn to_canvas(&self, scale: f64, offset: CXY) -> CXY {
        let canvas_x = (self.wx * scale) + offset.cx;
        let canvas_y = (self.wy * scale) + offset.cy;
        CXY {
            cx: canvas_x,
            cy: canvas_y,
        }
    }
    #[allow(dead_code)]
    pub fn addxy(&self, wx: f64, wy: f64) -> WPoint {
        WPoint {
            wx: self.wx + wx,
            wy: self.wy + wy,
        }
    }
    pub fn abs(&self) -> WPoint {
        WPoint {
            wx: self.wx.abs(),
            wy: self.wy.abs(),
        }
    }

    pub fn dist(&self, other: &WPoint) -> f64 {
        let dpt = *self - *other;
        (dpt.wx * dpt.wx + dpt.wy * dpt.wy).sqrt()
    }
    #[allow(dead_code)]
    pub fn norm(&self) -> f64 {
        (self.wx * self.wx + self.wy * self.wy).sqrt()
    }
    pub fn lerp(&self, other: &WPoint, t: f64) -> WPoint {
        WPoint {
            wx: self.wx + t * (other.wx - self.wx),
            wy: self.wy + t * (other.wy - self.wy),
        }
    }
}
impl Default for WPoint {
    fn default() -> Self {
        WPoint { wx: 0.0, wy: 0.0 }
    }
}
impl Neg for WPoint {
    type Output = WPoint;

    fn neg(self) -> WPoint {
        WPoint {
            wx: -self.wx,
            wy: -self.wy,
        }
    }
}
impl Add for WPoint {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            wx: self.wx + other.wx,
            wy: self.wy + other.wy,
        }
    }
}
impl Add<f64> for WPoint {
    type Output = WPoint;
    fn add(self, scalar: f64) -> Self::Output {
        WPoint {
            wx: self.wx + scalar,
            wy: self.wy + scalar,
        }
    }
}
impl AddAssign for WPoint {
    fn add_assign(&mut self, other: WPoint) {
        self.wx += other.wx;
        self.wy += other.wy;
    }
}
impl AddAssign<f64> for WPoint {
    fn add_assign(&mut self, scalar: f64) {
        self.wx += scalar;
        self.wy += scalar;
    }
}
impl Sub for WPoint {
    type Output = WPoint;
    fn sub(self, other: WPoint) -> WPoint {
        WPoint {
            wx: self.wx - other.wx,
            wy: self.wy - other.wy,
        }
    }
}
impl Sub<f64> for WPoint {
    type Output = WPoint;
    fn sub(self, scalar: f64) -> Self::Output {
        WPoint {
            wx: self.wx - scalar,
            wy: self.wy - scalar,
        }
    }
}
impl SubAssign for WPoint {
    fn sub_assign(&mut self, other: WPoint) {
        self.wx -= other.wx;
        self.wy -= other.wy;
    }
}
impl Div<f64> for WPoint {
    type Output = WPoint;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        WPoint {
            wx: self.wx / rhs,
            wy: self.wy / rhs,
        }
    }
}
impl DivAssign<f64> for WPoint {
    fn div_assign(&mut self, rhs: f64) {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        self.wx /= rhs;
        self.wy /= rhs;
    }
}
impl Mul<f64> for WPoint {
    type Output = WPoint;

    fn mul(self, rhs: f64) -> Self::Output {
        WPoint {
            wx: self.wx * rhs,
            wy: self.wy * rhs,
        }
    }
}
impl MulAssign<f64> for WPoint {
    fn mul_assign(&mut self, rhs: f64) {
        self.wx *= rhs;
        self.wy *= rhs;
    }
}
impl PartialEq for WPoint {
    fn eq(&self, other: &Self) -> bool {
        self.wx == other.wx && self.wy == other.wy
    }
}
impl Eq for WPoint {}

#[derive(Copy, Clone, Debug)]
pub struct CXY {
    pub cx: f64,
    pub cy: f64,
}
impl CXY {
    pub fn to_world(&self, scale: f64, offset: CXY) -> WPoint {
        let world_x = (self.cx - offset.cx) / scale;
        let world_y = (self.cy - offset.cy) / scale;
        WPoint {
            wx: world_x,
            wy: world_y,
        }
    }
}
impl Default for CXY {
    fn default() -> Self {
        CXY { cx: 0., cy: 0. }
    }
}
impl Add for CXY {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            cx: self.cx + other.cx,
            cy: self.cy + other.cy,
        }
    }
}
impl Add<f64> for CXY {
    type Output = CXY;
    fn add(self, scalar: f64) -> Self::Output {
        CXY {
            cx: self.cx + scalar,
            cy: self.cy + scalar,
        }
    }
}
impl AddAssign for CXY {
    fn add_assign(&mut self, other: CXY) {
        self.cx += other.cx;
        self.cy += other.cy;
    }
}
impl AddAssign<f64> for CXY {
    fn add_assign(&mut self, scalar: f64) {
        self.cx += scalar;
        self.cy += scalar;
    }
}
impl Sub for CXY {
    type Output = CXY;
    fn sub(self, other: CXY) -> CXY {
        CXY {
            cx: self.cx - other.cx,
            cy: self.cy - other.cy,
        }
    }
}
impl SubAssign for CXY {
    fn sub_assign(&mut self, other: CXY) {
        self.cx -= other.cx;
        self.cy -= other.cy;
    }
}
impl Div<f64> for CXY {
    type Output = CXY;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0. {
            panic!("Division by zero");
        }
        CXY {
            cx: self.cx / rhs,
            cy: self.cy / rhs,
        }
    }
}
impl DivAssign<f64> for CXY {
    fn div_assign(&mut self, rhs: f64) {
        if rhs == 0. {
            panic!("Division by zero");
        }
        self.cx /= rhs;
        self.cy /= rhs;
    }
}
impl Mul<f64> for CXY {
    type Output = CXY;

    fn mul(self, rhs: f64) -> Self::Output {
        CXY {
            cx: self.cx * rhs,
            cy: self.cy * rhs,
        }
    }
}
impl MulAssign<f64> for CXY {
    fn mul_assign(&mut self, rhs: f64) {
        self.cx *= rhs;
        self.cy *= rhs;
    }
}
impl PartialEq for CXY {
    fn eq(&self, other: &Self) -> bool {
        self.cx == other.cx && self.cy == other.cy
    }
}
impl Eq for CXY {}
