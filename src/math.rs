use crate::shapes::ConstructionType;
use std::{
    f64::consts::PI,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

pub fn magnet(pt1: &WXY, pt2: &mut WXY, pt: &WXY, snap_distance: f64) -> bool {
    let dx = (pt.wx - pt1.wx).abs();
    let dy = (pt.wy - pt1.wy).abs();

    if dy < 4. * snap_distance {
        pt2.wy = pt1.wy;
        pt2.wx = pt.wx;
        true
    } else {
        if dx < 4. * snap_distance {
            pt2.wx = pt1.wx;
            pt2.wy = pt.wy;
            true
        } else {
            let x1 = pt1.wx;
            let y1 = pt1.wy;
            let x2 = pt.wx;
            let y2 = pt.wy;
            // Projection of p on (pt1, m=1)
            let p_proj = WXY {
                wx: (x1 + x2 + y2 - y1) / 2.,
                wy: (-x1 + x2 + y2 + y1) / 2.,
            };
            if p_proj.dist(pt) < 4. * snap_distance {
                *pt2 = p_proj;
                true
            } else {
                // Projection of p on (pt1, m=-1)
                let p_proj = WXY {
                    wx: (x1 + x2 - y2 + y1) / 2.,
                    wy: (x1 - x2 + y2 + y1) / 2.,
                };

                if p_proj.dist(pt) < 4. * snap_distance {
                    *pt2 = p_proj;
                    true
                } else {
                    *pt2 = *pt;
                    false
                }
            }
        }
    }
}

fn is_vert(pt1: &WXY, pt2: &WXY) -> bool {
    (pt1.wx - pt2.wx).abs() < 0.001
}
fn is_hori(pt1: &WXY, pt2: &WXY) -> bool {
    (pt1.wy - pt2.wy).abs() < 0.001
}
fn is_45_135(pt1: &WXY, pt2: &WXY) -> bool {
    let dy = pt2.wy - pt1.wy;
    let dx = pt2.wx - pt1.wx;
    if dx != 0. {
        (dy / dx).abs() > 1. / 1.01 && (dy / dx).abs() < 1.01
    } else {
        false
    }
}

pub fn push_45_135(pt1: &WXY, pt2: &WXY, full: bool, cst: &mut Vec<ConstructionType>) {
    if is_45_135(pt1, pt2) {
        if full {
            use ConstructionType::*;
            cst.push(Move(*pt1));
            cst.push(Line(WXY {
                wx: 2. * pt1.wx - pt2.wx,
                wy: 2. * pt1.wy - pt2.wy,
            }));
            cst.push(Move(*pt1));
            cst.push(Line(*pt2));
            cst.push(Line(WXY {
                wx: 2. * pt2.wx - pt1.wx,
                wy: 2. * pt2.wy - pt1.wy,
            }));
        } else {
            use ConstructionType::*;
            cst.push(Move(*pt1));
            cst.push(Line(WXY {
                wx: 2. * pt1.wx - pt2.wx,
                wy: 2. * pt1.wy - pt2.wy,
            }));
            cst.push(Move(*pt2));
            cst.push(Line(WXY {
                wx: 2. * pt2.wx - pt1.wx,
                wy: 2. * pt2.wy - pt1.wy,
            }));
        }
    }
}
pub fn push_vertical(pt1: &WXY, pt2: &WXY, full: bool, cst: &mut Vec<ConstructionType>) {
    use ConstructionType::*;
    if is_vert(pt1, pt2) {
        if full {
            cst.push(Move(*pt1));
            cst.push(Line(WXY {
                wx: pt1.wx,
                wy: 2. * pt1.wy - pt2.wy,
            }));
            cst.push(Move(*pt1));
            cst.push(Line(*pt2));
            cst.push(Line(WXY {
                wx: pt1.wx,
                wy: 2. * pt2.wy - pt1.wy,
            }));
        } else {
            cst.push(Move(*pt1));
            cst.push(Line(WXY {
                wx: pt1.wx,
                wy: 2. * pt1.wy - pt2.wy,
            }));
            cst.push(Move(*pt2));
            cst.push(Line(WXY {
                wx: pt1.wx,
                wy: 2. * pt2.wy - pt1.wy,
            }));
        }
    }
}
pub fn push_horizontal(pt1: &WXY, pt2: &WXY, full: bool, cst: &mut Vec<ConstructionType>) {
    use ConstructionType::*;
    if is_hori(pt1, pt2) {
        if full {
            cst.push(Move(*pt1));
            cst.push(Line(WXY {
                wx: 2. * pt1.wx - pt2.wx,
                wy: pt1.wy,
            }));
            cst.push(Move(*pt1));
            cst.push(Line(*pt2));
            cst.push(Line(WXY {
                wx: 2. * pt2.wx - pt1.wx,
                wy: pt1.wy,
            }));
        } else {
            cst.push(Move(*pt1));
            cst.push(Line(WXY {
                wx: 2. * pt1.wx - pt2.wx,
                wy: pt1.wy,
            }));
            cst.push(Move(*pt2));
            cst.push(Line(WXY {
                wx: 2. * pt2.wx - pt1.wx,
                wy: pt1.wy,
            }));
        }
    }
}
pub fn push_handle(pt: &WXY, size_handle: &WXY, fill: bool, cst: &mut Vec<ConstructionType>) {
    let radius = *size_handle / 2.;
    use ConstructionType::*;
    cst.push(Move(
        *pt + WXY {
            wx: radius.wx,
            wy: 0.,
        },
    ));
    cst.push(Ellipse(*pt, radius, 0., 0., 2. * PI, fill));
}

pub fn snap_to_snap_grid(pos: &mut WXY, snap_distance: f64) {
    pos.wx = (pos.wx / snap_distance).round() * snap_distance;
    pos.wy = (pos.wy / snap_distance).round() * snap_distance;
}
pub fn snap_to_snap_grid_y(pos: &mut WXY, grid_spacing: f64) {
    pos.wy = (pos.wy / grid_spacing).round() * grid_spacing;
}
pub fn snap_to_snap_grid_x(pos: &mut WXY, grid_spacing: f64) {
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

pub fn is_point_on_point(pt1: &WXY, pt2: &WXY, precision: f64) -> bool {
    pt1.dist(pt2) < precision
}

pub fn is_point_on_segment(pt: &WXY, pt1: &WXY, pt2: &WXY, precision: f64) -> bool {
    let denominator = ((pt2.wy - pt1.wy).powf(2.) + (pt2.wx - pt1.wx).powf(2.)).sqrt();
    if denominator == 0. {
        return is_point_on_point(pt, pt1, precision);
    }
    let numerator = ((pt2.wy - pt1.wy) * pt.wx - (pt2.wx - pt1.wx) * pt.wy + pt2.wx * pt1.wy
        - pt2.wy * pt1.wx)
        .abs();

    if numerator / denominator > precision {
        return false;
    }
    is_between(pt, pt1, pt2)
}

pub fn is_point_on_quadbezier(pt: &WXY, pt1: &WXY, ctrl: &WXY, pt2: &WXY, precision: f64) -> bool {
    let mut t_min = 0.;
    let mut t_max = 1.;
    let mut min_dist = f64::MAX;

    for _i in 0..100 {
        // max iterations can be adjusted
        let t_mid = (t_min + t_max) / 2.;

        let bt = WXY {
            wx: (1f64 - t_mid).powf(2.) * pt1.wx
                + 2. * (1f64 - t_mid) * t_mid * ctrl.wx
                + t_mid.powf(2.) * pt2.wx,
            wy: (1f64 - t_mid).powf(2.) * pt1.wy
                + 2. * (1f64 - t_mid) * t_mid * ctrl.wy
                + t_mid.powf(2.) * pt2.wy,
        };

        let dist = bt.dist(pt);

        if dist < min_dist {
            min_dist = dist;
        }

        if dist < precision {
            return true; // We found a sufficiently close point
        }

        // Using gradient to decide the next tMid for the next iteration.
        let gradient = (bt.wx - pt.wx) * (pt2.wx - pt1.wx) + (bt.wy - pt.wy) * (pt2.wy - pt1.wy);

        if gradient > 0. {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }
    min_dist <= precision
}

pub fn is_point_on_cubicbezier(
    pt: &WXY,
    pt1: &WXY,
    ctrl1: &WXY,
    ctrl2: &WXY,
    pt2: &WXY,
    precision: f64,
) -> bool {
    let mut t_min = 0.;
    let mut t_max = 1.;
    let mut min_dist = f64::MAX;

    for _i in 0..100 {
        let t_mid = (t_min + t_max) / 2.;

        let bt = WXY {
            wx: (1f64 - t_mid).powf(3.) * pt1.wx
                + 3. * (1f64 - t_mid).powf(2.) * t_mid * ctrl1.wx
                + 3. * (1f64 - t_mid) * t_mid.powf(2.) * ctrl2.wx
                + (t_mid).powf(3.) * pt2.wx,
            wy: (1f64 - t_mid).powf(3.) * pt1.wy
                + 3. * (1f64 - t_mid).powf(2.) * t_mid * ctrl1.wy
                + 3. * (1f64 - t_mid) * t_mid.powf(2.) * ctrl2.wy
                + (t_mid).powf(3.) * pt2.wy,
        };

        let dist = bt.dist(pt);

        if dist < min_dist {
            min_dist = dist;
        }

        if dist < precision {
            return true; // We found a sufficiently close point
        }

        // Using gradient to decide the next tMid for the next iteration.
        let gradient = (bt.wx - pt.wx) * (pt2.wx - pt1.wx) + (bt.wy - pt.wy) * (pt2.wy - pt1.wy);

        if gradient > 0. {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }
    min_dist <= precision
}

pub fn is_box_inside(box_outer: &[WXY; 2], box_inner: &[WXY; 2]) -> bool {
    let bl_outer = box_outer[0];
    let tr_outer = box_outer[1];
    let bl_inner = box_inner[0];
    let tr_inner = box_inner[1];
    bl_inner.wx >= bl_outer.wx
        && bl_inner.wy >= bl_outer.wy
        && tr_inner.wx <= tr_outer.wx
        && tr_inner.wy <= tr_outer.wy
}

pub fn is_point_on_ellipse(pt: &WXY, center: &WXY, radius: &WXY, mut precision: f64) -> bool {
    // if radius.wx > 0. && radius.wy > 0. {
    precision /= radius.norm();
    precision *= 2.;
    let value = (pt.wx - center.wx).powf(2.) / (radius.wx * radius.wx)
        + (pt.wy - center.wy).powf(2.) / (radius.wy * radius.wy);
    value < 1. + precision && value > 1. - precision
    // } else {
    //     false
    // }
}

pub fn reorder_corners(bb: &mut [WXY; 2]) {
    let pt1 = bb[0];
    let pt2 = bb[1];
    if pt1.wx < pt2.wx {
        if pt1.wy < pt2.wy {
            let bl = WXY {
                wx: pt1.wx,
                wy: pt1.wy,
            };
            let tr = WXY {
                wx: pt2.wx,
                wy: pt2.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        } else {
            let bl = WXY {
                wx: pt1.wx,
                wy: pt2.wy,
            };
            let tr = WXY {
                wx: pt2.wx,
                wy: pt1.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        }
    } else {
        if pt1.wy < pt2.wy {
            let bl = WXY {
                wx: pt2.wx,
                wy: pt1.wy,
            };
            let tr = WXY {
                wx: pt1.wx,
                wy: pt2.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        } else {
            let bl = WXY {
                wx: pt2.wx,
                wy: pt2.wy,
            };
            let tr = WXY {
                wx: pt1.wx,
                wy: pt1.wy,
            };
            bb[0] = bl;
            bb[1] = tr;
        }
    }
}
fn is_between(pt: &WXY, pt1: &WXY, pt2: &WXY) -> bool {
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

#[derive(Copy, Clone, Debug)]
pub struct WXY {
    pub wx: f64,
    pub wy: f64,
}
impl WXY {
    pub fn to_canvas(&self, scale: f64, offset: CXY) -> CXY {
        let canvas_x = (self.wx * scale) + offset.cx;
        let canvas_y = (self.wy * scale) + offset.cy;
        CXY {
            cx: canvas_x,
            cy: canvas_y,
        }
    }
    pub fn add_x(&self, wx: f64) -> WXY {
        WXY {
            wx: self.wx + wx,
            wy: self.wy,
        }
    }
    pub fn add_y(&self, wy: f64) -> WXY {
        WXY {
            wx: self.wx,
            wy: self.wy + wy,
        }
    }

    pub fn dist(&self, other: &WXY) -> f64 {
        let dpt = *self - *other;
        (dpt.wx * dpt.wx + dpt.wy * dpt.wy).sqrt()
    }
    #[allow(dead_code)]
    pub fn norm(&self) -> f64 {
        (self.wx * self.wx + self.wy * self.wy).sqrt()
    }
}
impl Default for WXY {
    fn default() -> Self {
        WXY { wx: 0.0, wy: 0.0 }
    }
}
impl Add for WXY {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            wx: self.wx + other.wx,
            wy: self.wy + other.wy,
        }
    }
}
impl Add<f64> for WXY {
    type Output = WXY;
    fn add(self, scalar: f64) -> Self::Output {
        WXY {
            wx: self.wx + scalar,
            wy: self.wy + scalar,
        }
    }
}
impl AddAssign for WXY {
    fn add_assign(&mut self, other: WXY) {
        self.wx += other.wx;
        self.wy += other.wy;
    }
}
impl AddAssign<f64> for WXY {
    fn add_assign(&mut self, scalar: f64) {
        self.wx += scalar;
        self.wy += scalar;
    }
}
impl Sub for WXY {
    type Output = WXY;
    fn sub(self, other: WXY) -> WXY {
        WXY {
            wx: self.wx - other.wx,
            wy: self.wy - other.wy,
        }
    }
}
impl SubAssign for WXY {
    fn sub_assign(&mut self, other: WXY) {
        self.wx -= other.wx;
        self.wy -= other.wy;
    }
}
impl Div<f64> for WXY {
    type Output = WXY;

    fn div(self, rhs: f64) -> Self::Output {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        WXY {
            wx: self.wx / rhs,
            wy: self.wy / rhs,
        }
    }
}
impl DivAssign<f64> for WXY {
    fn div_assign(&mut self, rhs: f64) {
        if rhs == 0.0 {
            panic!("Division by zero");
        }
        self.wx /= rhs;
        self.wy /= rhs;
    }
}
impl Mul<f64> for WXY {
    type Output = WXY;

    fn mul(self, rhs: f64) -> Self::Output {
        WXY {
            wx: self.wx * rhs,
            wy: self.wy * rhs,
        }
    }
}
impl MulAssign<f64> for WXY {
    fn mul_assign(&mut self, rhs: f64) {
        self.wx *= rhs;
        self.wy *= rhs;
    }
}
impl PartialEq for WXY {
    fn eq(&self, other: &Self) -> bool {
        self.wx == other.wx && self.wy == other.wy
    }
}
impl Eq for WXY {}

#[derive(Copy, Clone, Debug)]
pub struct CXY {
    pub cx: f64,
    pub cy: f64,
}
impl CXY {
    pub fn to_world(&self, scale: f64, offset: CXY) -> WXY {
        let world_x = (self.cx - offset.cx) / scale;
        let world_y = (self.cy - offset.cy) / scale;
        WXY {
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
