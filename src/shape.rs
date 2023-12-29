use std::collections::{HashMap, HashSet};

use crate::{math::*, types::*};

pub enum ShapeTypes {
    Line(WPos, WPos),
    Rectangle(WPos, WPos),
    // Circle,
    // Ellipse,
    // ArcCircle,
    // ArcEllipse,
    // RoundedRectangle,
    // Triangle,
    // Polygon,
    // QuadBezier,
    // CubicBezier,
    // Custom,
}

pub struct Shape {
    selected: bool,
    bss: HashMap<BasicShapeId, (BasicShapeType, bool, HashMap<PointType, PointId>)>,
    pts: HashMap<PointId, (Point, HashSet<BasicShapeId>)>,
    constraints: HashMap<(PointId, PointId), PointConstraint>,
}
impl Shape {
    pub fn create(shape: ShapeTypes) -> Shape {
        use ShapeTypes::*;
        match shape {
            Line(start, end) => {
                // Vertices
                let mut pts = HashMap::new();
                let (pt_start, pt_start_id) = Point::new(&start, true, true, false);
                let (pt_end, pt_end_id) = Point::new(&end, true, true, true);
                let mut hpts = HashMap::new();
                hpts.insert(PointType::Start, pt_start_id);
                hpts.insert(PointType::End, pt_end_id);

                // Edges
                let bs_id = BasicShapeId::new_id();
                let mut bss = HashMap::new();
                bss.insert(bs_id, (BasicShapeType::Segment, false, hpts));

                let mut hbss = HashSet::new();
                hbss.insert(bs_id);
                pts.insert(pt_start_id, (pt_start, hbss.clone()));
                pts.insert(pt_end_id, (pt_end, hbss));

                Shape {
                    selected: false,
                    bss,
                    pts,
                    constraints: HashMap::new(),
                }
            }
            Rectangle(bot_left, top_right) => {
                let top_left = WPos::new(bot_left.wx, top_right.wy);
                let bot_right = WPos::new(top_right.wx, bot_left.wy);

                // Vertices
                let mut pts = HashMap::new();
                let (pt_bot_left, pt_bot_left_id) = Point::new(&bot_left, true, true, false);
                let (pt_top_left, pt_top_left_id) = Point::new(&top_left, true, true, false);
                let (pt_top_right, pt_top_right_id) = Point::new(&top_right, true, true, false);
                let (pt_bot_right, pt_bot_right_id) = Point::new(&bot_right, true, true, false);
                let mut hpts_left_line = HashMap::new();
                hpts_left_line.insert(PointType::Start, pt_bot_left_id);
                hpts_left_line.insert(PointType::End, pt_top_left_id);
                let mut hpts_top_line = HashMap::new();
                hpts_top_line.insert(PointType::Start, pt_top_left_id);
                hpts_top_line.insert(PointType::End, pt_top_right_id);
                let mut hpts_right_line = HashMap::new();
                hpts_right_line.insert(PointType::Start, pt_top_right_id);
                hpts_right_line.insert(PointType::End, pt_bot_right_id);
                let mut hpts_bot_line = HashMap::new();
                hpts_bot_line.insert(PointType::Start, pt_bot_right_id);
                hpts_bot_line.insert(PointType::End, pt_bot_left_id);

                // Edges
                let mut bss = HashMap::new();

                let id_left_line = BasicShapeId::new_id();
                bss.insert(
                    id_left_line,
                    (BasicShapeType::Segment, false, hpts_left_line),
                );
                let id_top_line = BasicShapeId::new_id();
                bss.insert(id_top_line, (BasicShapeType::Segment, false, hpts_top_line));
                let id_right_line = BasicShapeId::new_id();
                bss.insert(
                    id_right_line,
                    (BasicShapeType::Segment, false, hpts_right_line),
                );
                let id_bot_line = BasicShapeId::new_id();
                bss.insert(id_bot_line, (BasicShapeType::Segment, false, hpts_bot_line));

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
                let mut constraints = HashMap::new();
                constraints.insert(
                    (pt_bot_left_id, pt_top_left_id),
                    PointConstraint::VerticalAlign,
                );
                constraints.insert(
                    (pt_top_left_id, pt_top_right_id),
                    PointConstraint::HorizontalAlign,
                );
                constraints.insert(
                    (pt_top_right_id, pt_bot_right_id),
                    PointConstraint::VerticalAlign,
                );
                constraints.insert(
                    (pt_bot_right_id, pt_bot_left_id),
                    PointConstraint::HorizontalAlign,
                );
                Shape {
                    selected: false,
                    bss,
                    pts,
                    constraints,
                }
            }
        }
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
            .for_each(|(bs, selected, _)| *selected = selection);
    }
    pub fn clear_all_pts_selection(&mut self) {
        self.pts
            .values_mut()
            .for_each(|(pt, _)| pt.selected = false)
    }
    pub fn is_a_point_selected(&self) -> Option<PointId> {
        for (bs_id, (bs, _, hpts)) in self.bss.iter() {
            use BasicShapeType::*;
            match bs {
                Segment => {
                    let seg_points = vec![PointType::Start, PointType::End];
                    for pt_typ in seg_points.iter() {
                        if let Some(pt_id) = hpts.get(pt_typ) {
                            if let Some((pt, _)) = self.pts.get(pt_id) {
                                if pt.selected {
                                    return Some(*pt_id);
                                }
                            }
                        }
                    }
                } // QBezier(qbezier) => qbezier.save_current_position(),
                  // CBezier(cbezier) => cbezier.save_current_position(),
                  // ArcEllipse(aellipse) => aellipse.save_current_position(),
            }
        }
        None
    }
    pub fn set_bs_selection(&mut self, bs_id: &BasicShapeId, selection: bool) {
        if let Some((bs, selected, _)) = self.bss.get_mut(bs_id) {
            *selected = selection;
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
    pub fn s_dist(&self, pos: &WPos) -> Option<(BasicShapeId, f64)> {
        // Return the signed minimum (pos/neg nearest to zero)
        // of all the distances between pos and basic shapes
        let mut min_s_dist = f64::MAX;
        let mut bs_id_min = None;
        for (bs_id, (bs, _, bspts)) in self.bss.iter() {
            let mut s_dist = f64::MAX;
            use BasicShapeType::*;
            match bs {
                Segment => {
                    if let Some(pt_start_id) = bspts.get(&PointType::Start) {
                        if let Some(pt_end_id) = bspts.get(&PointType::End) {
                            if let Some((pt_start, _)) = self.pts.get(pt_start_id) {
                                if let Some((pt_end, _)) = self.pts.get(pt_end_id) {
                                    s_dist = pos.s_dist_seg(&pt_start.wpos, &pt_end.wpos)
                                }
                            }
                        }
                    }
                } // QBezier(qbezier) => qbezier.save_current_position(),
                  // CBezier(cbezier) => cbezier.save_current_position(),
                  // ArcEllipse(aellipse) => aellipse.save_current_position(),
            };

            if min_s_dist.abs() > s_dist.abs() {
                min_s_dist = s_dist;
                bs_id_min = Some(*bs_id);
            }
        }
        if let Some(bs_id) = bs_id_min {
            Some((bs_id, min_s_dist))
        } else {
            None
        }
    }
    pub fn is_shape_under_pos(&self, pick_pos: &WPos, grab_handle_precision: f64) -> bool {
        if let Some((_, s_dist)) = self.s_dist(pick_pos) {
            if s_dist.abs() < grab_handle_precision {
                true
            } else {
                false
            }
        } else {
            false
        }
    }
    pub fn get_point_under_pos(
        &self,
        pick_pos: &WPos,
        grab_handle_precision: f64,
    ) -> Option<(BasicShapeId, PointId, PointType)> {
        for (bs_id, (bs, _, hpts)) in self.bss.iter() {
            use BasicShapeType::*;
            match bs {
                Segment => {
                    let seg_points = vec![PointType::Start, PointType::End];
                    for pt_typ in seg_points.iter() {
                        if let Some(pt_id) = hpts.get(pt_typ) {
                            if let Some((pt, _)) = self.pts.get(pt_id) {
                                if pt.wpos.dist(pick_pos) < grab_handle_precision {
                                    return Some((*bs_id, *pt_id, *pt_typ));
                                }
                            }
                        }
                    }
                } // QBezier(qbezier) => qbezier.save_current_position(),
                  // CBezier(cbezier) => cbezier.save_current_position(),
                  // ArcEllipse(aellipse) => aellipse.save_current_position(),
            }
        }
        None
    }
    pub fn select_point_under_pos(&mut self, pick_pos: &WPos, grab_handle_precision: f64) -> bool {
        if let Some((_, pt_id, _)) = self.get_point_under_pos(pick_pos, grab_handle_precision) {
            if let Some((pt, _)) = self.pts.get_mut(&pt_id) {
                pt.selected = true;
                return true;
            }
        }
        false
    }
    pub fn move_elements(&mut self, pick_delta_pos: &WPos) {
        // If a point is selected, we move the point
        // In no point is selected we move all the position points
        if let Some(pt_id) = self.is_a_point_selected() {
            self.move_point(&pt_id, pick_delta_pos);
        } else {
            self.move_all_points(pick_delta_pos);
        }
    }
    pub fn move_point(&mut self, pt_id: &PointId, pick_delta_pos: &WPos) {
        if let Some((pt, _)) = self.pts.get_mut(pt_id) {
            pt.wpos += *pick_delta_pos;
        }
    }
    pub fn move_all_points(&mut self, pick_delta_pos: &WPos) {
        let mut pt_to_move = HashSet::new();
        for (_, (_, _, hpts)) in self.bss.iter() {
            hpts.values()
                .for_each(|pt_id| _ = pt_to_move.insert(*pt_id));
        }
        for pt_id in pt_to_move.iter() {
            self.move_point(pt_id, pick_delta_pos);
        }
    }
    pub fn get_bss_constructions(&self, cst: &mut Vec<ConstructionType>) {
        for (_, (bs, selected, hpts)) in self.bss.iter() {
            use ConstructionType::*;
            match bs {
                BasicShapeType::Segment => {
                    match (hpts.get(&PointType::Start), hpts.get(&PointType::End)) {
                        (Some(start_id), Some(end_id)) => {
                            match (self.pts.get(start_id), self.pts.get(end_id)) {
                                (Some((start_pt, _)), Some((end_pt, _))) => {
                                    let pattern = match (self.selected, selected) {
                                        (false, false) => Pattern::NoSelection,
                                        (false, true) => Pattern::SimpleSelection,
                                        (true, false) => Pattern::SimpleSelection,
                                        (true, true) => Pattern::DoubleSelection,
                                    };
                                    cst.push(Segment(pattern, start_pt.wpos, end_pt.wpos))
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
                BasicShapeType::QBezier => {
                    //
                }
                BasicShapeType::CBezier => {
                    //
                }
                BasicShapeType::ArcEllipse => {
                    //
                }
            }
        }
    }
    pub fn get_handles_construction(&self, cst: &mut Vec<ConstructionType>, size_handle: f64) {
        for (_, (pt, _)) in self.pts.iter() {
            push_handle(cst, &pt, size_handle);
        }
    }
    pub fn get_helpers_construction(&self, cst: &mut Vec<ConstructionType>) {
        for (_, (bs, selected, hpts)) in self.bss.iter() {
            use ConstructionType::*;
            match bs {
                BasicShapeType::Segment => {
                    match (hpts.get(&PointType::Start), hpts.get(&PointType::End)) {
                        (Some(start_id), Some(end_id)) => {
                            match (self.pts.get(start_id), self.pts.get(end_id)) {
                                (Some((start_pt, _)), Some((end_pt, _))) => {
                                    if is_aligned_vert(&start_pt.wpos, &end_pt.wpos) {
                                        helper_vertical(
                                            &(start_pt.wpos),
                                            &(end_pt.wpos),
                                            true,
                                            &mut cst,
                                        );
                                    }
                                    if is_aligned_hori(&start_pt.wpos, &end_pt.wpos) {
                                        helper_horizontal(
                                            &(start_pt.wpos),
                                            &(end_pt.wpos),
                                            true,
                                            &mut cst,
                                        );
                                    }
                                    if is_aligned_45_or_135(start_pt.wpos, end_point.wpos) {
                                        helper_45_135(
                                            &(start_pt.wpos),
                                            &(end_pt.wpos),
                                            true,
                                            &mut cst,
                                        );
                                    }
                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    }
                }
                BasicShapeType::QBezier => {
                    //
                }
                BasicShapeType::CBezier => {
                    //
                }
                BasicShapeType::ArcEllipse => {
                    //
                }
            }
        }
    }
}
