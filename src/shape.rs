use std::collections::{HashMap, HashSet};

use crate::basic_shapes::basic_shapes::*;
use crate::types::*;

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
    id: ShapeId,
    selected: bool,
    bss: HashMap<BasicShapeId, (BasicShapeType, HashMap<PointType, PointId>)>,
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
                bss.insert(bs_id, (BasicShapeType::Segment(false), hpts));

                let mut hbss = HashSet::new();
                hbss.insert(bs_id);
                pts.insert(pt_start_id, (pt_start, hbss.clone()));
                pts.insert(pt_end_id, (pt_end, hbss));

                Shape {
                    id: ShapeId::new_id(),
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
                    (BasicShapeType::Segment(false), hpts_left_line),
                );
                let id_top_line = BasicShapeId::new_id();
                bss.insert(id_top_line, (BasicShapeType::Segment(false), hpts_top_line));
                let id_right_line = BasicShapeId::new_id();
                bss.insert(
                    id_right_line,
                    (BasicShapeType::Segment(false), hpts_right_line),
                );
                let id_bot_line = BasicShapeId::new_id();
                bss.insert(id_bot_line, (BasicShapeType::Segment(false), hpts_bot_line));

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
                    id: ShapeId::new_id(),
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
        use BasicShapeType::*;
        self.bss.values_mut().for_each(|(bs, _)| match bs {
            Segment(selected) => *selected = selection,
            // QBezier(qbezier) => qbezier.save_current_position(),
            // CBezier(cbezier) => cbezier.save_current_position(),
            // ArcEllipse(aellipse) => aellipse.save_current_position(),
        });
    }
    pub fn clear_all_pts_selection(&mut self) {
        self.pts
            .values_mut()
            .for_each(|(pt, _)| pt.selected = false)
    }
    pub fn is_a_point_selected(&self) -> Option<PointId> {
        for (bs_id, (bs, hpts)) in self.bss.iter() {
            use BasicShapeType::*;
            match bs {
                Segment(_) => {
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
        if let Some((bs, _)) = self.bss.get_mut(bs_id) {
            use BasicShapeType::*;
            match bs {
                Segment(selected) => *selected = selection,
                // QBezier(qbezier) => qbezier.save_current_position(),
                // CBezier(cbezier) => cbezier.save_current_position(),
                // ArcEllipse(aellipse) => aellipse.save_current_position(),
            }
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
        for (bs_id, (bs, bspts)) in self.bss.iter() {
            let mut s_dist = f64::MAX;
            use BasicShapeType::*;
            match bs {
                Segment(_) => {
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
        for (bs_id, (bs, hpts)) in self.bss.iter() {
            use BasicShapeType::*;
            match bs {
                Segment(_) => {
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
        for (_, (_, hpts)) in self.bss.iter() {
            hpts.values()
                .for_each(|pt_id| _ = pt_to_move.insert(*pt_id));
        }
        for pt_id in pt_to_move.iter() {
            self.move_point(pt_id, pick_delta_pos);
        }
    }
    pub fn get_construction(&self) -> Vec<ConstructionType> {
        let mut cst: Vec<ConstructionType> = vec![];
        if !self.selected {
            cst.push(ConstructionType::Layer(LayerType::Worksheet));
        } else {
            cst.push(ConstructionType::Layer(LayerType::Selected));
        }
        cst.push(ConstructionType::Move(
            self.position + self.start_point.wpos,
        ));
        cst.push(ConstructionType::Line(self.position + self.end_point.wpos));
        cst
    }
}
