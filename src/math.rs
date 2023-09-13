use std::f64::consts::PI;

use js_sys::Math::sqrt;
// use web_sys::console;

use crate::shapes::{SegmentSnapping, XY};

pub fn snap_to_grid(pos: &mut XY, grid_spacing: f64) {
    pos.x = (pos.x / grid_spacing).round() * grid_spacing;
    pos.y = (pos.y / grid_spacing).round() * grid_spacing;
}

pub fn snap_h_v_45_135(
    handles: &mut Vec<XY>,
    idx1: &usize,
    idx2: &usize,
    snap_to_end: bool,
    snap_val: f64,
) -> SegmentSnapping {
    let mut start = handles[*idx1];
    let mut end = handles[*idx2];
    // Horizontal
    if (start.y - end.y).abs() < snap_val {
        if snap_to_end {
            end.y = start.y;
        } else {
            start.y = end.y;
        }
        *handles.get_mut(*idx1).unwrap() = start;
        *handles.get_mut(*idx2).unwrap() = end;
        return SegmentSnapping::Horizontal;
    } else {
        if (start.x - end.x).abs() < snap_val {
            if snap_to_end {
                end.x = start.x;
            } else {
                start.x = end.x;
            }
            *handles.get_mut(*idx1).unwrap() = start;
            *handles.get_mut(*idx2).unwrap() = end;
            return SegmentSnapping::Vertical;
        } else {
            if snap45(&mut start, &mut end, snap_to_end) {
                *handles.get_mut(*idx1).unwrap() = start;
                *handles.get_mut(*idx2).unwrap() = end;
                return SegmentSnapping::Diagonal45;
            } else {
                if snap135(&mut start, &mut end, snap_to_end) {
                    *handles.get_mut(*idx1).unwrap() = start;
                    *handles.get_mut(*idx2).unwrap() = end;
                    return SegmentSnapping::Diagonal135;
                } else {
                    return SegmentSnapping::None;
                }
            }
        }
    }
}

pub fn snap_equidistant(
    handles: &mut Vec<XY>,
    idx: &usize,
    idxs: &[usize; 2],
    snap_val: f64,
) -> SegmentSnapping {
    let pt = handles[*idx];
    let pt1 = handles[idxs[0]];
    let pt2 = handles[idxs[1]];

    let mid = (pt1 + pt2) / 2.0;
    let dx = pt2.x - pt1.x;
    let dy = pt2.y - pt1.y;

    if dx == 0. && dy == 0. {
        return SegmentSnapping::None;
    }

    let proj = if dx == 0. {
        XY {
            x: pt.x,
            y: (pt2.y + pt1.y) / 2.,
        }
    } else {
        if dy == 0. {
            XY {
                x: (pt2.x + pt1.x) / 2.,
                y: pt.y,
            }
        } else {
            let slope = dy / dx;
            let perp_slope = -1. / slope;
            let x_p = (perp_slope * mid.x - slope * pt.x + pt.y - mid.y) / (perp_slope - slope);
            let y_p = perp_slope * (x_p - mid.x) + mid.y;
            XY { x: x_p, y: y_p }
        }
    };

    if pt.dist(&proj) < snap_val {
        handles[*idx] = proj;
        SegmentSnapping::Middle
    } else {
        SegmentSnapping::None
    }
}

pub fn is_point_on_point(pt1: &XY, pt2: &XY, precision: f64) -> bool {
    // let dx = (pt1.x - pt2.x).abs();
    // let dy = (pt1.y - pt2.y).abs();
    // dx < precision && dy < precision
    pt1.dist(pt2) < precision
}

pub fn is_point_on_segment(pt: &XY, pt1: &XY, pt2: &XY, precision: f64) -> bool {
    let denominator = sqrt((pt2.y - pt1.y).powf(2.) + (pt2.x - pt1.x).powf(2.));
    if denominator == 0. {
        return is_point_on_point(pt, pt1, precision);
    }
    let numerator =
        ((pt2.y - pt1.y) * pt.x - (pt2.x - pt1.x) * pt.y + pt2.x * pt1.y - pt2.y * pt1.x).abs();

    if numerator / denominator > precision {
        return false;
    }
    is_between(pt, pt1, pt2)
}

pub fn is_point_on_quadbezier(pt: &XY, pt1: &XY, ctrl: &XY, pt2: &XY, precision: f64) -> bool {
    let mut t_min = 0.;
    let mut t_max = 1.;
    let mut min_dist = f64::MAX;

    for _i in 0..100 {
        // max iterations can be adjusted
        let t_mid = (t_min + t_max) / 2.;

        let bt = XY {
            x: (1f64 - t_mid).powf(2.) * pt1.x
                + 2. * (1f64 - t_mid) * t_mid * ctrl.x
                + t_mid.powf(2.) * pt2.x,
            y: (1f64 - t_mid).powf(2.) * pt1.y
                + 2. * (1f64 - t_mid) * t_mid * ctrl.y
                + t_mid.powf(2.) * pt2.y,
        };

        let dist = bt.dist(pt);

        if dist < min_dist {
            min_dist = dist;
        }

        if dist < precision {
            return true; // We found a sufficiently close point
        }

        // Using gradient to decide the next tMid for the next iteration.
        let gradient = (bt.x - pt.x) * (pt2.x - pt1.x) + (bt.y - pt.y) * (pt2.y - pt1.y);

        if gradient > 0. {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }
    min_dist <= precision
}

pub fn is_point_on_cubicbezier(
    pt: &XY,
    pt1: &XY,
    ctrl1: &XY,
    ctrl2: &XY,
    pt2: &XY,
    precision: f64,
) -> bool {
    let mut t_min = 0.;
    let mut t_max = 1.;
    let mut min_dist = f64::MAX;

    for _i in 0..100 {
        let t_mid = (t_min + t_max) / 2.;

        let bt = XY {
            x: (1f64 - t_mid).powf(3.) * pt1.x
                + 3. * (1f64 - t_mid).powf(2.) * t_mid * ctrl1.x
                + 3. * (1f64 - t_mid) * t_mid.powf(2.) * ctrl2.x
                + (t_mid).powf(3.) * pt2.x,
            y: (1f64 - t_mid).powf(3.) * pt1.y
                + 3. * (1f64 - t_mid).powf(2.) * t_mid * ctrl1.y
                + 3. * (1f64 - t_mid) * t_mid.powf(2.) * ctrl2.y
                + (t_mid).powf(3.) * pt2.y,
        };

        let dist = bt.dist(pt);

        if dist < min_dist {
            min_dist = dist;
        }

        if dist < precision {
            return true; // We found a sufficiently close point
        }

        // Using gradient to decide the next tMid for the next iteration.
        let gradient = (bt.x - pt.x) * (pt2.x - pt1.x) + (bt.y - pt.y) * (pt2.y - pt1.y);

        if gradient > 0. {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }
    min_dist <= precision
}

pub fn is_box_inside(box_outer: &[XY; 2], box_inner: &[XY; 2]) -> bool {
    let bl_outer = box_outer[0];
    let tr_outer = box_outer[1];
    let bl_inner = box_inner[0];
    let tr_inner = box_inner[1];
    bl_inner.x >= bl_outer.x
        && bl_inner.y >= bl_outer.y
        && tr_inner.x <= tr_outer.x
        && tr_inner.y <= tr_outer.y
}
fn _normalize_angle(mut angle: f64) -> f64 {
    while angle < 0. {
        angle += 2. * PI;
    }
    while angle >= 2. * PI {
        angle -= 2. * PI;
    }
    angle
}

pub fn is_point_on_ellipse(pt: &XY, c: &XY, r: &XY, mut precision: f64) -> bool {
    if r.x > 0. && r.y > 0. {
        precision /= r.norm();
        precision *= 2.;
        let value = (pt.x - c.x).powf(2.) / (r.x * r.x) + (pt.y - c.y).powf(2.) / (r.y * r.y);
        value < 1. + precision && value > 1. - precision
    } else {
        false
    }
}

pub fn get_segment(pta: &XY, ptb: &XY, segment_snapping: SegmentSnapping) -> Option<(XY, XY)> {
    use SegmentSnapping::*;
    match segment_snapping {
        SegmentSnapping::None => Option::None,
        Horizontal => {
            let (mut start, mut end) = if pta.x < ptb.x {
                (*pta, *ptb)
            } else {
                (*ptb, *pta)
            };
            start.x -= 100.;
            end.x += 100.;
            Some((start.clone(), end.clone()))
        }
        Vertical => {
            let (mut start, mut end) = if pta.y < ptb.y {
                (*pta, *ptb)
            } else {
                (*ptb, *pta)
            };
            start.y -= 100.;
            end.y += 100.;
            Some((start.clone(), end.clone()))
        }
        Diagonal45 => {
            let (mut start, mut end) = if pta.x < ptb.x {
                (*pta, *ptb)
            } else {
                (*ptb, *pta)
            };
            start.x -= 100.;
            start.y -= 100.;
            end.x += 100.;
            end.y += 100.;
            Some((start.clone(), end.clone()))
        }
        Diagonal135 => {
            let (mut start, mut end) = if pta.x < ptb.x {
                (*pta, *ptb)
            } else {
                (*ptb, *pta)
            };
            start.x -= 100.;
            start.y += 100.;
            end.x += 100.;
            end.y -= 100.;
            Some((start.clone(), end.clone()))
        }
        Middle => Option::None,
    }
}

pub fn reorder_corners(bb: &[XY; 2]) -> [XY; 2] {
    let pt1 = bb[0];
    let pt2 = bb[1];
    if pt1.x < pt2.x {
        if pt1.y < pt2.y {
            let bl = XY { x: pt1.x, y: pt1.y };
            let tr = XY { x: pt2.x, y: pt2.y };
            [bl, tr]
        } else {
            let bl = XY { x: pt1.x, y: pt2.y };
            let tr = XY { x: pt2.x, y: pt1.y };
            [bl, tr]
        }
    } else {
        if pt1.y < pt2.y {
            let bl = XY { x: pt2.x, y: pt1.y };
            let tr = XY { x: pt1.x, y: pt2.y };
            [bl, tr]
        } else {
            let bl = XY { x: pt2.x, y: pt2.y };
            let tr = XY { x: pt1.x, y: pt1.y };
            [bl, tr]
        }
    }
}
fn is_between(pt: &XY, pt1: &XY, pt2: &XY) -> bool {
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

fn snap45(start: &mut XY, end: &mut XY, snap_to_end: bool) -> bool {
    let mut dy = end.y - start.y;
    let dx = end.x - start.x;
    let m = dy / dx;
    if m > 0.95 && m < (1. / 0.95) {
        dy = dx;
        if snap_to_end {
            end.x = start.x + dx;
            end.y = start.y + dy;
        } else {
            start.x = end.x - dx;
            start.y = end.y - dy;
        }
        true
    } else {
        false
    }
}

fn snap135(start: &mut XY, end: &mut XY, snap_to_end: bool) -> bool {
    let mut dy = end.y - start.y;
    let dx = end.x - start.x;
    let m = dy / dx;
    if m < -0.95 && m > -(1. / 0.95) {
        dy = -dx;
        if snap_to_end {
            end.x = start.x + dx;
            end.y = start.y + dy;
        } else {
            start.x = end.x - dx;
            start.y = end.y - dy;
        }
        true
    } else {
        false
    }
}
