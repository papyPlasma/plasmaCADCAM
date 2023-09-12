use js_sys::Math::sqrt;

use crate::shapes::{Segment, XY};

pub enum SegmentSnapping {
    None,
    Horizontal,
    Vertical,
    Diagonal45,
    Diagonal135,
}

pub fn snap_to_grid(pos: &XY, grid_spacing: f64) -> XY {
    XY {
        x: (pos.x / grid_spacing).round() * grid_spacing,
        y: (pos.y / grid_spacing).round() * grid_spacing,
    }
}

fn snap45(seg: &mut Segment, snap_segment_end: bool) -> bool {
    let mut dy = seg.end.y - seg.start.y;
    let dx = seg.end.x - seg.start.x;
    let m = dy / dx;
    if m > 0.95 && m < (1. / 0.95) {
        dy = dx;
        if snap_segment_end {
            seg.end.x = seg.start.x + dx;
            seg.end.y = seg.start.y + dy;
        } else {
            seg.start.x = seg.end.x - dx;
            seg.start.y = seg.end.y - dy;
        }
        true
    } else {
        false
    }
}

fn snap135(seg: &mut Segment, snap_segment_end: bool) -> bool {
    let mut dy = seg.end.y - seg.start.y;
    let dx = seg.end.x - seg.start.x;
    let m = dy / dx;
    if m < -0.95 && m > -(1. / 0.95) {
        dy = -dx;
        if snap_segment_end {
            seg.end.x = seg.start.x + dx;
            seg.end.y = seg.start.y + dy;
        } else {
            seg.start.x = seg.end.x - dx;
            seg.start.y = seg.end.y - dy;
        }
        true
    } else {
        false
    }
}

pub fn snap_segment(seg: &mut Segment, snap_segment_end: bool, snap_val: f64) -> SegmentSnapping {
    // Horizontal
    if (seg.start.y - seg.end.y).abs() < snap_val {
        if snap_segment_end {
            seg.end.y = seg.start.y;
        } else {
            seg.start.y = seg.end.y;
        }
        return SegmentSnapping::Horizontal;
    } else {
        if (seg.start.x - seg.end.x).abs() < snap_val {
            if snap_segment_end {
                seg.end.x = seg.start.x;
            } else {
                seg.start.x = seg.end.x;
            }
            return SegmentSnapping::Vertical;
        } else {
            if snap45(seg, snap_segment_end) {
                return SegmentSnapping::Diagonal45;
            } else {
                if snap135(seg, snap_segment_end) {
                    return SegmentSnapping::Diagonal135;
                } else {
                    return SegmentSnapping::None;
                }
            }
        }
    }
}

pub fn is_point_on_point(pt1: &XY, pt2: &XY, precision: f64) -> bool {
    let dx = (pt1.x - pt2.x).abs();
    let dy = (pt1.y - pt2.y).abs();
    dx < precision && dy < precision
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
