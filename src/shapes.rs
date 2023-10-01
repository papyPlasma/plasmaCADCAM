// use web_sys::console;

// use web_sys::console;

use crate::math::*;
use std::collections::{HashMap, HashSet};
// use std::ops::{Deref, DerefMut};
use std::{
    f64::consts::PI,
    sync::atomic::{AtomicUsize, Ordering},
};

// #[allow(dead_code)]
// #[derive(Clone)]
// pub struct AllShapes {
//     shapes: HashMap<usize, Shape>,
// }
// impl AllShapes {
//     pub fn new() -> AllShapes {
//         AllShapes {
//             shapes: HashMap::new(),
//         }
//     }
// }
// impl Deref for AllShapes {
//     type Target = HashMap<usize, Shape>;
//     fn deref(&self) -> &Self::Target {
//         &self.shapes
//     }
// }
// impl DerefMut for AllShapes {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.shapes
//     }
// }

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct ShapeParameters {
    pub grab_handle_precision: f64,
    pub handles_size: f64,
    pub highlight_size: f64,
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum LayerType {
    Worksheet,
    Dimension,
    GeometryHelpers,
    Origin,
    Grid,
    SelectionTool,
    Selected,
    Highlight,
    Handle(bool),
}

#[allow(dead_code)]
#[derive(Clone)]
pub enum ConstructionType {
    Layer(LayerType),
    Move(WPoint),
    Line(WPoint),
    QuadBezier(WPoint, WPoint),
    CubicBezier(WPoint, WPoint, WPoint),
    Ellipse(WPoint, WPoint, f64, f64, f64, bool),
    Rectangle(WPoint, WPoint, bool),
    Text(WPoint, String),
}

#[derive(Copy, Clone)]
pub enum ShapeType {
    Line,
    QuadBezier,
    CubicBezier,
    Rectangle,
    Ellipse,
    Group,
}

#[derive(Copy, Clone)]
pub struct Line;
impl Line {
    pub fn new(
        data_pool: &mut DataPool,
        start_point: &WPoint,
        end_point: &WPoint,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> usize {
        let end_point = if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
            *start_point + snap_distance
        } else {
            *end_point
        };
        let start_id = data_pool.insert_point(*start_point);
        let end_id = data_pool.insert_point(end_point);
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Line,
            pts_ids: vec![start_id, end_id],
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            selected: false,
            tmp: WPoint::default(),
            init: true,
        };
        data_pool.insert_shape(shape)
    }
}
#[derive(Copy, Clone)]
pub struct QuadBezier;
impl QuadBezier {
    pub fn new(
        data_pool: &mut DataPool,
        start_point: &WPoint,
        ctrl_point: &WPoint,
        end_point: &WPoint,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> usize {
        let (end_point, ctrl_point) =
            if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
                (
                    *start_point + 2. * snap_distance,
                    *start_point + snap_distance,
                )
            } else {
                (*end_point, *ctrl_point)
            };
        let start_id = data_pool.insert_point(*start_point);
        let ctrl_id = data_pool.insert_point(ctrl_point);
        let end_id = data_pool.insert_point(end_point);
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::QuadBezier,
            pts_ids: vec![start_id, ctrl_id, end_id],
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            selected: false,
            tmp: WPoint::default(),
            init: true,
        };
        data_pool.insert_shape(shape)
    }
}
#[derive(Copy, Clone)]
pub struct CubicBezier;
impl CubicBezier {
    pub fn new(
        data_pool: &mut DataPool,
        start_point: &WPoint,
        ctrl1_point: &WPoint,
        ctrl2_point: &WPoint,
        end_point: &WPoint,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> usize {
        let (end_point, ctrl1_point, ctrl2_point) =
            if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
                (
                    *start_point + 3. * snap_distance,
                    *start_point + snap_distance,
                    *start_point + 2. * snap_distance,
                )
            } else {
                (*end_point, *ctrl1_point, *ctrl2_point)
            };
        let start_id = data_pool.insert_point(*start_point);
        let ctrl1_id = data_pool.insert_point(ctrl1_point);
        let ctrl2_id = data_pool.insert_point(ctrl2_point);
        let end_id = data_pool.insert_point(end_point);
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::CubicBezier,
            pts_ids: vec![start_id, ctrl1_id, ctrl2_id, end_id],
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            selected: false,
            tmp: WPoint::default(),
            init: true,
        };
        data_pool.insert_shape(shape)
    }
}
#[derive(Copy, Clone)]
pub struct Rectangle;
impl Rectangle {
    pub fn new(
        data_pool: &mut DataPool,
        bl: &WPoint,
        w: f64,
        h: f64,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> usize {
        let (w, h) = if w == 0. || h == 0. {
            (5. * snap_distance, 5. * snap_distance)
        } else {
            (w, h)
        };
        let bl_id = data_pool.insert_point(*bl);
        let tl_id = data_pool.insert_point(bl.addxy(0., h));
        let tr_id = data_pool.insert_point(bl.addxy(w, 0.));
        let br_id = data_pool.insert_point(bl.addxy(w, h));
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Rectangle,
            pts_ids: vec![bl_id, tl_id, tr_id, br_id],
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            selected: false,
            tmp: WPoint::default(),
            init: true,
        };
        data_pool.insert_shape(shape)
    }
}
#[derive(Copy, Clone)]
pub struct Ellipse;
impl Ellipse {
    pub fn new(
        data_pool: &mut DataPool,
        center: &WPoint,
        radius: &WPoint,
        start_angle: f64,
        end_angle: f64,
        parameters: &ShapeParameters,
        snap_distance: f64,
    ) -> usize {
        let radius = if radius.wx == 0. || radius.wy == 0. {
            WPoint {
                wx: 5. * snap_distance,
                wy: -5. * snap_distance,
            }
        } else {
            *radius
        };
        let center_id = data_pool.insert_point(*center);
        let radius_id = data_pool.insert_point(radius);
        let h_start_angle_id =
            data_pool.insert_point(get_point_from_angle(&center, &radius, -start_angle));
        let h_end_angle_id =
            data_pool.insert_point(get_point_from_angle(&center, &radius, -end_angle));
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Ellipse,
            pts_ids: vec![center_id, radius_id, h_start_angle_id, h_end_angle_id],
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            selected: false,
            tmp: WPoint::default(),
            init: true,
        };
        data_pool.insert_shape(shape)
    }
    pub fn get_constraint(data_pool: &mut DataPool, shape: &mut Shape, pt_id: usize) {
        let (center_id, radius_id, h_start_angle_id, h_end_angle_id) = (
            shape.pts_ids[0],
            shape.pts_ids[1],
            shape.pts_ids[2],
            shape.pts_ids[3],
        );
        if pt_id == h_start_angle_id {
            //     let mut angle = get_angle_from_point(&p, &center.1, 0.);
            //     h_start_angle.1 = get_point_from_angle(&center.1, &radius.1, *rotation, -angle);
            //     magnet_geometry(&center.1, &mut h_start_angle.1, self.snap_distance);
            //     angle = get_angle_from_point(&h_start_angle.1, &center.1, 0.);
            //     *start_angle = angle;
            //     if *start_angle > *end_angle {
            //         *start_angle -= 2. * PI;
            //     }
            //     h_start_angle.1 = get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
        }
    }
}

pub enum Constraint {
    Path(fn(&mut DataPool, &mut Shape)),
}
#[derive(Copy, Clone)]
pub struct Group;
impl Group {
    pub fn new(data_pool: &mut DataPool, parameters: &ShapeParameters) -> usize {
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Group,
            pts_ids: vec![],
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            selected: false,
            tmp: WPoint::default(),
            init: true,
        };
        data_pool.insert_shape(shape)
    }
}

#[derive(Clone)]
pub struct Shape {
    shape_type: ShapeType,
    pts_ids: Vec<usize>,

    parent_shape_id: Option<usize>,
    childs_shapes: HashSet<usize>,

    parameters: ShapeParameters,
    selected: bool,
    tmp: WPoint,
    init: bool,
}

impl Shape {
    pub fn add_child_shape_id(&mut self, shape_id: usize) -> bool {
        if let ShapeType::Group = self.shape_type {
            self.childs_shapes.insert(shape_id);
            true
        } else {
            false
        }
    }
    pub fn get_type(&self) -> ShapeType {
        self.shape_type
    }
    pub fn get_pts_ids(&self) -> &Vec<usize> {
        &self.pts_ids
    }
    pub fn is_selected(&self) -> bool {
        self.selected
    }
    pub fn set_parent(&mut self, id: usize) {
        self.parent_shape_id = Some(id);
    }
    // pub fn select_from_position(&mut self, pos: &WPoint) {
    //     use ShapeType::*;
    //     match self.shape_type {
    //         Line => (),
    //         QuadBezier => (),
    //         CubicBezier => (),
    //         Rectangle => (),
    //         Ellipse => (),
    //         Group => (),
    //     }
    // }
    pub fn set_selected(&mut self) {
        self.selected = true;
    }
    // fn get_all_points_ids(&self) -> Vec<usize> {
    //     use SimpleShape::*;
    //     let mut pts_ids: Vec<usize> = vec![];
    //     for (shape, _) in self.childs_shapes.values() {
    //         match shape {
    //             Line(id1, id2) => pts_ids.extend(&[*id1, *id2]),
    //             QuadBezier(id1, id2, id3) => pts_ids.extend(&[*id1, *id2, *id3]),
    //             CubicBezier(id1, id2, id3, id4) => pts_ids.extend([*id1, *id2, *id3, *id4]),
    //             Rectangle(id1, id2, id3, id4) => pts_ids.extend([*id1, *id2, *id3, *id4]),
    //             Ellipse(id1, id2, id3, id4) => pts_ids.extend([*id1, *id2, *id3, *id4]),
    //         };
    //     }
    //     pts_ids
    // }
    // fn get_all_shapes_ids_containing_point_id(&self, pt_id: usize) -> Vec<usize> {
    //     use SimpleShape::*;
    //     let mut shapes_ids: Vec<usize> = vec![];
    //     for (simple_shape_id, (simple_shape, _)) in self.childs_shapes.iter() {
    //         let ids = match simple_shape {
    //             Line(id1, id2) => vec![*id1, *id2],
    //             QuadBezier(id1, id2, id3) => vec![*id1, *id2, *id3],
    //             CubicBezier(id1, id2, id3, id4) => vec![*id1, *id2, *id3, *id4],
    //             Rectangle(id1, id2, id3, id4) => vec![*id1, *id2, *id3, *id4],
    //             Ellipse(id1, id2, id3, id4) => vec![*id1, *id2, *id3, *id4],
    //         };
    //         for id in ids.iter() {
    //             if *id == pt_id {
    //                 shapes_ids.push(*simple_shape_id)
    //             }
    //         }
    //     }
    //     shapes_ids
    // }
    // fn get_all_points_ids_from_position(
    //     &mut self,
    //     p: &WPoint,
    //     precision: f64,
    // ) -> Option<Vec<usize>> {
    //     let mut v = vec![];
    //     for pt_id in self.get_all_points_ids().iter() {
    //         if let Some(point) = get_point(*pt_id) {
    //             if is_point_on_point(p, &point, precision) {
    //                 v.push(*pt_id);
    //             }
    //         }
    //     }
    //     if v.len() > 0 {
    //         Some(v)
    //     } else {
    //         None
    //     }
    // }

    pub fn has_childs_shapes_selected(&self, data_pool: &DataPool) -> bool {
        for shape_id in self.childs_shapes.iter() {
            if data_pool.get_shape(*shape_id).unwrap().is_selected() {
                return true;
            }
        }
        false
    }

    // pub fn get_first_shape_selected_found(&self) -> Option<usize> {
    //     for (simple_shape_id, (simple_shape, selected)) in self.childs_shapes.iter() {
    //         if *selected {
    //             return Some(*simple_shape_id);
    //         }
    //     }
    //     None
    // }
    // pub fn get_first_point_selected_found(&self) -> Option<usize> {
    //     for pt_id in get_points_selected().iter() {
    //         return Some(*pt_id);
    //     }
    //     None
    // }
    // pub fn clear_any_shape_selection(&mut self) {
    //     for (_, selected) in self.childs_shapes.values_mut() {
    //         *selected = false;
    //     }
    // }
    // pub fn clear_any_point_selection(&mut self) {
    //     remove_all_points_selected();
    // }
    // pub fn select_all_shapes(&mut self) {
    //     for (_, selected) in self.childs_shapes.values_mut() {
    //         *selected = true;
    //     }
    // }
    // pub fn get_first_point_id_from_position(
    //     &mut self,
    //     p: &WPoint,
    //     precision: f64,
    // ) -> Option<usize> {
    //     for pt_id in self.get_all_points_ids().iter() {
    //         if let Some(point) = get_point(*pt_id) {
    //             if is_point_on_point(p, &point, precision) {
    //                 return Some(*pt_id);
    //             }
    //         }
    //     }
    //     None
    // }
    // pub fn get_first_simple_shape_id_found_from_position(
    //     &mut self,
    //     p: &WPoint,
    //     precision: f64,
    // ) -> Option<usize> {
    //     use SimpleShape::*;
    //     for (simple_shape_id, (simple_shape, _)) in self.childs_shapes.iter() {
    //         match simple_shape {
    //             Line(_, _) => {
    //                 if is_point_on_line(&p, simple_shape, precision) {
    //                     return Some(*simple_shape_id);
    //                 }
    //             }
    //             QuadBezier(_, _, _) => {
    //                 if is_point_on_quadbezier(&p, simple_shape, precision) {
    //                     return Some(*simple_shape_id);
    //                 }
    //             }
    //             CubicBezier(_, _, _, _) => {
    //                 if is_point_on_cubicbezier(&p, simple_shape, precision) {
    //                     return Some(*simple_shape_id);
    //                 }
    //             }
    //             Rectangle(_, _, _, _) => {
    //                 if is_point_on_rectangle(&p, simple_shape, precision) {
    //                     return Some(*simple_shape_id);
    //                 }
    //             }
    //             Ellipse(_, _, _, _) => {
    //                 if is_point_on_ellipse(&p, simple_shape, precision) {
    //                     return Some(*simple_shape_id);
    //                 }
    //             }
    //         };
    //     }
    //     None
    // }

    // fn move_point(&self, shape_id: usize, point_selected_id: usize, p: &WPoint, dp: &WPoint) {
    //     let mut p = *p;
    //     match &mut self.childs_shapes.get(&shape_id).unwrap().0 {
    //         SimpleShape::Line(start_id, end_id) => {
    //             let mut point1 = get_point(*start_id).unwrap();
    //             let mut point2 = get_point(*end_id).unwrap();
    //             if point_selected_id == *start_id {

    //             magnet_geometry(&point1, &mut point2, self.snap_distance);
    //             snap_to_snap_grid(&mut p, self.snap_distance);
    //             point2 = p;
    //             // Avoid degenerescence
    //             if point1 == point2 {
    //                 point2 += self.snap_distance;
    //             }
    //         }else {
    //             magnet_geometry(&point1, &mut point2, self.snap_distance);
    //             snap_to_snap_grid(&mut p, self.snap_distance);
    //             point2 = p;
    //             // Avoid degenerescence
    //             if point1 == point2 {
    //                 point2 += self.snap_distance;
    //             }
    //         }
    //             modify_point(*end_id, point2);
    //         }
    //         _=> ()
    //         // SimpleShape::QuadBezier(start_id, ctrl_id, end_id) => {

    //         //     Start => {
    //         //         magnet_geometry(&end.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         start.1 = p;
    //         //     }
    //         //     Ctrl => {
    //         //         if !magnet_geometry(&end.1, &mut p, self.snap_distance) {
    //         //             magnet_geometry(&start.1, &mut p, self.snap_distance);
    //         //         }
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         ctrl.1 = p;
    //         //     }
    //         //     End => {
    //         //         magnet_geometry(&start.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         end.1 = p;
    //         //         if self.init {
    //         //             ctrl.1 = (start.1 + end.1) / 2.;
    //         //         }
    //         //     }
    //         //     All => {
    //         //         self.tmp += *dp;
    //         //         if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
    //         //             snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
    //         //             start.1.wx += self.tmp.wx;
    //         //             ctrl.1.wx += self.tmp.wx;
    //         //             end.1.wx += self.tmp.wx;
    //         //             self.tmp.wx = 0.;
    //         //         }
    //         //         if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
    //         //             snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
    //         //             start.1.wy += self.tmp.wy;
    //         //             ctrl.1.wy += self.tmp.wy;
    //         //             end.1.wy += self.tmp.wy;
    //         //             self.tmp.wy = 0.;
    //         //         }
    //         //     }
    //         //     _ => {}
    //         // },
    //         // SimpleShape::CubicBezier(start, ctrl1, ctrl2, end) => match selection {
    //         //     Start => {
    //         //         magnet_geometry(&end.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         start.1 = p;
    //         //     }
    //         //     Ctrl1 => {
    //         //         if !magnet_geometry(&end.1, &mut p, self.snap_distance) {
    //         //             if !magnet_geometry(&start.1, &mut p, self.snap_distance) {
    //         //                 magnet_geometry(&ctrl2.1, &mut p, self.snap_distance);
    //         //             }
    //         //         }
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         ctrl1.1 = p;
    //         //     }
    //         //     Ctrl2 => {
    //         //         if !magnet_geometry(&end.1, &mut p, self.snap_distance) {
    //         //             if !magnet_geometry(&start.1, &mut p, self.snap_distance) {
    //         //                 magnet_geometry(&ctrl1.1, &mut p, self.snap_distance);
    //         //             }
    //         //         }
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         ctrl2.1 = p;
    //         //     }
    //         //     End => {
    //         //         magnet_geometry(&start.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         end.1 = p;
    //         //         if self.init {
    //         //             ctrl1.1 = start.1 + (end.1 - start.1) / 3.;
    //         //             ctrl2.1 = start.1 + (end.1 - start.1) / 3. * 2.;
    //         //         }
    //         //     }
    //         //     All => {
    //         //         self.tmp += *dp;
    //         //         if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
    //         //             snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
    //         //             start.1.wx += self.tmp.wx;
    //         //             ctrl1.1.wx += self.tmp.wx;
    //         //             ctrl2.1.wx += self.tmp.wx;
    //         //             end.1.wx += self.tmp.wx;
    //         //             self.tmp.wx = 0.;
    //         //         }
    //         //         if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
    //         //             snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
    //         //             start.1.wy += self.tmp.wy;
    //         //             ctrl1.1.wy += self.tmp.wy;
    //         //             ctrl2.1.wy += self.tmp.wy;
    //         //             end.1.wy += self.tmp.wy;
    //         //             self.tmp.wy = 0.;
    //         //         }
    //         //     }
    //         //     _ => {}
    //         // },
    //         // SimpleShape::Rectangle(bl, tl, tr, br) => match selection {
    //         //     BottomLeft => {
    //         //         magnet_geometry(&tr.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         bl.1 = p;
    //         //         if bl.1.wx >= tr.1.wx {
    //         //             bl.1.wx = tr.1.wx - self.snap_distance;
    //         //         }
    //         //         if bl.1.wy <= tr.1.wy {
    //         //             bl.1.wy = tr.1.wy + self.snap_distance;
    //         //         }
    //         //         tl.1.wx = bl.1.wx;
    //         //         br.1.wy = bl.1.wy;
    //         //     }
    //         //     TopLeft => {
    //         //         magnet_geometry(&br.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         tl.1 = p;
    //         //         if tl.1.wx >= br.1.wx {
    //         //             tl.1.wx = br.1.wx - self.snap_distance;
    //         //         }
    //         //         if tl.1.wy >= br.1.wy {
    //         //             tl.1.wy = br.1.wy - self.snap_distance;
    //         //         }
    //         //         tr.1.wy = tl.1.wy;
    //         //         bl.1.wx = tl.1.wx;
    //         //     }
    //         //     TopRight => {
    //         //         magnet_geometry(&bl.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         tr.1 = p;
    //         //         if tr.1.wx <= bl.1.wx {
    //         //             tr.1.wx = bl.1.wx + self.snap_distance;
    //         //         }
    //         //         if tr.1.wy >= bl.1.wy {
    //         //             tr.1.wy = bl.1.wy - self.snap_distance;
    //         //         }
    //         //         tl.1.wy = tr.1.wy;
    //         //         br.1.wx = tr.1.wx;
    //         //     }
    //         //     BottomRight => {
    //         //         magnet_geometry(&tl.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         br.1 = p;
    //         //         if br.1.wx <= tl.1.wx {
    //         //             br.1.wx = tl.1.wx + self.snap_distance;
    //         //         }
    //         //         if br.1.wy <= tl.1.wy {
    //         //             br.1.wy = tl.1.wy + self.snap_distance;
    //         //         }
    //         //         tr.1.wx = br.1.wx;
    //         //         bl.1.wy = br.1.wy;
    //         //     }
    //         //     All => {
    //         //         self.tmp += *dp;
    //         //         if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
    //         //             snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
    //         //             bl.1.wx += self.tmp.wx;
    //         //             tl.1.wx += self.tmp.wx;
    //         //             tr.1.wx += self.tmp.wx;
    //         //             br.1.wx += self.tmp.wx;
    //         //             self.tmp.wx = 0.;
    //         //         }
    //         //         if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
    //         //             snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
    //         //             bl.1.wy += self.tmp.wy;
    //         //             tl.1.wy += self.tmp.wy;
    //         //             tr.1.wy += self.tmp.wy;
    //         //             br.1.wy += self.tmp.wy;
    //         //             self.tmp.wy = 0.;
    //         //         }
    //         //     }
    //         //     _ => {}
    //         // },
    //         // SimpleShape::Ellipse(
    //         //     center,
    //         //     radius,
    //         //     h_start_angle,
    //         //     h_end_angle,
    //         //     (rotation, start_angle, end_angle),
    //         // ) => match selection {
    //         //     Center => {
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         let delta_pos = p - center.1;
    //         //         center.1 = p;
    //         //         radius.1 += delta_pos;
    //         //         h_start_angle.1 =
    //         //             get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
    //         //         h_end_angle.1 =
    //         //             get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
    //         //     }
    //         //     Radius => {
    //         //         magnet_geometry(&center.1, &mut p, self.snap_distance);
    //         //         snap_to_snap_grid(&mut p, self.snap_distance);
    //         //         radius.1 = p;
    //         //         if radius.1.wx <= center.1.wx {
    //         //             radius.1.wx = center.1.wx + self.snap_distance;
    //         //         }
    //         //         if radius.1.wy >= center.1.wy {
    //         //             radius.1.wy = center.1.wy - self.snap_distance;
    //         //         }
    //         //         h_start_angle.1 =
    //         //             get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
    //         //         h_end_angle.1 =
    //         //             get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
    //         //     }
    //         //     StartAngle => {
    //         //         let mut angle = get_angle_from_point(&p, &center.1, 0.);
    //         //         h_start_angle.1 = get_point_from_angle(&center.1, &radius.1, *rotation, -angle);
    //         //         magnet_geometry(&center.1, &mut h_start_angle.1, self.snap_distance);
    //         //         angle = get_angle_from_point(&h_start_angle.1, &center.1, 0.);
    //         //         *start_angle = angle;
    //         //         if *start_angle > *end_angle {
    //         //             *start_angle -= 2. * PI;
    //         //         }
    //         //         h_start_angle.1 =
    //         //             get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
    //         //     }
    //         //     EndAngle => {
    //         //         let mut angle = get_angle_from_point(&p, &center.1, 0.);
    //         //         h_end_angle.1 = get_point_from_angle(&center.1, &radius.1, *rotation, -angle);
    //         //         magnet_geometry(&center.1, &mut h_end_angle.1, self.snap_distance);
    //         //         angle = get_angle_from_point(&h_end_angle.1, &center.1, 0.);
    //         //         *end_angle = angle;
    //         //         if *end_angle < *start_angle {
    //         //             *end_angle += 2. * PI;
    //         //         }
    //         //         h_end_angle.1 =
    //         //             get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
    //         //     }
    //         //     All => {
    //         //         self.tmp += *dp;
    //         //         if self.tmp.wx > self.snap_distance || self.tmp.wx < -self.snap_distance {
    //         //             snap_to_snap_grid_x(&mut self.tmp, self.snap_distance);
    //         //             center.1.wx += self.tmp.wx;
    //         //             radius.1.wx += self.tmp.wx;
    //         //             h_start_angle.1 =
    //         //                 get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
    //         //             h_end_angle.1 =
    //         //                 get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
    //         //             self.tmp.wx = 0.;
    //         //         }
    //         //         if self.tmp.wy > self.snap_distance || self.tmp.wy < -self.snap_distance {
    //         //             snap_to_snap_grid_y(&mut self.tmp, self.snap_distance);
    //         //             center.1.wy += self.tmp.wy;
    //         //             radius.1.wy += self.tmp.wy;
    //         //             h_start_angle.1 =
    //         //                 get_point_from_angle(&center.1, &radius.1, *rotation, -*start_angle);
    //         //             h_end_angle.1 =
    //         //                 get_point_from_angle(&center.1, &radius.1, *rotation, -*end_angle);
    //         //             self.tmp.wy = 0.;
    //         //         }
    //         //     }
    //         //     _ => {}
    //         // },
    //     }
    // }
    // pub fn move_selection(&mut self, p: &WPoint, dp: &WPoint) {
    //     let mut point_moved = false;
    //     let mut shapes_containing_moving_point = vec![];
    //     if let Some(point_selected_id) = self.get_first_point_selected_found() {
    //         shapes_containing_moving_point =
    //             self.get_all_shapes_ids_containing_point_id(point_selected_id);
    //     }

    //     if let Some(shape) = shapes_containing_moving_point.get(0) {
    //         //
    //     }
    // }

    // pub fn get_highlightable_positions_ids(&self, p: &WPoint, precision: f64) -> Vec<usize> {
    //     let mut v = Vec::new();
    //     if let Some(pts_ids) = self.get_all_points_ids_from_position(p, precision) {
    //         for pt_id in pts_ids.iter() {
    //             for shape_id in self.get_all_shapes_ids_containing_point_id(*pt_id).iter() {
    //                 use SimpleShape::*;
    //                 match self.simples_shapes.get(shape_id).unwrap().0 {
    //                     Line(pair1, pair2) => vec![pair1, pair2],
    //                     QuadBezier(pair1, _, pair3) => vec![pair1, pair3],
    //                     CubicBezier(pair1, _, _, pair4) => vec![pair1, pair4],
    //                     Rectangle(pair1, _, _, pair4) => vec![pair1, pair4],
    //                     Ellipse(pair1, _, _, pair4) => vec![pair1, pair4],
    //                 };
    //             }
    //         }
    //     }

    //     v.push(handle_pairs[0].clone());
    //     v.push(handle_pairs[1].clone());
    //     v
    // }
    pub fn get_shape_construction(&self, pool: &DataPool) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        if !self.selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        use ShapeType::*;
        match &self.shape_type {
            Line => {
                cst.push(ConstructionType::Move(
                    *pool.get_point(self.pts_ids[0]).unwrap(),
                ));
                cst.push(ConstructionType::Move(
                    *pool.get_point(self.pts_ids[1]).unwrap(),
                ));
            }
            QuadBezier => {
                cst.push(ConstructionType::Move(
                    *pool.get_point(self.pts_ids[0]).unwrap(),
                ));
                cst.push(ConstructionType::QuadBezier(
                    *pool.get_point(self.pts_ids[1]).unwrap(),
                    *pool.get_point(self.pts_ids[2]).unwrap(),
                ));
            }
            CubicBezier => {
                cst.push(ConstructionType::Move(
                    *pool.get_point(self.pts_ids[0]).unwrap(),
                ));
                cst.push(ConstructionType::CubicBezier(
                    *pool.get_point(self.pts_ids[1]).unwrap(),
                    *pool.get_point(self.pts_ids[2]).unwrap(),
                    *pool.get_point(self.pts_ids[3]).unwrap(),
                ));
            }
            Rectangle => {
                cst.push(ConstructionType::Move(
                    *pool.get_point(self.pts_ids[0]).unwrap(),
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(self.pts_ids[1]).unwrap(),
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(self.pts_ids[2]).unwrap(),
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(self.pts_ids[3]).unwrap(),
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(self.pts_ids[0]).unwrap(),
                ));
            }
            Ellipse => {
                cst.push(ConstructionType::Move(
                    *pool.get_point(self.pts_ids[2]).unwrap(),
                ));
                let center = *pool.get_point(self.pts_ids[0]).unwrap();
                let radius = *pool.get_point(self.pts_ids[1]).unwrap();
                let start_angle =
                    get_angle_from_point(pool.get_point(self.pts_ids[2]).unwrap(), &center);
                let end_angle =
                    get_angle_from_point(pool.get_point(self.pts_ids[3]).unwrap(), &center);
                cst.push(ConstructionType::Ellipse(
                    center,
                    (radius - center).abs(),
                    0.,
                    start_angle,
                    end_angle,
                    false,
                ));
            }
            Group => cst = vec![], //self.get_shape_construction(pool),
        };
        cst
    }
    // pub fn get_handles_construction(&self) -> Vec<ConstructionType> {
    //     let mut cst = Vec::new();
    //     use Handle::*;
    //     if let Some(selection) = self.selection {
    //         use SimpleShapeType::*;
    //         let handles_pairs = match &self.shape {
    //             Line(pair1, pair2) => match selection {
    //                 Start => vec![(pair1.1, true), (pair2.1, false)],
    //                 End => vec![(pair1.1, false), (pair2.1, true)],
    //                 _ => vec![(pair1.1, false), (pair2.1, false)],
    //             },
    //             QuadBezier(pair1, pair2, pair3) => match selection {
    //                 Start => vec![(pair1.1, true), (pair2.1, false), (pair3.1, false)],
    //                 Ctrl => vec![(pair1.1, false), (pair2.1, true), (pair3.1, false)],
    //                 End => vec![(pair1.1, false), (pair2.1, false), (pair3.1, true)],
    //                 _ => vec![(pair1.1, false), (pair2.1, false), (pair3.1, false)],
    //             },

    //             CubicBezier(pair1, pair2, pair3, pair4) => match selection {
    //                 Start => vec![
    //                     (pair1.1, true),
    //                     (pair2.1, false),
    //                     (pair3.1, false),
    //                     (pair4.1, false),
    //                 ],
    //                 Ctrl1 => vec![
    //                     (pair1.1, false),
    //                     (pair2.1, true),
    //                     (pair3.1, false),
    //                     (pair4.1, false),
    //                 ],
    //                 Ctrl2 => vec![
    //                     (pair1.1, false),
    //                     (pair2.1, false),
    //                     (pair3.1, true),
    //                     (pair4.1, false),
    //                 ],
    //                 End => vec![
    //                     (pair1.1, false),
    //                     (pair2.1, false),
    //                     (pair3.1, false),
    //                     (pair4.1, true),
    //                 ],
    //                 _ => vec![
    //                     (pair1.1, false),
    //                     (pair2.1, false),
    //                     (pair3.1, false),
    //                     (pair4.1, false),
    //                 ],
    //             },
    //             Rectangle(bl, tl, tr, br) => match selection {
    //                 BottomLeft => vec![(bl.1, true), (tl.1, false), (tr.1, false), (br.1, false)],
    //                 TopLeft => vec![(bl.1, false), (tl.1, true), (tr.1, false), (br.1, false)],
    //                 TopRight => vec![(bl.1, false), (tl.1, false), (tr.1, true), (br.1, false)],
    //                 BottomRight => vec![(bl.1, false), (tl.1, false), (tr.1, false), (br.1, true)],
    //                 _ => vec![(bl.1, false), (tl.1, false), (tr.1, false), (br.1, false)],
    //             },
    //             Ellipse(center, radius, h_start_angle, h_end_angle, _) => match selection {
    //                 Center => vec![
    //                     (center.1, true),
    //                     (radius.1, false),
    //                     (h_start_angle.1, false),
    //                     (h_end_angle.1, false),
    //                 ],
    //                 Radius => vec![
    //                     (center.1, false),
    //                     (radius.1, true),
    //                     (h_start_angle.1, false),
    //                     (h_end_angle.1, false),
    //                 ],
    //                 StartAngle => vec![
    //                     (center.1, false),
    //                     (radius.1, false),
    //                     (h_start_angle.1, true),
    //                     (h_end_angle.1, false),
    //                 ],
    //                 EndAngle => vec![
    //                     (center.1, false),
    //                     (radius.1, false),
    //                     (h_start_angle.1, false),
    //                     (h_end_angle.1, true),
    //                 ],
    //                 _ => vec![
    //                     (center.1, false),
    //                     (radius.1, false),
    //                     (h_start_angle.1, false),
    //                     (h_end_angle.1, false),
    //                 ],
    //             },
    //         };
    //         for (point, fill) in handles_pairs.iter() {
    //             push_handle(&point, &self.handles_size, *fill, &mut cst);
    //         }
    //     }
    //     cst
    // }
    // pub fn get_highlight_construction(&self) -> Vec<ConstructionType> {
    //     let mut cst = Vec::new();
    //     // Get a list of all highlightable handle-point pairs in the current shape
    //     use SimpleShapeType::*;
    //     let handle_pairs = match &self.shape {
    //         Line(pair1, pair2) => vec![pair1.1, pair2.1],
    //         QuadBezier(pair1, _, pair3) => vec![pair1.1, pair3.1],
    //         CubicBezier(pair1, _, _, pair4) => vec![pair1.1, pair4.1],
    //         Rectangle(pair1, _, _, pair4) => vec![pair1.1, pair4.1],
    //         Ellipse(pair1, _, _, pair4, _) => vec![pair1.1, pair4.1],
    //     };

    //     if let Some(highlight) = self.highlight {
    //         match highlight {
    //             Handle::Start => {
    //                 cst.push(ConstructionType::Layer(LayerType::Highlight));
    //                 push_handle(&handle_pairs[0], &self.highlight_size, false, &mut cst);
    //             }
    //             Handle::End => {
    //                 cst.push(ConstructionType::Layer(LayerType::Highlight));
    //                 push_handle(&handle_pairs[1], &self.highlight_size, false, &mut cst);
    //             }
    //             _ => (),
    //         };
    //     }
    //     cst
    // }
    // pub fn get_helpers_construction(&self) -> Vec<ConstructionType> {
    //     let mut cst = Vec::new();
    //     // Get a list of all highlightable handle-point pairs in the current shape
    //     use Handle::*;
    //     use SimpleShapeType::*;
    //     match &self.shape {
    //         Line(pair1, pair2) => {
    //             if let Some(selection) = self.selection {
    //                 match selection {
    //                     Start | End => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&pair1.1, &pair2.1, true, &mut cst);
    //                         push_horizontal(&pair1.1, &pair2.1, true, &mut cst);
    //                         push_45_135(&pair1.1, &pair2.1, true, &mut cst);
    //                     }
    //                     _ => (),
    //                 }
    //             }
    //         }
    //         QuadBezier(pair1, pair2, pair3) => {
    //             if let Some(selection) = self.selection {
    //                 match selection {
    //                     Start | End => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&pair1.1, &pair3.1, true, &mut cst);
    //                         push_horizontal(&pair1.1, &pair3.1, true, &mut cst);
    //                         push_45_135(&pair1.1, &pair3.1, true, &mut cst);
    //                     }
    //                     Ctrl => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&pair2.1, &pair1.1, true, &mut cst);
    //                         push_horizontal(&pair2.1, &pair1.1, true, &mut cst);
    //                         push_45_135(&pair2.1, &pair1.1, true, &mut cst);
    //                         push_vertical(&pair2.1, &pair3.1, true, &mut cst);
    //                         push_horizontal(&pair2.1, &pair3.1, true, &mut cst);
    //                         push_45_135(&pair2.1, &pair3.1, true, &mut cst);
    //                     }
    //                     _ => (),
    //                 }
    //             }
    //         }
    //         CubicBezier(pair1, pair2, pair3, pair4) => {
    //             if let Some(selection) = self.selection {
    //                 match selection {
    //                     Start | End => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&pair1.1, &pair4.1, true, &mut cst);
    //                         push_horizontal(&pair1.1, &pair4.1, true, &mut cst);
    //                         push_45_135(&pair1.1, &pair4.1, true, &mut cst);
    //                     }
    //                     Ctrl1 => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&pair2.1, &pair1.1, true, &mut cst);
    //                         push_horizontal(&pair2.1, &pair1.1, true, &mut cst);
    //                         push_45_135(&pair2.1, &pair1.1, true, &mut cst);
    //                         push_vertical(&pair2.1, &pair3.1, true, &mut cst);
    //                         push_horizontal(&pair2.1, &pair3.1, true, &mut cst);
    //                         push_45_135(&pair2.1, &pair3.1, true, &mut cst);
    //                         push_vertical(&pair2.1, &pair4.1, true, &mut cst);
    //                         push_horizontal(&pair2.1, &pair4.1, true, &mut cst);
    //                         push_45_135(&pair2.1, &pair4.1, true, &mut cst);
    //                     }
    //                     Ctrl2 => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&pair3.1, &pair1.1, true, &mut cst);
    //                         push_horizontal(&pair3.1, &pair1.1, true, &mut cst);
    //                         push_45_135(&pair3.1, &pair1.1, true, &mut cst);
    //                         push_vertical(&pair3.1, &pair2.1, true, &mut cst);
    //                         push_horizontal(&pair3.1, &pair2.1, true, &mut cst);
    //                         push_45_135(&pair3.1, &pair2.1, true, &mut cst);
    //                         push_vertical(&pair3.1, &pair4.1, true, &mut cst);
    //                         push_horizontal(&pair3.1, &pair4.1, true, &mut cst);
    //                         push_45_135(&pair3.1, &pair4.1, true, &mut cst);
    //                     }
    //                     _ => (),
    //                 }
    //             }
    //         }
    //         Rectangle(bl, tl, tr, br) => {
    //             if let Some(selection) = self.selection {
    //                 match selection {
    //                     BottomLeft => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_45_135(&bl.1, &tr.1, true, &mut cst);
    //                     }
    //                     TopLeft => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_45_135(&tl.1, &br.1, true, &mut cst);
    //                     }
    //                     TopRight => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_45_135(&tr.1, &bl.1, true, &mut cst);
    //                     }
    //                     BottomRight => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_45_135(&br.1, &tl.1, true, &mut cst);
    //                     }
    //                     _ => (),
    //                 }
    //             }
    //         }
    //         Ellipse(center, radius, h_start_angle, h_end_angle, _) => {
    //             if let Some(selection) = self.selection {
    //                 match selection {
    //                     Center => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&center.1, &h_start_angle.1, true, &mut cst);
    //                         push_horizontal(&center.1, &h_start_angle.1, true, &mut cst);
    //                         push_45_135(&center.1, &h_start_angle.1, true, &mut cst);
    //                         push_vertical(&center.1, &h_end_angle.1, true, &mut cst);
    //                         push_horizontal(&center.1, &h_end_angle.1, true, &mut cst);
    //                         push_45_135(&center.1, &h_end_angle.1, true, &mut cst);
    //                         push_vertical(&center.1, &radius.1, true, &mut cst);
    //                         push_horizontal(&center.1, &radius.1, true, &mut cst);
    //                         push_45_135(&center.1, &radius.1, true, &mut cst);
    //                     }
    //                     Radius => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&center.1, &radius.1, true, &mut cst);
    //                         push_horizontal(&center.1, &radius.1, true, &mut cst);
    //                         push_45_135(&center.1, &radius.1, true, &mut cst);
    //                     }
    //                     StartAngle => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&h_start_angle.1, &center.1, true, &mut cst);
    //                         push_horizontal(&h_start_angle.1, &center.1, true, &mut cst);
    //                         push_45_135(&h_start_angle.1, &center.1, true, &mut cst);
    //                         push_vertical(&h_start_angle.1, &h_end_angle.1, true, &mut cst);
    //                         push_horizontal(&h_start_angle.1, &h_end_angle.1, true, &mut cst);
    //                         push_45_135(&h_start_angle.1, &h_end_angle.1, true, &mut cst);
    //                     }
    //                     EndAngle => {
    //                         cst.push(ConstructionType::Layer(LayerType::GeometryHelpers));
    //                         push_vertical(&h_end_angle.1, &center.1, true, &mut cst);
    //                         push_horizontal(&h_end_angle.1, &center.1, true, &mut cst);
    //                         push_45_135(&h_end_angle.1, &center.1, true, &mut cst);
    //                         push_vertical(&h_end_angle.1, &h_start_angle.1, true, &mut cst);
    //                         push_horizontal(&h_end_angle.1, &h_start_angle.1, true, &mut cst);
    //                         push_45_135(&h_end_angle.1, &h_start_angle.1, true, &mut cst);
    //                     }
    //                     _ => (),
    //                 }
    //             }
    //         }
    //     };
    //     cst
    // }
    pub fn init_done(&mut self) {
        self.init = false;
    }
}
