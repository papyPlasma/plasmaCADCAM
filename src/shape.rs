// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use ordered_float::OrderedFloat;

use crate::{math::*, types::*};
use std::{
    collections::{HashMap, HashSet},
    f64::consts::PI,
};

pub enum ShapeTypes {
    Segment(WPos, WPos),
    Rectangle(WPos, WPos),
    // Circle,
    // Ellipse,
    // ArcCircle,
    // ArcEllipse,
    // RoundedRectangle,
    // Triangle,
    // QuadBezier,
    // CubicBezier,
    // Custom,
}

pub struct Shape {
    selected: bool,
    primitives: HashMap<PrimitiveId, Primitive>,
    points: HashMap<PointId, Point>,
    point_to_primitive: HashMap<PointId, PrimitiveId>,
    //
    points_constraints: HashMap<PointIdCouple, PointsBinaryConstraint>,
}
impl Shape {
    fn create_segment(
        pos1: &WPos,
        pos2: &WPos,
        end_selected: bool,
        points: &mut HashMap<PointId, Point>,
        primitives: &mut HashMap<PrimitiveId, Primitive>,
        points_to_primitives: &mut HashMap<PointId, PrimitiveId>,
    ) -> (PointId, PointId, PrimitiveId) {
        let (pt1, pt1_id) = Point::new(&pos1, true, true, false);
        let (pt2, pt2_id) = Point::new(&pos2, true, true, end_selected);
        points.insert(pt1_id, pt1);
        points.insert(pt2_id, pt2);

        let (prim, prim_id) = Primitive::new(&PrimitiveType::Segment(pt1_id, pt2_id), false);
        primitives.insert(prim_id, prim);

        points_to_primitives.insert(pt1_id, prim_id);
        points_to_primitives.insert(pt2_id, prim_id);

        (pt1_id, pt2_id, prim_id)
    }
    pub fn create(shape: ShapeTypes) -> Shape {
        use ShapeTypes::*;
        match shape {
            Segment(pos1, pos2) => {
                let mut points = HashMap::new();
                let mut primitives = HashMap::new();
                let mut points_to_primitives = HashMap::new();

                Shape::create_segment(
                    &pos1,
                    &pos2,
                    true,
                    &mut points,
                    &mut primitives,
                    &mut points_to_primitives,
                );

                Shape {
                    selected: false,
                    primitives,
                    points,
                    point_to_primitive: points_to_primitives,
                    points_constraints: HashMap::new(),
                }
            }

            Rectangle(pos_bl, pos_tr) => {
                let pos_tl = WPos::new(pos_bl.wx - 2., pos_tr.wy);
                let pos_br = WPos::new(pos_tr.wx + 2., pos_bl.wy);

                let mut points = HashMap::new();
                let mut primitives = HashMap::new();
                let mut points_to_primitives = HashMap::new();

                let (pt_bl_a_id, pt_tl_a_id, prim_ll_id) = Shape::create_segment(
                    &pos_bl,
                    &pos_tl,
                    false,
                    &mut points,
                    &mut primitives,
                    &mut points_to_primitives,
                );
                let (pt_tl_b_id, pt_tr_a_id, prim_lt_id) = Shape::create_segment(
                    &pos_tl,
                    &pos_tr,
                    true,
                    &mut points,
                    &mut primitives,
                    &mut points_to_primitives,
                );
                let (pt_tr_b_id, pt_br_a_id, prim_lr_id) = Shape::create_segment(
                    &pos_tr,
                    &pos_br,
                    false,
                    &mut points,
                    &mut primitives,
                    &mut points_to_primitives,
                );
                let (pt_br_b_id, pt_bl_b_id, prim_lb_id) = Shape::create_segment(
                    &pos_br,
                    &pos_bl,
                    false,
                    &mut points,
                    &mut primitives,
                    &mut points_to_primitives,
                );

                // Rectangle constraints
                use PrimitiveConstraint::*;
                if let Some(line_left) = primitives.get_mut(&prim_ll_id) {
                    line_left.set_constraint(&SegParallel(OrderedFloat(PI / 2.)));
                }
                if let Some(line_right) = primitives.get_mut(&prim_lr_id) {
                    line_right.set_constraint(&SegParallel(OrderedFloat(PI / 2.)));
                }
                if let Some(line_top) = primitives.get_mut(&prim_lt_id) {
                    line_top.set_constraint(&SegParallel(OrderedFloat(0.)));
                }
                if let Some(line_bottom) = primitives.get_mut(&prim_lb_id) {
                    line_bottom.set_constraint(&SegParallel(OrderedFloat(0.)));
                }
                let mut points_constraints = HashMap::new();
                use PointsBinaryConstraint::*;
                points_constraints.insert(PointIdCouple(pt_bl_a_id, pt_bl_b_id), Binded);
                points_constraints.insert(PointIdCouple(pt_tl_a_id, pt_tl_b_id), Binded);
                points_constraints.insert(PointIdCouple(pt_tr_a_id, pt_tr_b_id), Binded);
                points_constraints.insert(PointIdCouple(pt_br_a_id, pt_br_b_id), Binded);

                Shape {
                    selected: false,
                    primitives,
                    points,
                    point_to_primitive: points_to_primitives,
                    points_constraints,
                }
            }
        }
    }
    // pub fn add_line(&mut self, pos: WPos) -> Option<PrimitiveId> {
    //     // If a point is selected
    //     if let Some(ae1_id) = self.is_a_point_selected() {
    //         if let Some(ae) = self.points.get_mut(&ae1_id) {
    //             if let Point::AEPoint(pt1) = ae {
    //                 // deselect this point
    //                 pt1.selected = false;
    //                 // Create a new selected point and add a new segment with this point
    //                 let (pt2, ae2_id) = Point::new(&pos, true, true, true);
    //                 let (pc1, ae3_id) = PolarCoord::new(&pos_to_polar(&pt1.wpos, &pos));
    //                 // Create and store the Line
    //                 let mut hs_aes = HashSet::new();
    //                 hs_aes.insert(ae3_id);
    //                 let (bs, bs_id) = Primitive::new(PrimitiveType::Line, false);
    //                 self.primitives.insert(bs_id, (bs, hs_aes));
    //                 // Constraints the two points to belong to the line
    //                 self.constraints
    //                     .insert(Constraint::OnBasicShape(ae1_id, bs_id));
    //                 self.constraints
    //                     .insert(Constraint::OnBasicShape(ae2_id, bs_id));
    //             }
    //         }
    //     }
    //     None
    // }
    pub fn is_selected(&self) -> bool {
        self.selected == true
    }
    pub fn set_selected(&mut self, selection: bool) {
        self.selected = selection;
    }
    pub fn set_all_bss_selection(&mut self, selection: bool) {
        self.primitives
            .values_mut()
            .for_each(|prim| prim.selected = selection);
    }
    pub fn clear_all_pts_selection(&mut self) {
        self.points.values_mut().for_each(|pt| pt.selected = false)
    }
    pub fn is_a_point_selected(&self) -> Option<PointId> {
        for (pt_id, pt) in self.points.iter() {
            if pt.selected {
                return Some(*pt_id);
            }
        }
        None
    }
    pub fn set_bs_selection(&mut self, bs_id: &PrimitiveId, selection: bool) {
        if let Some(prim) = self.primitives.get_mut(bs_id) {
            prim.selected = selection;
        }
    }
    pub fn set_pt_selection(&mut self, pt_id: &PointId, selection: bool) {
        if let Some(pt) = self.points.get_mut(pt_id) {
            pt.selected = selection;
        }
    }
    pub fn get_bounded_rectangle(&self) -> [WPos; 2] {
        // TODO
        [WPos::zero(), WPos::zero()]
    }
    pub fn min_bss_s_dist(&self, pos: &WPos) -> Option<(PrimitiveId, f64)> {
        // Return the signed minimum (pos/neg nearest to zero)
        // of all the distances between pos and basic shapes
        let mut min_s_dist = f64::MAX;
        let mut bs_id_min = None;
        for (prim_id, prim) in self.primitives.iter() {
            let s_dist = prim.s_dist(pos, &self.points);
            if min_s_dist.abs() > s_dist.abs() {
                min_s_dist = s_dist;
                bs_id_min = Some(*prim_id);
            }
        }
        if let Some(bs_id) = bs_id_min {
            Some((bs_id, min_s_dist))
        } else {
            None
        }
    }
    pub fn select_shape_under_pos(&mut self, pick_pos: &WPos, grab_handle_precision: f64) -> bool {
        if let Some((_, s_dist)) = self.min_bss_s_dist(pick_pos) {
            if s_dist.abs() < grab_handle_precision {
                self.selected = true;
                self.points
                    .values_mut()
                    .for_each(|pt| pt.saved_wpos = pt.wpos);
                return true;
            }
        }
        false
    }
    pub fn get_point_under_pos(
        &self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<(PrimitiveId, PointId)> {
        for (pt_id, pt) in self.points.iter() {
            if pt.wpos.dist(pick_pos) < grab_handle_precision {
                if let Some(prim_id) = self.point_to_primitive.get(pt_id) {
                    return Some((*prim_id, *pt_id));
                }
            }
        }
        None
    }
    pub fn select_point_under_pos(&mut self, pick_pos: &WPos, grab_handle_precision: f64) {
        if let Some((_, pt_id)) = self.get_point_under_pos(pick_pos, grab_handle_precision) {
            if let Some(pt) = self.points.get_mut(&pt_id) {
                pt.selected = true;
                pt.saved_wpos = pt.wpos;
            }
        }
    }

    pub fn move_elements(&mut self, dpos: &WPos) {
        // If a point is selected, we move the point
        if let Some(pt_id) = self.is_a_point_selected() {
            // and move
            self.move_points(&pt_id, dpos);
        } else {
            // If no points are selected we move all the position points
            self.move_all_points(dpos);
        }
    }

    fn get_binded_pts(&self, pt_id: &PointId, pts: &mut HashMap<PointId, PointUnaryConstraint>) {
        // First
        pts.insert(*pt_id, PointUnaryConstraint::FreeToMove);
        self.points_constraints.iter().for_each(|(ids, cstr)| {
            if let PointsBinaryConstraint::Binded = cstr {
                pts.insert(ids.0, PointUnaryConstraint::FreeToMove);
                pts.insert(ids.1, PointUnaryConstraint::FreeToMove);
            }
        });
    }

    fn move_points(&mut self, pt_id: &PointId, delta_pick_pos: &WPos) {
        // First get all points binded to this point (PointsBinaryConstraint::Binded)
        // Hence, all those points are required to move the same way
        // pts_to_move contains also pt_id
        let mut pts_to_move = HashMap::new();
        self.get_binded_pts(pt_id, &mut pts_to_move);

        // Second, for pts_to_move check for fixed point, if there is any fixed point, stop
        for pt_id in pts_to_move.keys() {
            if let Some(pt) = self.points.get(pt_id) {
                if let PointUnaryConstraint::Fixed = pt.csrt {
                    // This point can't be moved, hence neither the binded points, stop
                    return;
                }
            }
        }

        // Third, for pts_to_move get the points constraints from the primitives they belong to
        // For each point of pts_to_move, get its unique primitive (if any) and check the constraint on
        // this primitive. Regarding the constraint, it is possible that other points belonging
        // to the primitive has to be moved
        for (pt_id, csrt) in pts_to_move.iter_mut() {
            if let Some(prim_id) = self.point_to_primitive.get(pt_id) {
                if let Some(prim) = self.primitives.get(prim_id) {
                    use PrimitiveConstraint::*;
                    use PrimitiveType::*;
                    match prim.prim_type {
                        Segment(pt1_id, pt2_id) => match prim.prim_cstr {
                            Unconstrained =>
                            // The segment is unconstrained, hence the point remains FreeToMove
                            {
                                ()
                            }
                            _ =>
                            // The segment is constrained, hence its move will depend of
                            // the other point constraint, let's check this other point,
                            {
                                let other_pt_id = if *pt_id == pt1_id { pt2_id } else { pt1_id };
                                if let Some(other_pt) = self.points.get(&other_pt_id) {
                                    if let PointUnaryConstraint::Fixed = other_pt.csrt {
                                        // This point can't be moved, since segment is constrained,
                                        // No DOF remains and the pt_id can't be moved neither, stop
                                        return;
                                    }
                                    // Check if this other_pt point is binded to third point
                                    let mut other_pt_binds = HashSet::new();
                                    for (c, other_pt_csrt) in self.points_constraints.iter() {
                                        if c.0 == other_pt_id {
                                            if let PointsBinaryConstraint::Binded = other_pt_csrt {
                                                other_pt_binds.insert(c.1);
                                            };
                                        }
                                        if c.1 == other_pt_id {
                                            if let PointsBinaryConstraint::Binded = other_pt_csrt {
                                                other_pt_binds.insert(c.0);
                                            }
                                        }
                                    }
                                    // Check constraints of other_pt_binds points
                                    for third_pt_id in other_pt_binds.iter() {
                                        if let Some(third_pt) = self.points.get(third_pt_id) {
                                            if let PointUnaryConstraint::Fixed = third_pt.csrt {
                                                // This point can't be moved, hence neither the related points, stop
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        QBezier(_, _, _) => (),       //TODO
                        CBezier(_, _, _, _) => (),    //TODO
                        ArcEllipse(_, _, _, _) => (), //TODO
                    }
                }
            }
        }

        // Resolve constraints

        if let Some(pt) = self.points.get_mut(pt_id) {
            pt.wpos = pt.saved_wpos + *delta_pick_pos;
        }
    }

    pub fn bs_get_line_pts(&self, bs_id: &PrimitiveId) -> Option<(Point, Point)> {
        if let Some(prim) = self.primitives.get(&bs_id) {
            if let PrimitiveType::Segment(pt0_id, pt1_id) = prim.prim_type {
                if let Some(pt0) = self.points.get(&pt0_id) {
                    if let Some(pt1) = self.points.get(&pt1_id) {
                        return Some((*pt0, *pt1));
                    }
                }
            }
        }
        None
    }
    pub fn move_all_points(&mut self, delta_pick_pos: &WPos) {
        for (_, prim) in self.primitives.iter_mut() {
            prim.move_points(&mut self.points, delta_pick_pos);
        }
    }
    pub fn get_bss_constructions(&self, cst: &mut Vec<ConstructionType>) {
        for (_, prim) in self.primitives.iter() {
            prim.get_bss_constructions(cst, &self.points, self.selected);
        }
    }
    pub fn get_handles_construction(&self, cst: &mut Vec<ConstructionType>) {
        for (_, pt) in self.points.iter() {
            push_handle(cst, &pt);
        }
    }
    pub fn get_helpers_construction(&self, cst: &mut Vec<ConstructionType>) {
        for (_, prim) in self.primitives.iter() {
            prim.get_helpers_constructions(cst, &self.points);
        }
    }
}
