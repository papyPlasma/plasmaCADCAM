// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use kurbo::Point;

use crate::bindings::*;
use crate::math::*;
use crate::shape::*;
use crate::types::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;

pub struct VerticesPool(HashMap<VertexId, Vertex>);
impl Deref for VerticesPool {
    type Target = HashMap<VertexId, Vertex>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for VerticesPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl VerticesPool {
    pub fn new() -> VerticesPool {
        VerticesPool(HashMap::new())
    }
    pub fn add(&mut self, pt: Point) -> Vertex {
        let id = VertexId::new_id();
        let v = Vertex {
            id,
            pt,
            saved_pt: pt,
            magnetic: true,
            draggable: true,
            selected: false,
        };
        self.insert(id, v);
        v
    }
}

pub struct ShapesPool(HashMap<ShapeId, Shapes>);
impl Deref for ShapesPool {
    type Target = HashMap<ShapeId, Shapes>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ShapesPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ShapesPool {
    pub fn new() -> ShapesPool {
        log!("Creating shapes_pool");
        ShapesPool(HashMap::new())
    }
    pub fn add_line(&mut self, va: &Vertex, vb: &Vertex) -> LineShape {
        let id: ShapeId = ShapeId::new_id();
        let line = LineShape::new(id, va, vb);
        self.insert(id, Shapes::STLine(line.clone()));
        line
    }
    pub fn is_any_shape_selected(&mut self) -> bool {
        let mut selected = false;
        for shape in self.values_mut() {
            if shape.is_selected() {
                selected = true;
                break;
            }
        }
        selected
    }
    pub fn delete_selected_shapes(&mut self) {
        self.retain(|_, shape| !shape.is_selected());
    }
    pub fn select_shapes_bounded_by_rectangle(&mut self, bb_outer: [Point; 2]) {
        for (_sh_id, shape) in self.iter_mut() {
            let bb_inner = shape.get_bounded_rectangle();
            if is_box_inside(&bb_outer, &bb_inner) {
                // shape.set_selected(true);
                // self.selections.insert(*sh_id);
            }
        }
    }

    // pub fn create_group_id(&mut self) -> GroupId {
    //     self.groups_pool.create_id()
    // }
    // pub fn _set_shape_group(&mut self, gr_id: &GroupId, sh_id: &ShapeId) {
    //     self.groups_pool.insert_shape_id(gr_id, sh_id);
    // }
    // pub fn _get_shape_group(&mut self, gr_id: &GroupId) -> Option<&Vec<ShapeId>> {
    //     self.groups_pool.get(gr_id)
    // }

    pub fn magnet_to_point(
        &self,
        pick_pos: &mut Point,
        excluded_sh_id: Option<ShapeId>,
        magnet_distance: f64,
    ) {
        // // Test all all shapes points but not the one that is excluded
        // for (sh_id, shape) in self.shapes_pool.iter() {
        //     let exclude = if let Some(exc_sh_id) = excluded_sh_id {
        //         if *sh_id == exc_sh_id {
        //             true
        //         } else {
        //             false
        //         }
        //     } else {
        //         false
        //     };
        //     if !exclude {
        //         shape.magnet_to_point(pick_pos, magnet_distance);
        //     }
        // }
    }

    // fn seek_intersection_precise(
    //     &self,
    //     sh_id: &ShapeId,
    //     other_sh_id: &ShapeId,
    //     r_a: f64,
    //     r_b: f64,
    // ) -> Option<Point> {
    //     let shape = self.shapes_pool.get(&sh_id).unwrap();
    //     let other_shape = self.shapes_pool.get(&other_sh_id).unwrap();
    //     let stepping_r = shape.get_step_r(EPSILON);
    //     // Linear search, distance are not signed, can't do a binary search
    //     let mut r = r_a;
    //     loop {
    //         let pos = shape.get_pos_from_ratio(r);
    //         if other_shape.dist(&pos) < EPSILON {
    //             return Some(pos);
    //         }
    //         r += stepping_r;
    //         if r >= r_b {
    //             break;
    //         }
    //     }
    //     None
    // }

    // fn seek_intersection(
    //     &self,
    //     sh_id: &ShapeId,
    //     pos_init: &Point,
    //     stepping: f64,
    //     dir: bool,
    // ) -> Option<Point> {
    //     let shape = self.shapes_pool.get(&sh_id).unwrap();
    //     let r_init = shape.get_ratio_from_pos(pos_init);
    //     let stepping_r = shape.get_step_r(stepping);
    //     let mut r = if dir {
    //         (r_init + stepping_r + 0.0001).min(1.0)
    //     } else {
    //         (r_init - stepping_r - 0.0001).max(0.0)
    //     };
    //     let mut count = 0;
    //     loop {
    //         let pos = shape.get_pos_from_ratio(r);
    //         log!("CURRENT r: {}", r);
    //         for (other_sh_id, other_shape) in self.shapes_pool.iter().filter(|(id, _)| id != &sh_id)
    //         {
    //             log!("other_sh_id: {}, dir: {}", other_sh_id.0, dir);
    //             if other_shape.dist(&pos) <= stepping {
    //                 // Set the range for the precise seek
    //                 let (r_a, r_b) = if dir {
    //                     ((r - stepping_r).max(0.0), (r + stepping_r).min(1.0))
    //                 } else {
    //                     ((r - 2. * stepping_r).max(0.0), r)
    //                 };
    //                 log!("dir: {}, r_a: {:.2}, r_b: {:.2}", dir, r_a, r_b);
    //                 // And search precisely
    //                 // If a position is found return it
    //                 if let Some(pos_int) =
    //                     self.seek_intersection_precise(&sh_id, &other_sh_id, r_a, r_b)
    //                 {
    //                     return Some(pos_int);
    //                 };
    //             }
    //         }
    //         r = if dir {
    //             (r + stepping_r).min(1.0)
    //         } else {
    //             (r - stepping_r).max(0.0)
    //         };
    //         if r == 0. || r == 1. {
    //             break;
    //         }
    //         count += 1;
    //         if count == 1000 {
    //             log!("Couille dans l'potage");
    //             break;
    //         }
    //     }
    //     None
    // }

    // pub fn cut_shape(&mut self, sh_id: &ShapeId, pick_pos: &Point, grab_handle_precision: f64) {
    //     let pos = self
    //         .shapes_pool
    //         .get(&sh_id)
    //         .unwrap()
    //         .get_projected_pos(&pick_pos);
    //     log!(
    //         "pick_pos: ({:.0}, {:.0}) proj_pos: ({:.0}, {:.0})",
    //         pick_pos.wx,
    //         pick_pos.wy,
    //         pos.wx,
    //         pos.wy
    //     );
    //     let stepping = grab_handle_precision / 10.;
    //     // Search intersection with another shape in the first direction
    //     let o_pos_p: Option<Point> = self.seek_intersection(&sh_id, &pos, stepping, true);
    //     // Search intersection with another shape in the second direction
    //     let o_pos_n: Option<Point> = self.seek_intersection(&sh_id, &pos, stepping, false);
    //     if let Some(pos_p) = o_pos_p {
    //         log!("pos_p: ({:.0},{:.0})", pos_p.wx, pos_p.wy);
    //     } else {
    //         log!("pos_p: None");
    //     }
    //     if let Some(pos_n) = o_pos_n {
    //         log!("pos_n: ({:.0},{:.0})", pos_n.wx, pos_n.wy);
    //     } else {
    //         log!("pos_n: None");
    //     }
    //     // inf and sup names are convention
    //     match (o_pos_n, o_pos_p) {
    //         (None, None) => {
    //             // Nothing to do here, there are no intersections with other shapes
    //             // then this shape will be simply deleted
    //             ()
    //         }
    //         (None, Some(pos_p)) => {
    //             if let Some(sh_sup) = self.shapes_pool.get(&sh_id).unwrap().split(&pos_p).1 {
    //                 self.insert_shape(sh_sup);
    //             }
    //         }
    //         (Some(pos_n), None) => {
    //             let shape = self.shapes_pool.get(&sh_id).unwrap();
    //             if let Some(sh_inf) = shape.split(&pos_n).0 {
    //                 self.insert_shape(sh_inf);
    //             }
    //         }
    //         (Some(pos_n), Some(pos_p)) => {
    //             if let Some(shape_n) = self.shapes_pool.get(&sh_id).unwrap().split(&pos_n).0 {
    //                 self.insert_shape(shape_n);
    //             }
    //             if let Some(shape_p) = self.shapes_pool.get(&sh_id).unwrap().split(&pos_p).1 {
    //                 self.insert_shape(shape_p);
    //             }
    //         }
    //     }
    //     // Suppress the original shape from the pool
    //     self.shapes_pool.remove(&sh_id);
    // }
}

#[derive(Clone, Debug)]
pub struct BindingsPool(HashMap<BindingId, Binding>);
impl Deref for BindingsPool {
    type Target = HashMap<BindingId, Binding>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for BindingsPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(dead_code)]
impl BindingsPool {
    pub fn new() -> BindingsPool {
        BindingsPool(HashMap::new())
    }
    pub fn add_bind_fixed(&mut self, v: &Vertex) -> BindFixed {
        let id = BindingId::new_id();
        let bind = BindFixed {
            id,
            fixed_value: v.pt,
            v_id: v.id,
        };
        self.insert(id, Binding::Fixed(bind.clone()));
        bind
    }
    pub fn add_bind_fixed_x(&mut self, v: &Vertex) -> BindFixedX {
        let id = BindingId::new_id();
        let bind = BindFixedX {
            id,
            fixed_value: v.pt.x,
            v_id: v.id,
        };
        self.insert(id, Binding::FixedX(bind.clone()));
        bind
    }
    pub fn add_bind_fixed_y(&mut self, v: &Vertex) -> BindFixedY {
        let id = BindingId::new_id();
        let bind = BindFixedY {
            id,
            fixed_value: v.pt.y,
            v_id: v.id,
        };
        self.insert(id, Binding::FixedY(bind.clone()));
        bind
    }
    pub fn add_bind_vertical(&mut self, seg: (&Vertex, &Vertex)) -> BindVertical {
        let id = BindingId::new_id();
        let bind = BindVertical {
            id,
            va_id: seg.0.id,
            vb_id: seg.1.id,
        };
        self.insert(id, Binding::Vertical(bind.clone()));
        bind
    }
    pub fn add_bind_horizontal(&mut self, seg: (&Vertex, &Vertex)) -> BindHorizontal {
        let id = BindingId::new_id();
        let bind = BindHorizontal {
            id,
            va_id: seg.0.id,
            vb_id: seg.1.id,
        };
        self.insert(id, Binding::Horizontal(bind.clone()));
        bind
    }
    pub fn add_bind_parallel(
        &mut self,
        seg1: (&Vertex, &Vertex),
        seg2: (&Vertex, &Vertex),
    ) -> BindParallel {
        let id = BindingId::new_id();
        let bind = BindParallel {
            id,
            l1va_id: seg1.0.id,
            l1vb_id: seg1.1.id,
            l2va_id: seg2.0.id,
            l2vb_id: seg2.1.id,
        };
        self.insert(id, Binding::Parallel(bind.clone()));
        bind
    }
    pub fn add_bind_distance(&mut self, seg: (&Vertex, &Vertex)) -> BindDistance {
        let id = BindingId::new_id();
        let bind = BindDistance {
            id,
            sq_distance_value: seg.0.dist_sq(seg.1),
            va_id: seg.0.id,
            vb_id: seg.1.id,
        };
        self.insert(id, Binding::Distance(bind.clone()));
        bind
    }
}

// pub struct GroupsPool(HashMap<GroupId, Vec<ShapeId>>);
// impl std::ops::Deref for GroupsPool {
//     type Target = HashMap<GroupId, Vec<ShapeId>>;
//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }
// impl std::ops::DerefMut for GroupsPool {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }
// impl GroupsPool {
//     pub fn new() -> GroupsPool {
//         GroupsPool(HashMap::new())
//     }
//     pub fn create_id(&mut self) -> GroupId {
//         GroupId::new_id()
//     }
//     pub fn insert_shape_id(&mut self, grp_id: &GroupId, sh_id: &ShapeId) {
//         if let Some(sh_ids) = self.get_mut(grp_id) {
//             sh_ids.push(*sh_id);
//         } else {
//             self.0.insert(*grp_id, vec![*sh_id]);
//         }
//     }
//     pub fn _get_shapes_ids(&self, grp_id: &GroupId) -> Option<Vec<ShapeId>> {
//         self.0.get(grp_id).cloned()
//     }
//     pub fn delete_shape_id(&mut self, sh_id: &ShapeId) {
//         for sh_ids in self.values_mut() {
//             sh_ids.retain(|vec_sh_id| vec_sh_id != sh_id)
//         }
//     }
// }
