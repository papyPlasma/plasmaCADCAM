use crate::datapool::DataPool;
use crate::datapool::PointId;
use crate::datapool::ShapeId;
use crate::math::*;
use crate::shapes::*;
use std::collections::{HashMap, HashSet};
pub trait ShapesOperations {
    fn get_shape_construction(
        pool: &DataPool,
        shape: &Shape,
        selected: bool,
    ) -> Vec<ConstructionType>;
    fn get_helpers_construction(
        pool: &DataPool,
        shape: &Shape,
        ohandle_selected: &Option<(HandleType, PointId)>,
    ) -> Vec<ConstructionType>;
}

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
    pub shape_type: ShapeType,
    pub handles_bundles: HashMap<HandleType, PointId>,
    pub parent_shape_id: Option<ShapeId>,
    pub childs_shapes: HashSet<ShapeId>,
    pub parameters: ShapeParameters,
    pub coord: WPoint,
    pub init: bool,
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
    pub fn get_handle_bundle_from_point(&self, point_id: PointId) -> Option<(HandleType, PointId)> {
        self.handles_bundles
            .iter()
            .find(|(_, &pt_id)| pt_id == point_id)
            .map(|(&k, &v)| (k, v))
    }
    pub fn move_shape(&mut self, pos: &WPoint) {
        self.coord = *pos;
    }
    pub fn get_coord(&self) -> &WPoint {
        &self.coord
    }
    pub fn get_coord_mut(&mut self) -> &mut WPoint {
        &mut self.coord
    }
    // pub fn get_handle_bundle_from_point_mut(
    //     &mut self,
    //     point_id: PointId,
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
    // pub fn select_handle(&mut self, point_id: PointId) -> Option<(HandleType, PointId)> {
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
    pub fn get_handles_construction(
        &self,
        pool: &DataPool,
        ohandle_selected: &Option<(HandleType, PointId)>,
    ) -> Vec<ConstructionType> {
        let mut cst = Vec::new();
        for (handle, &point_id) in self.handles_bundles.iter() {
            // Check selection
            let selected = if let Some((handle_selected, _)) = ohandle_selected {
                if handle_selected == handle {
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
    //         Line(pair1, pair2) => vec![start, ctrl1],
    //         QuadBezier(pair1, _, pair3) => vec![start, ctrl2],
    //         CubicBezier(pair1, _, _, pair4) => vec![start, end],
    //         Rectangle(pair1, _, _, pair4) => vec![start, end],
    //         Ellipse(pair1, _, _, pair4, _) => vec![start, end],
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
}
impl ShapesOperations for Shape {
    fn get_shape_construction(
        pool: &DataPool,
        shape: &Shape,
        selected: bool,
    ) -> Vec<ConstructionType> {
        use ShapeType::*;
        match shape.shape_type {
            Line => line::Line::get_shape_construction(pool, shape, selected),
            QuadBezier => quadbezier::QuadBezier::get_shape_construction(pool, shape, selected),
            CubicBezier => cubicbezier::CubicBezier::get_shape_construction(pool, shape, selected),
            Rectangle => rectangle::Rectangle::get_shape_construction(pool, shape, selected),
            Ellipse => ellipse::Ellipse::get_shape_construction(pool, shape, selected),
            Group => vec![],
        }
    }
    fn get_helpers_construction(
        pool: &DataPool,
        shape: &Shape,
        ohandle_selected: &Option<(HandleType, PointId)>,
    ) -> Vec<ConstructionType> {
        use ShapeType::*;
        match shape.shape_type {
            Line => line::Line::get_helpers_construction(pool, shape, ohandle_selected),
            QuadBezier => {
                quadbezier::QuadBezier::get_helpers_construction(pool, shape, ohandle_selected)
            }
            CubicBezier => {
                cubicbezier::CubicBezier::get_helpers_construction(pool, shape, ohandle_selected)
            }
            Rectangle => {
                rectangle::Rectangle::get_helpers_construction(pool, shape, ohandle_selected)
            }
            Ellipse => ellipse::Ellipse::get_helpers_construction(pool, shape, ohandle_selected),
            Group => vec![],
        }
    }
}
