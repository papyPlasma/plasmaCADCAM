use crate::datapool::PointId;
use crate::datapool::PointProperty;
use crate::datapool::PointType;
use crate::datapool::WPoint;
use std::collections::HashMap;

pub type PtIdProp = HashMap<PointType, (PointId, PointProperty)>;

pub trait Shape {
    fn is_init(&self) -> bool;
    fn get_pos_id(&self) -> (PointId, PointProperty);
    fn init_done(&mut self);
    fn get_points_ids(&self) -> PtIdProp;
    fn is_point_on_shape(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        pt: &WPoint,
        precision: f64,
    ) -> bool;
    fn update_points_pos(
        &self,
        pts_pos: &mut HashMap<PointType, (PointId, WPoint)>,
        pt_id: &PointId,
        pick_pt: &WPoint,
        snap_distance: f64,
    );
    fn get_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        selected: bool,
    ) -> Vec<ConstructionType>;
    fn get_handles_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
        opt_sel_id_prop: &Option<(PointId, PointProperty)>,
        size_handle: f64,
    ) -> Vec<ConstructionType>;
    fn get_helpers_construction(
        &self,
        pts_pos: &HashMap<PointType, (PointId, WPoint)>,
    ) -> Vec<ConstructionType>;
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub struct ShapeParameters {
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
