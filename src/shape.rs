// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use std::collections::{HashMap, HashSet};

use web_sys::console;

use crate::{math::*, types::*};

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
    bss: HashMap<BasicShapeId, (BasicShape, HashSet<PointId>)>,
    pts: HashMap<PointId, (Point, HashSet<BasicShapeId>)>,
    pts_cstr: HashMap<(PointId, PointId), PointConstraint>,
    bss_cstr: HashMap<(BasicShapeId, BasicShapeId), ShapeConstraint>,
}
impl Shape {
    pub fn create(shape: ShapeTypes) -> Shape {
        use ShapeTypes::*;
        match shape {
            Segment(pos1, pos2) => {
                // Vertices
                let mut pts = HashMap::new();
                let (pt1, pt1_id) = Point::new(&pos1, true, true, false);
                let (pt2, pt2_id) = Point::new(&pos2, true, true, true);
                let mut hpts = HashSet::new();
                hpts.insert(pt1_id);
                hpts.insert(pt2_id);

                // Edge
                let (bs, bs_id) = BasicShape::new(BasicShapeType::Line, false);
                let mut bss = HashMap::new();
                bss.insert(bs_id, (bs, hpts));

                let mut hbss = HashSet::new();
                hbss.insert(bs_id);
                pts.insert(pt1_id, (pt1, hbss.clone()));
                pts.insert(pt2_id, (pt2, hbss));

                Shape {
                    selected: false,
                    bss,
                    pts,
                    pts_cstr: HashMap::new(),
                    bss_cstr: HashMap::new(),
                }
            }
            Rectangle(bot_left, top_right) => {
                let top_left = WPos::new(bot_left.wx - 2., top_right.wy);
                let bot_right = WPos::new(top_right.wx + 2., bot_left.wy);

                // Vertices
                let mut pts = HashMap::new();
                let (pt_bot_left, pt_bot_left_id) = Point::new(&bot_left, true, true, false);
                let (pt_top_left, pt_top_left_id) = Point::new(&top_left, true, true, false);
                let (pt_top_right, pt_top_right_id) = Point::new(&top_right, true, true, true);
                let (pt_bot_right, pt_bot_right_id) = Point::new(&bot_right, true, true, false);
                let mut hpts_left_seg = HashSet::new();
                hpts_left_seg.insert(pt_bot_left_id);
                hpts_left_seg.insert(pt_top_left_id);
                let mut hpts_top_seg = HashSet::new();
                hpts_top_seg.insert(pt_top_left_id);
                hpts_top_seg.insert(pt_top_right_id);
                let mut hpts_right_line = HashSet::new();
                hpts_right_line.insert(pt_top_right_id);
                hpts_right_line.insert(pt_bot_right_id);
                let mut hpts_bot_line = HashSet::new();
                hpts_bot_line.insert(pt_bot_right_id);
                hpts_bot_line.insert(pt_bot_left_id);

                // Edges
                let (left_line, id_left_line) = BasicShape::new(BasicShapeType::Line, false);
                let (top_line, id_top_line) = BasicShape::new(BasicShapeType::Line, false);
                let (right_line, id_right_line) = BasicShape::new(BasicShapeType::Line, false);
                let (bot_line, id_bot_line) = BasicShape::new(BasicShapeType::Line, false);
                let mut bss = HashMap::new();
                bss.insert(id_left_line, (left_line, hpts_left_seg));
                bss.insert(id_top_line, (top_line, hpts_top_seg));
                bss.insert(id_right_line, (right_line, hpts_right_line));
                bss.insert(id_bot_line, (bot_line, hpts_bot_line));

                let mut hbss = HashSet::new();
                hbss.insert(id_left_line);
                hbss.insert(id_bot_line);
                pts.insert(pt_bot_left_id, (pt_bot_left, hbss));
                let mut hbss = HashSet::new();
                hbss.insert(id_left_line);
                hbss.insert(id_top_line);
                pts.insert(pt_top_left_id, (pt_top_left, hbss));
                let mut hbss = HashSet::new();
                hbss.insert(id_top_line);
                hbss.insert(id_right_line);
                pts.insert(pt_top_right_id, (pt_top_right, hbss));
                let mut hbss = HashSet::new();
                hbss.insert(id_right_line);
                hbss.insert(id_bot_line);
                pts.insert(pt_bot_right_id, (pt_bot_right, hbss));

                // Add the constrains between vertices
                let mut pts_cstr = HashMap::new();
                // use PointConstraint::*;
                // pts_cstr.insert((pt_bot_left_id, pt_top_left_id), Direction);
                // pts_cstr.insert((pt_top_left_id, pt_top_right_id), Direction);
                // pts_cstr.insert((pt_top_right_id, pt_bot_right_id), Direction);
                // pts_cstr.insert((pt_bot_right_id, pt_bot_left_id), Direction);
                // Add the constrains between edges
                let mut bss_cstr = HashMap::new();
                bss_cstr.insert((id_left_line, id_right_line), ShapeConstraint::Parallel);
                bss_cstr.insert((id_top_line, id_bot_line), ShapeConstraint::Parallel);
                bss_cstr.insert((id_left_line, id_bot_line), ShapeConstraint::Perpendicular);
                Shape {
                    selected: false,
                    bss,
                    pts,
                    pts_cstr,
                    bss_cstr,
                }
            }
        }
    }
    pub fn add_line(&mut self, pos: WPos) -> Option<BasicShapeId> {
        // If a point is selected
        if let Some(pt1_id) = self.is_a_point_selected() {
            if let Some((pt1, hbss_pt1)) = self.pts.get_mut(&pt1_id) {
                // deselect this point
                pt1.selected = false;
                // Create a new selected point and add a new segment with this point
                let (pt2, pt2_id) = Point::new(&pos, true, true, true);
                let mut hpts = HashSet::new();
                hpts.insert(pt1_id);
                hpts.insert(pt2_id);
                let (bs, bs_id) = BasicShape::new(BasicShapeType::Line, false);
                self.bss.insert(bs_id, (bs, hpts));
                hbss_pt1.insert(bs_id);

                let mut hbss_pt2 = HashSet::new();
                hbss_pt2.insert(bs_id);
                self.pts.insert(pt2_id, (pt2, hbss_pt2));
            }
        }
        None
    }
    pub fn is_selected(&self) -> bool {
        self.selected == true
    }
    pub fn set_selected(&mut self, selection: bool) {
        self.selected = selection;
    }
    pub fn set_all_bss_selection(&mut self, selection: bool) {
        self.bss
            .values_mut()
            .for_each(|(bs, _)| bs.selected = selection);
    }
    pub fn clear_all_pts_selection(&mut self) {
        self.pts
            .values_mut()
            .for_each(|(pt, _)| pt.selected = false)
    }
    pub fn is_a_point_selected(&self) -> Option<PointId> {
        for (pt_id, (pt, _)) in self.pts.iter() {
            if pt.selected {
                return Some(*pt_id);
            }
        }
        None
    }
    pub fn set_bs_selection(&mut self, bs_id: &BasicShapeId, selection: bool) {
        if let Some((bs, _)) = self.bss.get_mut(bs_id) {
            bs.selected = selection;
        }
    }
    pub fn set_pt_selection(&mut self, pt_id: &PointId, selection: bool) {
        if let Some((pt, _)) = self.pts.get_mut(pt_id) {
            pt.selected = selection;
        }
    }
    pub fn get_bounded_rectangle(&self) -> [WPos; 2] {
        // TODO
        [WPos::zero(), WPos::zero()]
    }
    pub fn min_bss_s_dist(&self, pos: &WPos) -> Option<(BasicShapeId, f64)> {
        // Return the signed minimum (pos/neg nearest to zero)
        // of all the distances between pos and basic shapes
        let mut min_s_dist = f64::MAX;
        let mut bs_id_min = None;
        for (bs_id, (bs, hpts)) in self.bss.iter() {
            if let Some(s_dist) = bs.s_dist(pos, hpts, &self.pts) {
                if min_s_dist.abs() > s_dist.abs() {
                    min_s_dist = s_dist;
                    bs_id_min = Some(*bs_id);
                }
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
                self.pts
                    .values_mut()
                    .for_each(|(pt, _)| pt.saved_wpos = pt.wpos);
                return true;
            }
        }
        false
    }
    pub fn get_point_under_pos(
        &self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<(BasicShapeId, PointId)> {
        for (bs_id, (_, hpts)) in self.bss.iter() {
            for pt_id in hpts.iter() {
                if let Some((pt, _)) = self.pts.get(pt_id) {
                    if pt.wpos.dist(pick_pos) < grab_handle_precision {
                        return Some((*bs_id, *pt_id));
                    }
                }
            }
        }
        None
    }
    pub fn select_point_under_pos(&mut self, pick_pos: &WPos, grab_handle_precision: f64) {
        if let Some((_, pt_id)) = self.get_point_under_pos(pick_pos, grab_handle_precision) {
            if let Some((pt, _)) = self.pts.get_mut(&pt_id) {
                pt.selected = true;
                pt.saved_wpos = pt.wpos;
            }
        }
    }
    pub fn move_elements(&mut self, delta_pick_pos: &WPos) {
        // If a point is selected, we move the point
        // If no point is selected we move all the position points
        if let Some(pt_id) = self.is_a_point_selected() {
            self.move_point(&pt_id, delta_pick_pos);
            // Apply point cstr
            //self.apply_pts_cstr(&pt_id);
            // Seek bs belonging to the point and apply bs cstr
            self.apply_bss_cstr(&pt_id);
        } else {
            self.move_all_points(delta_pick_pos);
        }
    }
    pub fn move_point(&mut self, pt_id: &PointId, delta_pick_pos: &WPos) {
        if let Some((pt, _)) = self.pts.get_mut(pt_id) {
            pt.wpos = pt.saved_wpos + *delta_pick_pos;
        }
    }

    pub fn bs_get_line_pts(&self, bs_id: &BasicShapeId) -> Option<(Point, Point)> {
        if let Some((bs, hpts)) = self.bss.get(&bs_id) {
            if let BasicShapeType::Line = bs.bs_typ {
                let v: Vec<PointId> = hpts.iter().cloned().collect();
                if v.len() == 2 {
                    if let Some((pt0, _)) = self.pts.get(&v[0]).cloned() {
                        if let Some((pt1, _)) = self.pts.get(&v[1]).cloned() {
                            return Some((pt0, pt1));
                        }
                    }
                }
            }
        }
        None
    }

    pub fn apply_pts_cstr(&mut self, pt_id: &PointId) {
        for ((pt1_id, pt2_id), constraint) in self.pts_cstr.iter() {
            if pt1_id == pt_id || pt2_id == pt_id {
                use PointConstraint::*;
                match constraint {
                    Equal => (),
                    VerticalAlign => (),
                    HorizontalAlign => (),
                    Direction => {
                        if pt1_id == pt_id {
                            if let Some((pt1, _)) = self.pts.get(pt1_id).cloned() {
                                if let Some((pt2, _)) = self.pts.get_mut(pt2_id) {
                                    apply_cstr_direction(pt2, &pt1);
                                }
                            }
                        } else {
                            if let Some((pt2, _)) = self.pts.get(pt2_id).cloned() {
                                if let Some((pt1, _)) = self.pts.get_mut(pt1_id) {
                                    apply_cstr_direction(pt1, &pt2);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn apply_bss_cstr(&mut self, pt_id: &PointId) {
        for ((bs1_id, bs2_id), constraint) in self.bss_cstr.iter() {
            match constraint {
                ShapeConstraint::Parallel => {
                    if let Some(bs1_pts) = self.bs_get_line_pts(bs1_id) {
                        if let Some(bs2_pts) = self.bs_get_line_pts(&bs2_id) {
                            let (next_bs1_pts, next_bs2_pts) =
                                apply_cstr_parallel(pt_id, &bs1_pts, &bs2_pts);
                            if let Some((mut_pt, _)) = self.pts.get_mut(&next_bs1_pts.0.id) {
                                mut_pt.wpos = next_bs1_pts.0.wpos;
                            }
                            if let Some((mut_pt, _)) = self.pts.get_mut(&next_bs1_pts.1.id) {
                                mut_pt.wpos = next_bs1_pts.1.wpos;
                            }
                            if let Some((mut_pt, _)) = self.pts.get_mut(&next_bs2_pts.0.id) {
                                mut_pt.wpos = next_bs2_pts.0.wpos;
                            }
                            if let Some((mut_pt, _)) = self.pts.get_mut(&next_bs2_pts.1.id) {
                                mut_pt.wpos = next_bs2_pts.1.wpos;
                            }
                        }
                    }
                }
                ShapeConstraint::Perpendicular => {
                    if let Some(bs1_pts) = self.bs_get_line_pts(bs1_id) {
                        if let Some(bs2_pts) = self.bs_get_line_pts(&bs2_id) {
                            let (next_bs1_pts, next_bs2_pts) =
                                apply_cstr_perpendicular(pt_id, &bs1_pts, &bs2_pts);
                            if let Some((mut_pt, _)) = self.pts.get_mut(&next_bs1_pts.0.id) {
                                mut_pt.wpos = next_bs1_pts.0.wpos;
                            }
                            if let Some((mut_pt, _)) = self.pts.get_mut(&next_bs1_pts.1.id) {
                                mut_pt.wpos = next_bs1_pts.1.wpos;
                            }
                            if let Some((mut_pt, _)) = self.pts.get_mut(&next_bs2_pts.0.id) {
                                mut_pt.wpos = next_bs2_pts.0.wpos;
                            }
                            if let Some((mut_pt, _)) = self.pts.get_mut(&next_bs2_pts.1.id) {
                                mut_pt.wpos = next_bs2_pts.1.wpos;
                            }
                        }
                    }
                }
            }
        }
    }
    pub fn move_all_points(&mut self, delta_pick_pos: &WPos) {
        let mut pt_to_move = HashSet::new();
        for (_, (_, hpts)) in self.bss.iter() {
            hpts.iter().for_each(|pt_id| _ = pt_to_move.insert(*pt_id));
        }
        for pt_id in pt_to_move.iter() {
            self.move_point(pt_id, delta_pick_pos);
        }
    }
    pub fn get_bss_constructions(&self, cst: &mut Vec<ConstructionType>) {
        for (_, (bs, hpts)) in self.bss.iter() {
            if let Some(cst_bs) = bs.get_bss_constructions(cst, self.selected, hpts, &self.pts) {
                cst.push(cst_bs)
            }
        }
    }
    pub fn get_handles_construction(&self, cst: &mut Vec<ConstructionType>, size_handle: f64) {
        for (_, (pt, _)) in self.pts.iter() {
            push_handle(cst, &pt, size_handle);
        }
    }
    pub fn get_helpers_construction(&self, cst: &mut Vec<ConstructionType>) {
        for (_, (bs, hpts)) in self.bss.iter() {
            bs.get_helpers_construction(cst, hpts, &self.pts);
        }
    }
}
