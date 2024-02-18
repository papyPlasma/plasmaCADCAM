// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use crate::types::*;
use kurbo::{BezPath, Line, ParamCurveNearest, Point, Shape};
use std::collections::HashMap;

pub trait ApiShapes {
    fn get_id(&self) -> ShapeId;
    fn get_path(&self, tol: f64, v_pool: &HashMap<VertexId, Vertex>) -> BezPath;
    fn is_selected(&self) -> bool;
    fn set_selected(&mut self, selection: bool);
    fn is_near_pos(
        &self,
        pick_pos: &Point,
        grab_handle_precision: f64,
        v_pool: &HashMap<VertexId, Vertex>,
    ) -> bool;
    fn get_vertices_ids(&self) -> Vec<VertexId>;
    fn get_vextex_construction(&self) -> VertexId;
    fn get_bounded_rectangle(&self) -> [Point; 2];
}

#[derive(Clone, Debug)]
pub enum Shapes {
    STLine(LineShape),
}
impl ApiShapes for Shapes {
    fn get_id(&self) -> ShapeId {
        use Shapes::*;
        match self {
            STLine(line_shape) => line_shape.id,
        }
    }
    fn get_path(&self, tol: f64, v_pool: &HashMap<VertexId, Vertex>) -> BezPath {
        use Shapes::*;
        match self {
            STLine(line_shape) => line_shape.get_path(tol, v_pool),
        }
    }
    fn is_selected(&self) -> bool {
        use Shapes::*;
        match self {
            STLine(line_shape) => line_shape.is_selected(),
        }
    }
    fn set_selected(&mut self, selection: bool) {
        use Shapes::*;
        match self {
            STLine(line_shape) => line_shape.set_selected(selection),
        };
    }
    fn is_near_pos(
        &self,
        pick_pos: &Point,
        grab_handle_precision: f64,
        v_pool: &HashMap<VertexId, Vertex>,
    ) -> bool {
        use Shapes::*;
        match self {
            STLine(line_shape) => line_shape.is_near_pos(pick_pos, grab_handle_precision, v_pool),
        }
    }
    fn get_vertices_ids(&self) -> Vec<VertexId> {
        use Shapes::*;
        match self {
            STLine(line_shape) => line_shape.get_vertices_ids(),
        }
    }
    fn get_vextex_construction(&self) -> VertexId {
        use Shapes::*;
        match self {
            STLine(line_shape) => line_shape.get_vextex_construction(),
        }
    }
    fn get_bounded_rectangle(&self) -> [Point; 2] {
        // TODO
        [Point::ZERO, Point::ZERO]
    }
}

#[derive(Clone, Debug)]
pub struct LineShape {
    id: ShapeId,
    selected: bool,
    highlighted: bool,
    va_id: VertexId,
    vb_id: VertexId,
}
impl LineShape {
    pub fn new(id: ShapeId, va: &Vertex, vb: &Vertex) -> LineShape {
        LineShape {
            id,
            selected: false,
            highlighted: false,
            va_id: va.id,
            vb_id: vb.id,
        }
    }
}
impl ApiShapes for LineShape {
    fn get_id(&self) -> ShapeId {
        self.id
    }
    fn get_path(&self, tol: f64, v_pool: &HashMap<VertexId, Vertex>) -> BezPath {
        let va = v_pool.get(&self.va_id).unwrap();
        let vb = v_pool.get(&self.vb_id).unwrap();
        Line::new(va.pt, vb.pt).into_path(tol)
    }
    fn is_selected(&self) -> bool {
        self.selected
    }
    fn set_selected(&mut self, selection: bool) {
        self.selected = selection
    }
    fn is_near_pos(
        &self,
        pick_pos: &Point,
        grab_handle_precision: f64,
        v_pool: &HashMap<VertexId, Vertex>,
    ) -> bool {
        let va = v_pool.get(&self.va_id).unwrap();
        let vb = v_pool.get(&self.vb_id).unwrap();
        let online = Line::new(va.pt, vb.pt)
            .nearest(*pick_pos, 0.)
            .distance_sq
            .sqrt()
            < grab_handle_precision / 4.;
        let on_va = va.pt.distance(*pick_pos) < grab_handle_precision / 2.;
        let on_vb = vb.pt.distance(*pick_pos) < grab_handle_precision / 2.;
        if on_va || on_vb {
            false
        } else {
            online
        }
    }
    fn get_vertices_ids(&self) -> Vec<VertexId> {
        let mut v = vec![];
        v.push(self.va_id);
        v.push(self.vb_id);
        v
    }
    fn get_vextex_construction(&self) -> VertexId {
        self.vb_id
    }
    fn get_bounded_rectangle(&self) -> [Point; 2] {
        // TODO
        [Point::ZERO, Point::ZERO]
    }
}

// #[derive(Clone, Debug)]
// pub struct ArcShape {
//     pub shape_id: ShapeTypeId,
//     selected: bool,
//     arc: Arc,
//     pub v_center: Vertex,
//     pub v_radii: Vertex,
//     pub v_start_angle: Vertex,
//     pub v_end_angle: Vertex,
// }
// impl ArcShape {
//     fn get_v_start_angle(_arc: &Arc) -> Vertex {
//         Vertex::new((0., 0.))
//         //TODO
//     }
//     fn get_v_end_angle(_arc: &Arc) -> Vertex {
//         Vertex::new((0., 0.))
//         //TODO
//     }
//     pub fn new(arc: Arc) -> ArcShape {
//         ArcShape {
//             shape_id: ShapeTypeId::new_id(),
//             selected: false,
//             arc,
//             v_center: Vertex::new(arc.center),
//             v_radii: Vertex::new((arc.radii.x, arc.radii.y)).set_selection(true),
//             v_start_angle: ArcShape::get_v_start_angle(&arc),
//             v_end_angle: ArcShape::get_v_end_angle(&arc),
//         }
//     }
// }
// impl ApiShapes for ArcShape {
//     fn is_selected(&self) -> bool {
//         self.selected
//     }
//     fn set_selected(&mut self, selection: bool) {
//         self.selected = selection
//     }
//     fn set_all_vertices(&mut self, selection: bool) {
//         self.v_center.selected = selection;
//         self.v_radii.selected = selection;
//         self.v_start_angle.selected = selection;
//         self.v_end_angle.selected = selection;
//         self.move_selection_end();
//     }
//     fn get_vertex_selected(&self) -> Option<VertexId> {
//         match (
//             self.v_center.selected,
//             self.v_radii.selected,
//             self.v_start_angle.selected,
//             self.v_end_angle.selected,
//         ) {
//             (true, false, false, false) => Some(self.v_center.id),
//             (false, true, false, false) => Some(self.v_radii.id),
//             (false, false, true, false) => Some(self.v_start_angle.id),
//             (false, false, false, true) => Some(self.v_end_angle.id),
//             _ => None,
//         }
//     }
//     fn get_vertex_under_pos(&self, pt: &Point, grab_handle_precision: f64) -> Option<VertexId> {
//         if self.v_center.pt.distance(*pt) < grab_handle_precision {
//             return Some(self.v_center.id);
//         }
//         if self.v_radii.pt.distance(*pt) < grab_handle_precision {
//             return Some(self.v_radii.id);
//         }
//         if self.v_start_angle.pt.distance(*pt) < grab_handle_precision {
//             return Some(self.v_start_angle.id);
//         }
//         if self.v_end_angle.pt.distance(*pt) < grab_handle_precision {
//             return Some(self.v_end_angle.id);
//         }
//         None
//     }
//     fn select_vertex_under_pos(&mut self, pt: &Point, grab_handle_precision: f64) {
//         if self.v_center.pt.distance(*pt) < grab_handle_precision {
//             self.v_center.saved_pt = self.v_center.pt;
//             self.v_center.selected = true;
//             return;
//         }
//         if self.v_radii.pt.distance(*pt) < grab_handle_precision {
//             self.v_radii.saved_pt = self.v_radii.pt;
//             self.v_radii.selected = true;
//             return;
//         }
//         if self.v_start_angle.pt.distance(*pt) < grab_handle_precision {
//             self.v_start_angle.saved_pt = self.v_start_angle.pt;
//             self.v_start_angle.selected = true;
//             return;
//         }
//         if self.v_end_angle.pt.distance(*pt) < grab_handle_precision {
//             self.v_end_angle.saved_pt = self.v_end_angle.pt;
//             self.v_end_angle.selected = true;
//             return;
//         }
//     }
//     fn move_selection(&mut self, dpt: &Point) {
//         if self.selected {
//             match (
//                 self.v_center.selected,
//                 self.v_radii.selected,
//                 self.v_start_angle.selected,
//                 self.v_end_angle.selected,
//             ) {
//                 (true, false, false, false) => {
//                     self.v_center.pt = self.v_center.saved_pt + (dpt.x, dpt.y);
//                     self.arc.center = self.v_center.pt;
//                 }
//                 (false, true, false, false) => {
//                     //TODO
//                 }
//                 (false, false, true, false) => {
//                     //TODO
//                 }
//                 (false, false, false, true) => {
//                     //TODO
//                 }
//                 _ => (),
//             }
//         }
//     }
//     fn move_selection_end(&mut self) {
//         self.v_center.saved_pt = self.v_center.pt;
//         self.v_radii.saved_pt = self.v_radii.pt;
//         self.v_start_angle.saved_pt = self.v_start_angle.pt;
//         self.v_end_angle.saved_pt = self.v_end_angle.pt;
//     }
//     fn get_handles_vertices(&self) -> Vec<Vertex> {
//         vec![
//             self.v_center,
//             self.v_radii,
//             self.v_start_angle,
//             self.v_end_angle,
//         ]
//     }
//     fn get_path(&self, tol: f64) -> BezPath {
//         self.arc.into_path(tol)
//     }
// }
