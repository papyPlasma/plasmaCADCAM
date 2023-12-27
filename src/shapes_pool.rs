// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use crate::math::*;
use crate::shape::*;
use crate::types::*;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct ShapesPool(HashMap<ShapeId, Box<Shape>>);
impl std::ops::Deref for ShapesPool {
    type Target = HashMap<ShapeId, Box<Shape>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for ShapesPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ShapesPool {
    pub fn new() -> ShapesPool {
        log!("Creating datapools");
        ShapesPool(HashMap::new())
    }

    pub fn insert_shape(&mut self, shape: Box<Shape>) -> ShapeId {
        let sh_id = ShapeId::new_id();
        self.insert(sh_id, shape);
        sh_id
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

    pub fn clear_shapes_selections(&mut self) {
        for shape in self.values_mut() {
            shape.set_selected(false);
            shape.set_all_bss_selection(false);
            shape.clear_all_pts_selection();
        }
    }
    pub fn delete_selected_shapes(&mut self) {
        self.retain(|_, shape| !shape.is_selected());
    }
    pub fn _set_shape_selected(&mut self, sh_id: &ShapeId, selection: bool) {
        if let Some(shape) = self.get_mut(sh_id) {
            shape.set_selected(selection)
        }
    }
    pub fn _get_shape_under_pos(
        &self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<ShapeId> {
        for (sh_id, shape) in self.iter() {
            if shape.is_shape_under_pos(pick_pos, grab_handle_precision) {
                return Some(*sh_id);
            }
        }
        None
    }

    pub fn select_all_under_pos(&mut self, pick_pos: &WPos, grab_handle_precision: f64) {
        for shape in self.values_mut() {
            if shape.is_shape_under_pos(pick_pos, grab_handle_precision) {
                shape.set_selected(true);
                shape.select_point_under_pos(pick_pos, grab_handle_precision);
            }
        }
    }

    pub fn move_selection(&mut self, pick_delta_pos: &WPos, _magnet_distance: f64) {
        for shape in self.values_mut() {
            if shape.is_selected() {
                shape.move_elements(&pick_delta_pos);
            }
        }
    }
    pub fn select_shapes_bounded_by_rectangle(&mut self, bb_outer: [WPos; 2]) {
        // for (sh_id, shape) in self.shapes_pool.iter_mut() {
        //     let bb_inner = shape.get_bounded_rectangle();
        //     if is_box_inside(&bb_outer, &bb_inner) {
        //         // shape.set_selected(true);
        //         self.selections.insert(*sh_id);
        //     }
        // }
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

    // pub fn _get_shape_position(&self, sh_id: &ShapeId) -> WPos {
    //     self.shapes_pool.get(sh_id).unwrap().get_pos()
    // }
    pub fn magnet_to_point(
        &self,
        pick_pos: &mut WPos,
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
    // ) -> Option<WPos> {
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
    //     pos_init: &WPos,
    //     stepping: f64,
    //     dir: bool,
    // ) -> Option<WPos> {
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

    // pub fn cut_shape(&mut self, sh_id: &ShapeId, pick_pos: &WPos, grab_handle_precision: f64) {
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
    //     let o_pos_p: Option<WPos> = self.seek_intersection(&sh_id, &pos, stepping, true);
    //     // Search intersection with another shape in the second direction
    //     let o_pos_n: Option<WPos> = self.seek_intersection(&sh_id, &pos, stepping, false);

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
