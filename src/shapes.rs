// use web_sys::console;

// use web_sys::console;

use web_sys::console;

use crate::math::*;
use std::collections::{HashMap, HashSet};

use std::{
    f64::consts::PI,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum HandleType {
    Start,
    End,
    Center,
    Radius,
    StartAngle,
    EndAngle,
    BL,
    TL,
    TR,
    BR,
    Ctrl,
    Ctrl1,
    Ctrl2,
}

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
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *start_point;
        let start_point = *start_point - coord;
        let end_point = *end_point - coord;

        let end_point = if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
            start_point + snap_distance
        } else {
            end_point
        };
        let start_id = data_pool.insert_point(&start_point);
        let end_id = data_pool.insert_point(&end_point);
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Line,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::Start, start_id);
                pts_ids.insert(HandleType::End, end_id);
                pts_ids
            },
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord,
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, Some((HandleType::End, end_id)))
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
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *start_point;
        let start_point = *start_point - coord;
        let ctrl_point = *ctrl_point - coord;
        let end_point = *end_point - coord;

        let (end_point, ctrl_point) =
            if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
                (
                    start_point + 2. * snap_distance,
                    start_point + snap_distance,
                )
            } else {
                (end_point, ctrl_point)
            };
        let start_id = data_pool.insert_point(&start_point);
        let ctrl_id = data_pool.insert_point(&ctrl_point);
        let end_id = data_pool.insert_point(&end_point);
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::QuadBezier,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::Start, start_id);
                pts_ids.insert(HandleType::Ctrl, ctrl_id);
                pts_ids.insert(HandleType::End, end_id);
                pts_ids
            },
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord,
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, Some((HandleType::End, end_id)))
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
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *start_point;
        let start_point = *start_point - coord;
        let ctrl1_point = *ctrl1_point - coord;
        let ctrl2_point = *ctrl2_point - coord;
        let end_point = *end_point - coord;

        let (end_point, ctrl1_point, ctrl2_point) =
            if start_point.wx == end_point.wx || start_point.wy == end_point.wy {
                (
                    start_point + 3. * snap_distance,
                    start_point + snap_distance,
                    start_point + 2. * snap_distance,
                )
            } else {
                (end_point, ctrl1_point, ctrl2_point)
            };
        let start_id = data_pool.insert_point(&start_point);
        let ctrl1_id = data_pool.insert_point(&ctrl1_point);
        let ctrl2_id = data_pool.insert_point(&ctrl2_point);
        let end_id = data_pool.insert_point(&end_point);
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::CubicBezier,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::Start, start_id);
                pts_ids.insert(HandleType::Ctrl1, ctrl1_id);
                pts_ids.insert(HandleType::Ctrl2, ctrl2_id);
                pts_ids.insert(HandleType::End, end_id);
                pts_ids
            },
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord,
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, Some((HandleType::End, end_id)))
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
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *bl;
        let bl = *bl - coord;

        let (w, h) = if w == 0. || h == 0. {
            (5. * snap_distance, 5. * snap_distance)
        } else {
            (w, h)
        };
        let bl_id = data_pool.insert_point(&bl);
        let tl_id = data_pool.insert_point(&WPoint::new(0., h));
        let tr_id = data_pool.insert_point(&WPoint::new(w, h));
        let br_id = data_pool.insert_point(&WPoint::new(w, 0.));
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Rectangle,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::BL, bl_id);
                pts_ids.insert(HandleType::TL, tl_id);
                pts_ids.insert(HandleType::TR, tr_id);
                pts_ids.insert(HandleType::BR, br_id);
                pts_ids
            },
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord,
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, Some((HandleType::End, tr_id)))
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
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let coord = *center;
        let center = *center - coord;
        let radius = *radius - coord;
        let radius = if radius.wx == 0. || radius.wy == 0. {
            WPoint::new(5. * snap_distance, -5. * snap_distance)
        } else {
            radius
        };
        let center_id = data_pool.insert_point(&center);
        let radius_id = data_pool.insert_point(&(radius + center));
        let h_start_angle = get_point_from_angle(&(radius + center), -start_angle);
        let h_end_angle = get_point_from_angle(&(radius + center), -end_angle);
        let h_start_angle_id = data_pool.insert_point(&h_start_angle);
        let h_end_angle_id = data_pool.insert_point(&h_end_angle);

        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Ellipse,
            handles_bundles: {
                let mut pts_ids = HashMap::new();
                pts_ids.insert(HandleType::Center, center_id);
                pts_ids.insert(HandleType::Radius, radius_id);
                pts_ids.insert(HandleType::StartAngle, h_start_angle_id);
                pts_ids.insert(HandleType::EndAngle, h_end_angle_id);
                pts_ids
            },
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord,
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, Some((HandleType::End, radius_id)))
    }
}

#[derive(Copy, Clone)]
pub struct Group;
impl Group {
    pub fn new(
        data_pool: &mut DataPool,
        parameters: &ShapeParameters,
    ) -> (ShapeId, Option<(HandleType, PointId)>) {
        let shape = Shape {
            parent_shape_id: None,
            shape_type: ShapeType::Group,
            handles_bundles: HashMap::new(),
            childs_shapes: HashSet::new(),
            parameters: *parameters,
            // selected: false,
            coord: WPoint::default(),
            init: true,
        };
        let shape_id = data_pool.insert_shape(shape);
        (shape_id, None)
    }
}

#[derive(Clone)]
pub struct Shape {
    shape_type: ShapeType,
    handles_bundles: HashMap<HandleType, PointId>,
    parent_shape_id: Option<ShapeId>,
    childs_shapes: HashSet<ShapeId>,
    parameters: ShapeParameters,
    // selected: bool,
    coord: WPoint,
    init: bool,
}

impl Shape {
    pub fn is_init(&self) -> bool {
        self.init
    }
    pub fn init_done(&mut self) {
        self.init = false;
    }
    pub fn get_childs_shapes_mut(&mut self) -> &mut HashSet<ShapeId> {
        &mut self.childs_shapes
    }
    pub fn get_childs_shapes(&self) -> &HashSet<ShapeId> {
        &self.childs_shapes
    }
    pub fn add_child_shape_id(&mut self, shape_id: ShapeId) -> bool {
        if let ShapeType::Group = self.shape_type {
            self.childs_shapes.insert(shape_id);
            true
        } else {
            false
        }
    }
    pub fn remove_child_shape_id(&mut self, shape_id: ShapeId) -> bool {
        if let ShapeType::Group = self.shape_type {
            self.childs_shapes.remove(&shape_id);
            true
        } else {
            false
        }
    }
    pub fn get_type(&self) -> ShapeType {
        self.shape_type
    }
    pub fn get_handle_bundle_from_handle(
        &self,
        handle_type: &HandleType,
    ) -> Option<(HandleType, PointId)> {
        if let Some(point_id) = self.handles_bundles.get(handle_type) {
            return Some((*handle_type, *point_id));
        }
        None
    }
    pub fn get_handle_bundle_from_point(
        &self,
        point_id: &PointId,
    ) -> Option<(HandleType, PointId)> {
        self.handles_bundles
            .iter()
            .find(|(_, &pt_id)| pt_id == *point_id)
            .map(|(&k, &v)| (k, v))
    }
    pub fn move_shape(&mut self, pos: WPoint) {
        // for pt_id in self.handles_bundles.values() {
        //     let mut pt = pool.get_point(pt_id).unwrap().clone();
        //     pt = pt - self.coord + pos;
        //     pool.modify_point(pt_id, &pt);
        // }
        self.coord = pos;
    }
    pub fn get_coord(&self) -> &WPoint {
        &self.coord
    }
    pub fn get_coord_mut(&mut self) -> &mut WPoint {
        &mut self.coord
    }
    // pub fn get_handle_bundle_from_point_mut(
    //     &mut self,
    //     point_id: &PointId,
    // ) -> Option<&mut (HandleType, PointId)> {
    //     self.handles_bundles
    //         .get_key_value(k)
    //         .iter_mut()
    //         .find(|(handle_type, pt_id)| *pt_id == *point_id)
    //         .map(|(k, v)| &(k, v))
    // }
    pub fn get_handles_bundles(&self) -> &HashMap<HandleType, PointId> {
        &self.handles_bundles
    }
    pub fn get_handles_bundles_mut(&mut self) -> &mut HashMap<HandleType, PointId> {
        &mut self.handles_bundles
    }
    // pub fn select_handle(&mut self, point_id: &PointId) -> Option<(HandleType, PointId)> {
    //     let mut selection_done = false;
    //     for handle_bdl in self.handles_bundles.iter_mut() {
    //         if handle_bdl.1 == *point_id {
    //             handle_bdl.2 = true;
    //             return Some(*handle_bdl);
    //         }
    //     }
    //     None
    // }
    // pub fn is_selected(&self) -> bool {
    //     self.selected
    // }
    pub fn set_parent(&mut self, shape_id: &ShapeId) {
        self.parent_shape_id = Some(*shape_id);
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
    // pub fn set_selected(&mut self, selected: bool) {
    //     self.selected = selected;
    // }
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
    // pub fn has_childs_shapes_selected(&self, data_pool: &DataPool) -> bool {
    //     for shape_id in self.childs_shapes.iter() {
    //         if data_pool.get_shape(shape_id).unwrap().is_selected() {
    //             return true;
    //         }
    //     }
    //     false
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
    pub fn get_shape_construction(&self, pool: &DataPool, selected: bool) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        if !selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        use ShapeType::*;
        let coord = self.coord;
        match &self.shape_type {
            Line => {
                let start_id = self.handles_bundles.get(&HandleType::Start).unwrap();
                let end_id = self.handles_bundles.get(&HandleType::End).unwrap();
                cst.push(ConstructionType::Move(
                    *pool.get_point(start_id).unwrap() + coord,
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(end_id).unwrap() + coord,
                ));
            }
            QuadBezier => {
                let start_id = self.handles_bundles.get(&HandleType::Start).unwrap();
                let ctrl_id = self.handles_bundles.get(&HandleType::Ctrl).unwrap();
                let end_id = self.handles_bundles.get(&HandleType::End).unwrap();
                cst.push(ConstructionType::Move(
                    *pool.get_point(start_id).unwrap() + coord,
                ));
                cst.push(ConstructionType::QuadBezier(
                    *pool.get_point(ctrl_id).unwrap() + coord,
                    *pool.get_point(end_id).unwrap() + coord,
                ));
            }
            CubicBezier => {
                let start_id = self.handles_bundles.get(&HandleType::Start).unwrap();
                let ctrl1_id = self.handles_bundles.get(&HandleType::Ctrl1).unwrap();
                let ctrl2_id = self.handles_bundles.get(&HandleType::Ctrl2).unwrap();
                let end_id = self.handles_bundles.get(&HandleType::End).unwrap();
                cst.push(ConstructionType::Move(
                    *pool.get_point(start_id).unwrap() + coord,
                ));
                cst.push(ConstructionType::CubicBezier(
                    *pool.get_point(ctrl1_id).unwrap() + coord,
                    *pool.get_point(ctrl2_id).unwrap() + coord,
                    *pool.get_point(end_id).unwrap() + coord,
                ));
            }
            Rectangle => {
                let bl_id = self.handles_bundles.get(&HandleType::BL).unwrap();
                let tl_id = self.handles_bundles.get(&HandleType::TL).unwrap();
                let tr_id = self.handles_bundles.get(&HandleType::TR).unwrap();
                let br_id = self.handles_bundles.get(&HandleType::BR).unwrap();
                cst.push(ConstructionType::Move(
                    *pool.get_point(bl_id).unwrap() + coord,
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(tl_id).unwrap() + coord,
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(tr_id).unwrap() + coord,
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(br_id).unwrap() + coord,
                ));
                cst.push(ConstructionType::Line(
                    *pool.get_point(bl_id).unwrap() + coord,
                ));
            }
            Ellipse => {
                let center_id = self.handles_bundles.get(&HandleType::Center).unwrap();
                let radius_id = self.handles_bundles.get(&HandleType::Radius).unwrap();
                let h_start_angle_id = self.handles_bundles.get(&HandleType::StartAngle).unwrap();
                let h_end_angle_id = self.handles_bundles.get(&HandleType::EndAngle).unwrap();

                let center = *pool.get_point(center_id).unwrap();
                let radius = *pool.get_point(radius_id).unwrap();
                let h_start_angle = *pool.get_point(h_start_angle_id).unwrap();
                let h_end_angle = *pool.get_point(h_end_angle_id).unwrap();

                cst.push(ConstructionType::Move(coord + center + h_start_angle));

                let start_angle = -center.angle_on_ellipse(&h_start_angle, &radius);
                let end_angle = -center.angle_on_ellipse(&h_end_angle, &radius);
                cst.push(ConstructionType::Ellipse(
                    coord + center,
                    radius.abs(),
                    0.,
                    start_angle,
                    end_angle,
                    false,
                ));
            }
            Group => cst = vec![],
        };
        cst
    }
    pub fn get_handles_construction(
        &self,
        pool: &DataPool,
        ohandle_selected: Option<(HandleType, PointId)>,
    ) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        for (handle, point_id) in self.handles_bundles.iter() {
            // Check selection
            let selected = if let Some((handle_selected, _)) = ohandle_selected {
                if handle_selected == *handle {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            let pt = *pool.get_point(point_id).unwrap();
            push_handle(
                &(pt + self.coord),
                self.parameters.handles_size,
                selected,
                &mut cst,
            );
        }
        cst
    }
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
}
