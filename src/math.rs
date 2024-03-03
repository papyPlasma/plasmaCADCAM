#![allow(dead_code)]
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

use kurbo::{BezPath, PathEl, Point};

use std::f64::consts::PI;

pub fn is_aligned_vert(pt1: &Point, pt2: &Point) -> bool {
    // I can do this because of snaping
    (pt1.x - pt2.x).abs() == 0.
}
pub fn helper_vertical(pt1: &Point, pt2: &Point, full: bool) -> BezPath {
    use PathEl::*;
    let mut v: Vec<PathEl> = vec![];
    let y1 = 2. * pt1.y - pt2.y;
    let y2 = 2. * pt2.y - pt1.y;

    v.push(MoveTo(*pt1));
    if full {
        v.push(LineTo(Point::new(pt1.x, y1)));
        v.push(LineTo(*pt2));
        v.push(LineTo(Point::new(pt1.x, y2)));
    } else {
        v.push(LineTo(Point::new(pt1.x, y1)));
        v.push(MoveTo(*pt2));
        v.push(LineTo(Point::new(pt1.x, y2)));
    }
    BezPath::from_vec(v)
}

pub fn is_aligned_hori(pt1: &Point, pt2: &Point) -> bool {
    // I can do this because of snaping
    (pt1.y - pt2.y).abs() == 0.
}
pub fn helper_horizontal(pt1: &Point, pt2: &Point, full: bool) -> BezPath {
    use PathEl::*;
    let mut v: Vec<PathEl> = vec![];
    let x1 = 2. * pt1.x - pt2.x;
    let x2 = 2. * pt2.x - pt1.x;

    v.push(MoveTo(*pt1));
    if full {
        v.push(LineTo(Point::new(x1, pt1.y)));
        v.push(LineTo(*pt2));
        v.push(LineTo(Point::new(x2, pt1.y)));
    } else {
        v.push(LineTo(Point::new(x1, pt1.y)));
        v.push(MoveTo(*pt2));
        v.push(LineTo(Point::new(x2, pt1.y)));
    }
    BezPath::from_vec(v)
}

pub fn _is_aligned_45_or_135(pt1: &Point, pt2: &Point) -> bool {
    let dy = pt2.y - pt1.y;
    let dx = pt2.x - pt1.x;
    if dx != 0. {
        let m = (dy / dx).abs();
        // Equality test works because of snapping
        m == 1.
    } else {
        false
    }
}
pub fn _helper_45_135(pt1: &Point, pt2: &Point, full: bool) -> BezPath {
    use PathEl::*;
    let mut v: Vec<PathEl> = vec![];
    let x1 = 2. * pt1.x - pt2.x;
    let y1 = 2. * pt1.y - pt2.y;
    let x2 = 2. * pt2.x - pt1.x;
    let y2 = 2. * pt2.y - pt1.y;

    v.push(MoveTo(*pt1));
    if full {
        v.push(LineTo(Point::new(x1, y1)));
        v.push(LineTo(*pt2));
        v.push(LineTo(Point::new(x2, y2)));
    } else {
        v.push(LineTo(Point::new(x1, y1)));
        v.push(MoveTo(*pt2));
        v.push(LineTo(Point::new(x2, y2)));
    }

    BezPath::from_vec(v)
}

fn _is_between(pt: &Point, pt1: &Point, pt2: &Point) -> bool {
    let dot_product = (pt.x - pt1.x) * (pt2.x - pt1.x) + (pt.y - pt1.y) * (pt2.y - pt1.y);
    if dot_product < 0. {
        return false;
    }
    let length2 = (pt2.x - pt1.x).powf(2.) + (pt2.y - pt1.y).powf(2.);
    if dot_product > length2 {
        return false;
    }
    return true;
}
// pub fn is_point_on_segment(pos1: &Point, pos2: &Point, pos: &Point, precision: f64) -> bool {
//     let denominator = ((pos2.y - pos1.y).powf(2.) + (pos2.x - pos1.x).powf(2.)).sqrt();
//     if denominator == 0. {
//         return is_point_on_point(pos, &pos1, precision);
//     }
//     let numerator = ((pos2.y - pos1.y) * pos.x - (pos2.x - pos1.x) * pos.y
//         + pos2.x * pos1.y
//         - pos2.y * pos1.x)
//         .abs();
//     if numerator / denominator > precision {
//         return false;
//     }
//     is_between(pos, &pos1, &pos2)
// }

pub fn is_box_inside(box_outer: &[Point; 2], box_inner: &[Point; 2]) -> bool {
    let bl_outer = box_outer[0];
    let tr_outer = box_outer[1];
    let bl_inner = box_inner[0];
    let tr_inner = box_inner[1];
    bl_inner.x >= bl_outer.x
        && bl_inner.y >= bl_outer.y
        && tr_inner.x <= tr_outer.x
        && tr_inner.y <= tr_outer.y
}
pub fn reorder_corners(bb: &mut [Point; 2]) {
    let pt1 = bb[0];
    let pt2 = bb[1];
    if pt1.x < pt2.x {
        if pt1.y < pt2.y {
            let bl = Point { x: pt1.x, y: pt1.y };
            let tr = Point { x: pt2.x, y: pt2.y };
            bb[0] = bl;
            bb[1] = tr;
        } else {
            let bl = Point { x: pt1.x, y: pt2.y };
            let tr = Point { x: pt2.x, y: pt1.y };
            bb[0] = bl;
            bb[1] = tr;
        }
    } else {
        if pt1.y < pt2.y {
            let bl = Point { x: pt2.x, y: pt1.y };
            let tr = Point { x: pt1.x, y: pt2.y };
            bb[0] = bl;
            bb[1] = tr;
        } else {
            let bl = Point { x: pt2.x, y: pt2.y };
            let tr = Point { x: pt1.x, y: pt1.y };
            bb[0] = bl;
            bb[1] = tr;
        }
    }
}

pub fn _snap_to_positive_value(value: f64, snap_value: f64) -> f64 {
    let value = (value / snap_value).round() * snap_value;
    if value == 0. {
        snap_value
    } else {
        if value < 0. {
            -value
        } else {
            value
        }
    }
}

pub fn magnet_to_x(pos: &mut Point, ref_pos: &Point, magnet_distance: f64) {
    let dx = (pos.x - ref_pos.x).abs();
    if dx < magnet_distance {
        pos.x = ref_pos.x;
    }
}
pub fn magnet_to_y(pos: &mut Point, ref_pos: &Point, magnet_distance: f64) {
    let dy = (pos.y - ref_pos.y).abs();
    if dy < magnet_distance {
        pos.y = ref_pos.y;
    }
}
pub fn get_shape_groupmagnet_to(pos: &mut Point, ref_pos: &Point, magnet_distance: f64) {
    let dx = (pos.x - ref_pos.x).abs();
    let dy = (pos.y - ref_pos.y).abs();
    if dx < magnet_distance && dy < magnet_distance {
        *pos = *ref_pos;
    }
}
pub fn magnet_to_45(angle: &mut f64, radius_pos: &Point, magnet_distance: f64) {
    let a = radius_pos.x * angle.sin();
    let b = radius_pos.y * angle.cos();
    let dangle = magnet_distance / (a * a + b * b).sqrt();
    if *angle < PI / 4. + dangle && *angle > PI / 4. - dangle {
        *angle = PI / 4.;
    }
}
pub fn magnet_to_xy(pos: &mut Point, ref_pos: &Point, magnet_distance: f64) {
    let snap_distance = 2. * magnet_distance;

    let x1 = ref_pos.x;
    let y1 = ref_pos.y;
    let x2 = pos.x;
    let y2 = pos.y;
    // Projection of p on (pt1, m=1)
    let p_proj = Point {
        x: (x1 + x2 + y2 - y1) / 2.,
        y: (-x1 + x2 + y2 + y1) / 2.,
    };
    if p_proj.distance(*pos) < snap_distance {
        *pos = p_proj;
    } else {
        // Projection of p on (pt1, m=-1)
        let p_proj = Point {
            x: (x1 + x2 - y2 + y1) / 2.,
            y: (x1 - x2 + y2 + y1) / 2.,
        };
        if p_proj.distance(*pos) < snap_distance {
            *pos = p_proj;
        }
    }
}

// pub fn apply_cstr_direction(pt_to_modif: &mut Vertex, pt_moved: &Vertex) {
//     let (x1old, y1old) = (pt_moved.saved_pt.x, pt_moved.saved_pt.y);
//     let (x2old, y2old) = (pt_to_modif.saved_pt.x, pt_to_modif.saved_pt.y);
//     let (x1, y1) = (pt_moved.pt.x, pt_moved.pt.y);
//     let (x2, y2) = (pt_to_modif.pt.x, pt_to_modif.pt.y);
//     // Calculate the first line ( (den_a1)*y = (num_a1)*(x-x1) + (den_a1)*y1)
//     let num_a1 = y2old - y1old;
//     let den_a1 = x2old - x1old;
//     // Calculate the second line ( (den_a2)*y = (num_a2)*(x-x2) + (den_a2)*y2)
//     let num_a2 = y1 - y1old;
//     let den_a2 = x1 - x1old;
//     let den = den_a1 * num_a2 - den_a2 * num_a1;
//     if den != 0. {
//         let x3_num = den_a1 * den_a2 * y1 - den_a1 * den_a2 * y2 + den_a1 * num_a2 * x2
//             - den_a2 * num_a1 * x1;
//         let y3_num = den_a1 * num_a2 * y1 - den_a2 * num_a1 * y2 - num_a1 * num_a2 * x1
//             + num_a1 * num_a2 * x2;
//         pt_to_modif.pt = Point::new(x3_num / den, y3_num / den)
//     }
// }

/// We have two lines that must stay parallels
/// The point moved
///
// pub fn apply_cstr_parallel(
//     pt_id: &VertexId,
//     bs1_pts: &(Vertex, Vertex),
//     bs2_pts: &(Vertex, Vertex),
// ) -> ((Vertex, Vertex), (Vertex, Vertex)) {
//     let mut bs1_pts = *bs1_pts;
//     let mut bs2_pts = *bs2_pts;
//     match (
//         bs1_pts.0.id == *pt_id,
//         bs1_pts.1.id == *pt_id,
//         bs2_pts.0.id == *pt_id,
//         bs2_pts.1.id == *pt_id,
//     ) {
//         // Pt doesn't belong to the lines
//         (false, false, false, false) => (),
//         // Pt belong to bs1.0, move bs1.1 according to perpendicular of bs2 direction
//         (true, false, false, false) => {
//             // line parallel to bs2 that contains bs1.0
//             let na1 = bs2_pts.1.pt.y - bs2_pts.0.pt.y;
//             let da1 = bs2_pts.1.pt.x - bs2_pts.0.pt.x;
//             let x1 = bs1_pts.0.pt.x;
//             let y1 = bs1_pts.0.pt.y;
//             // line perp to bs2 that contains bs1.1
//             let na2 = -da1;
//             let da2 = na1;
//             let x2 = bs1_pts.1.saved_pt.x;
//             let y2 = bs1_pts.1.saved_pt.y;
//             // Intersection of the two lines
//             let den = da1 * na2 - da2 * na1;
//             if den != 0. {
//                 let x = (da1 * da2 * y1 - da1 * da2 * y2 + da1 * na2 * x2 - da2 * na1 * x1) / den;
//                 let y = (da1 * na2 * y1 - da2 * na1 * y2 - na1 * na2 * x1 + na1 * na2 * x2) / den;
//                 bs1_pts.1.pt = Point::new(x, y);
//             }
//         }
//         // Pt belong to bs1.1, move bs1.0 according to perpendicular of bs2 direction
//         (false, true, false, false) => {
//             // log!("XXXXXXXXXXXXXXXXXXXXXX");
//             // line parallel to bs2 that contains bs1.1
//             let na1 = bs2_pts.1.pt.y - bs2_pts.0.pt.y;
//             let da1 = bs2_pts.1.pt.x - bs2_pts.0.pt.x;
//             let x1 = bs1_pts.1.pt.x;
//             let y1 = bs1_pts.1.pt.y;
//             // line perp to bs2 that contains bs1.0
//             let na2 = -da1;
//             let da2 = na1;
//             let x2 = bs1_pts.0.pt.x;
//             let y2 = bs1_pts.0.pt.y;
//             // Intersection of the two lines
//             let den = da1 * na2 - da2 * na1;
//             if den != 0. {
//                 let x = (da1 * da2 * y1 - da1 * da2 * y2 + da1 * na2 * x2 - da2 * na1 * x1) / den;
//                 let y = (da1 * na2 * y1 - da2 * na1 * y2 - na1 * na2 * x1 + na1 * na2 * x2) / den;
//                 bs1_pts.0.pt = Point::new(x, y);
//             }
//         }
//         // Pt belong to bs2.0, move bs2.1 according to perpendicular of bs1 direction
//         (false, false, true, false) => {
//             // log!("XXXXXXXXXXXXXXXXXXXXXX");
//             // line parallel to bs1 that contains bs2.0
//             let na1 = bs1_pts.1.pt.y - bs1_pts.0.pt.y;
//             let da1 = bs1_pts.1.pt.x - bs1_pts.0.pt.x;
//             let x1 = bs2_pts.0.pt.x;
//             let y1 = bs2_pts.0.pt.y;
//             // line perp to bs1 that contains bs2.1
//             let na2 = -da1;
//             let da2 = na1;
//             let x2 = bs2_pts.1.pt.x;
//             let y2 = bs2_pts.1.pt.y;
//             // Intersection of the two lines
//             let den = da1 * na2 - da2 * na1;
//             if den != 0. {
//                 let x = (da1 * da2 * y1 - da1 * da2 * y2 + da1 * na2 * x2 - da2 * na1 * x1) / den;
//                 let y = (da1 * na2 * y1 - da2 * na1 * y2 - na1 * na2 * x1 + na1 * na2 * x2) / den;
//                 bs2_pts.1.pt = Point::new(x, y);
//             }
//         }
//         // Pt belong to bs2.1, move bs2.0 according to perpendicular of bs1 direction
//         (false, false, false, true) => {
//             // log!("XXXXXXXXXXXXXXXXXXXXXX");
//             // line parallel to bs1 that contains bs2.1
//             let na1 = bs1_pts.1.pt.y - bs1_pts.0.pt.y;
//             let da1 = bs1_pts.1.pt.x - bs1_pts.0.pt.x;
//             let x1 = bs2_pts.1.pt.x;
//             let y1 = bs2_pts.1.pt.y;
//             // line perp to bs1 that contains bs2.0
//             let na2 = -da1;
//             let da2 = na1;
//             let x2 = bs2_pts.0.pt.x;
//             let y2 = bs2_pts.0.pt.y;
//             // Intersection of the two lines
//             let den = da1 * na2 - da2 * na1;
//             if den != 0. {
//                 let x = (da1 * da2 * y1 - da1 * da2 * y2 + da1 * na2 * x2 - da2 * na1 * x1) / den;
//                 let y = (da1 * na2 * y1 - da2 * na1 * y2 - na1 * na2 * x1 + na1 * na2 * x2) / den;
//                 bs2_pts.0.pt = Point::new(x, y);
//             }
//         }
//         _ => (),
//     }
//     (bs1_pts, bs2_pts)
// }

// /// We have two lines that must stay parallels
// /// The point moved
// ///
// pub fn apply_cstr_perpendicular(
//     pt_id: &VertexId,
//     bs1_pts: &(Vertex, Vertex),
//     bs2_pts: &(Vertex, Vertex),
// ) -> ((Vertex, Vertex), (Vertex, Vertex)) {
//     let mut bs1_pts = *bs1_pts;
//     let mut bs2_pts = *bs2_pts;
//     match (
//         bs1_pts.0.id == *pt_id,
//         bs1_pts.1.id == *pt_id,
//         bs2_pts.0.id == *pt_id,
//         bs2_pts.1.id == *pt_id,
//     ) {
//         // Pt doesn't belong to the lines
//         (false, false, false, false) => (),
//         // Pt belongs to at least one line
//         _ => {
//             if bs1_pts.0.id == *pt_id {
//                 //
//             }
//         }
//     }
//     (bs1_pts, bs2_pts)
// }

pub fn _pos_to_polar(pos1: &Point, pos2: &Point) -> (Point, f64) {
    let (x1, y1) = (pos1.x, pos1.y);
    let (x2, y2) = (pos2.x, pos2.y);

    let angle = (y2 - y1).atan2(x2 - x1);
    let rho = if x2 != x1 {
        let b = (y1 * x2 - x1 * y2) / (x2 - x1);
        let m = (y2 - y1) / (x2 - x1);
        Point::new(-m / (m * m + 1.) * b, b / (m * m + 1.))
    } else {
        Point::new(0., 0.)
    };

    (rho, angle)
}
// pub fn snap_to_snap_grid(pos: &Point, snap_distance: f64) -> Point {
//     (*pos / snap_distance).round() * snap_distance
// }
//     magnet_geometry(&br, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     tl = p;
//     if tl.x >= br.x {
//         tl.x = br.x - self.snap_distance;
//     }
//     if tl.y >= br.y {
//         tl.y = br.y - self.snap_distance;
//     }
//     tr.y = tl.y;
//     bl.x = tl.x;
// }
// TopRight => {
//     magnet_geometry(&bl, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     tr = p;
//     if tr.x <= bl.x {
//         tr.x = bl.x + self.snap_distance;
//     }
//     if tr.y >= bl.y {
//         tr.y = bl.y - self.snap_distance;
//     }
//     tl.y = tr.y;
//     br.x = tr.x;
// }
// BottomRight => {
//     magnet_geometry(&tl, &mut p, self.snap_distance);
//     snap_to_snap_grid(&mut p, self.snap_distance);
//     br = p;
//     if br.x <= tl.x {
//         br.x = tl.x + self.snap_distance;
//     }
//     if br.y <= tl.y {
//         br.y = tl.y + self.snap_distance;
//     }
//     tr.x = br.x;
//     bl.y = br.y;

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
//                     (Handle::StartAngle, h_start_angle.addxy(center.1.x, 0.)),
//                     (Handle::EndAngle, h_min_angle.addxy(center.1.x, 0.)),
//                     (rotation, start_angle, min_angle),
//                 ),
//                 SimpleShape::Ellipse(
//                     (Handle::Center, center.1.clone()),
//                     (Handle::End, center.1.clone() + radius.1.clone()),
//                     (Handle::StartAngle, h_max_angle.addxy(center.1.x, 0.)),
//                     (Handle::EndAngle, h_end_angle.addxy(center.1.x, 0.)),
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

// pub fn _get_point_on_cubic_bezier(
//     t: f64,
//     start: &Point,
//     ctrl1: &Point,
//     ctrl2: &Point,
//     end: &Point,
// ) -> Point {
// let u = 1.0 - t;
// let tt = t * t;
// let uu = u * u;
// let uuu = uu * u;
// let ttt = tt * t;
// let mut result = *start * uuu; // (1-t)^3 * start
// result += *ctrl1 * 3.0 * uu * t; // 3(1-t)^2 * t * ctrl1
// result += *ctrl2 * 3.0 * u * tt; // 3(1-t) * t^2 * ctrl2
// result += *end * ttt; // t^3 * end
// result
//TODO
//     Point::ZERO
// }

pub fn _get_point_from_angle(radius: &Point, angle: f64) -> Point {
    let x = radius.x.abs() * angle.cos();
    let y = radius.y.abs() * angle.sin();
    Point { x: x, y: y }
}

#[inline]
pub fn _switch_wx(point1: &mut Point, point2: &mut Point) {
    let pos = point1.x;
    point1.x = point2.x;
    point2.x = pos;
}
#[inline]
pub fn _order_pos_x(x_inf: &mut Point, x_sup: &mut Point) {
    if x_inf.x > x_sup.x {
        let tmp = x_inf.x;
        x_inf.x = x_sup.x;
        x_sup.x = tmp;
    }
}
#[inline]
pub fn _order_pos_y(x_inf: &mut Point, x_sup: &mut Point) {
    if x_inf.y > x_sup.y {
        let tmp = x_inf.y;
        x_inf.y = x_sup.y;
        x_sup.y = tmp;
    }
}

#[inline]
pub fn _get_atan2(point: &Point) -> f64 {
    point.y.atan2(point.x)
}

// #[inline]
// #[allow(dead_code)]
// fn find_t_for_point_on_quad_bezier(p: &Point, start: &Point, ctrl: &Point, end: &Point) -> Option<f64> {
//     let mut t_min = 0.0;
//     let mut t_max = 1.0;
//     for _ in 0..MAX_ITERATIONS {
//         let t_mid = (t_min + t_max) / 2.0;
//         let mid_point = get_point_on_quad_bezier(t_mid, start, ctrl, end);
//         let dist = mid_point.dist(p);
//         if dist < EPSILON {
//             return Some(t_mid);
//         }
//         if get_point_on_quad_bezier((t_min + t_mid) / 2.0, start, ctrl, end).dist(p) < dist {
//             t_max = t_mid;
//         } else {
//             t_min = t_mid;
//         }
//     }
//     None
// }

// #[allow(dead_code)]
// fn find_t_for_point_on_cubic_bezier(
//     p: &Point,
//     start: &Point,
//     ctrl1: &Point,
//     ctrl2: &Point,
//     end: &Point,
// ) -> Option<f64> {
// let mut t_min = 0.0;
// let mut t_max = 1.0;
// for _ in 0..MAX_ITERATIONS {
//     let t_mid = (t_min + t_max) / 2.0;
//     let mid_point = get_point_on_cubic_bezier(t_mid, start, ctrl1, ctrl2, end);
//     let dist = mid_point.dist(p);
//     if dist < EPSILON {
//         return Some(t_mid);
//     }
//     if get_point_on_cubic_bezier((t_min + t_mid) / 2.0, start, ctrl1, ctrl2, end).dist(p) < dist
//     {
//         t_max = t_mid;
//     } else {
//         t_min = t_mid;
//     }
// }
//TODO
//     None
// }

pub fn to_canvas(pt: &Point, scale: f64, offset: &Point) -> Point {
    Point {
        x: (pt.x * scale) + offset.x,
        y: (pt.y * scale) + offset.y,
    }
}
pub fn to_world(pt: &Point, scale: f64, offset: &Point) -> Point {
    Point {
        x: (pt.x - offset.x) / scale,
        y: (pt.y - offset.y) / scale,
    }
}

use std::collections::HashMap;

// fn main() {
//     // Create a HashMap where Ids instances are keys.
//     let mut map = HashMap::new();

//     // Create instances of Ids.
//     let id1 = Ids(1, 2);
//     let id2 = Ids(3, 4);

//     // Insert values into the map using Ids as keys.
//     map.insert(id1, "Value associated with id1");
//     map.insert(id2, "Value associated with id2");

//     // Accessing values in the map.
//     let lookup_id = Ids(1, 2);
//     if let Some(value) = map.get(&lookup_id) {
//         println!("Found: {}", value);
//     } else {
//         println!("No value found for the given key.");
//     }
// }
