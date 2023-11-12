use crate::math::*;
use crate::shapes::types::{GroupId, Shape, ShapeId, WPos};
use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use web_sys::console;

static COUNTER_SHAPES: AtomicUsize = AtomicUsize::new(0);

pub trait ShapePool: Shape + Any {}

pub struct DataPools {
    shapes_pool: ShapesPool,
    groups_pool: GroupsPool,
    shapes_selected: HashSet<ShapeId>,
}
impl DataPools {
    pub fn new() -> DataPools {
        DataPools {
            groups_pool: GroupsPool::new(),
            shapes_pool: ShapesPool::new(),
            shapes_selected: HashSet::new(),
        }
    }

    pub fn clear_shapes_selection(&mut self) {
        for (_, shape) in self.shapes_pool.iter_mut() {
            shape.deselect_all_points();
            shape.set_selected(false);
        }
        self.shapes_selected.clear();
    }
    pub fn set_shape_selected(&mut self, sh_id: &ShapeId, selected: bool) {
        let shape = self.shapes_pool.get_mut(sh_id).unwrap();
        shape.set_selected(selected);
        if selected {
            self.shapes_selected.insert(*sh_id);
        }
    }
    pub fn get_shapes_selected(&self) -> &HashSet<ShapeId> {
        &self.shapes_selected
    }
    pub fn shapes_selection(
        &mut self,
        pick_pos: &WPos,
        shift_pressed: bool,
        grab_handle_precision: f64,
    ) {
        let mut new_shapes_selection: HashSet<ShapeId> = HashSet::new();

        let mut point_under_pick_pos = None;
        let mut shape_under_pick_pos = None;

        // Retreive the firt point that is under the pick pos
        // If a point is found, retrive the associated shape
        for (sh_id, shape) in self.shapes_pool.iter_mut() {
            if let Some(point_type) =
                shape.get_shape_point_under_pick_pos(pick_pos, grab_handle_precision)
            {
                if shape.is_selected() {
                    point_under_pick_pos = Some((*sh_id, point_type));
                    shape_under_pick_pos = Some(*sh_id);
                    break;
                }
            }
        }
        // If no point is under the pick pos,
        // retreive THE FIRST shape that is under the pick pos
        if let None = point_under_pick_pos {
            for (sh_id, shape) in self.shapes_pool.iter_mut() {
                if shape.is_shape_under_pick_pos(pick_pos, grab_handle_precision) {
                    shape_under_pick_pos = Some(*sh_id);
                    break;
                }
            }
        }

        // If neither shape nor point are under the pick pos then deselect all
        // shapes and points and return
        if !shift_pressed {
            if let None = shape_under_pick_pos {
                if let None = point_under_pick_pos {
                    for (_, shape) in self.shapes_pool.iter_mut() {
                        shape.set_selected(false);
                        shape.deselect_all_points();
                    }
                    self.shapes_selected.clear();
                    return;
                }
            }
        }

        // Calculate the number of shapes that will be selected
        let nb_shape_will_select = if shift_pressed {
            if let None = shape_under_pick_pos {
                self.shapes_selected.len()
            } else {
                self.shapes_selected.len() + 1
            }
        } else {
            if let None = shape_under_pick_pos {
                0
            } else {
                1
            }
        };
        if nb_shape_will_select > 1 {
            // If more then one shape will be selected at the end then remove all points selection
            self.shapes_pool
                .iter_mut()
                .for_each(|(_, shape)| shape.deselect_all_points());
            point_under_pick_pos = None;
        }

        // Clear all shapes and points selection
        for (_, shape) in self.shapes_pool.iter_mut() {
            shape.set_selected(false);
            shape.deselect_all_points();
        }

        // fill new_shapes_selection
        if let Some((sh_id, point_type)) = point_under_pick_pos {
            let shape = self.shapes_pool.get_mut(&sh_id).unwrap();
            new_shapes_selection.insert(sh_id);
            shape.save_current_position();
            shape.set_selected(true);
            shape.select_point(&point_type);
        } else {
            if let Some(sh_id) = shape_under_pick_pos {
                let shape = self.shapes_pool.get_mut(&sh_id).unwrap();
                new_shapes_selection.insert(sh_id);
                shape.save_current_position();
                shape.set_selected(true);
            }
        }

        // Save new selection
        if shift_pressed {
            for sh_id in self.shapes_selected.iter() {
                let shape = self.shapes_pool.get_mut(&sh_id).unwrap();
                shape.set_selected(true);
                new_shapes_selection.insert(*sh_id);
            }
        }
        self.shapes_selected = new_shapes_selection;
    }

    pub fn select_shapes_bounded_by_rectangle(&mut self, bb_outer: [WPos; 2]) {
        for (sh_id, shape) in self.shapes_pool.iter_mut() {
            let bb_inner = shape.get_bounded_rectangle();
            if is_box_inside(&bb_outer, &bb_inner) {
                shape.set_selected(true);
                self.shapes_selected.insert(*sh_id);
            }
        }
    }
    pub fn insert_shape<T: ShapePool>(&mut self, shape: T) -> ShapeId {
        let sh_id = ShapeId(COUNTER_SHAPES.fetch_add(1, Ordering::Relaxed));
        self.shapes_pool.insert(sh_id, Box::new(shape));
        sh_id
    }

    pub fn create_group_id(&mut self) -> GroupId {
        self.groups_pool.create_id()
    }
    pub fn set_shape_group(&mut self, gr_id: &GroupId, sh_id: &ShapeId) {
        self.groups_pool.insert_shape_id(gr_id, sh_id);
    }
    pub fn get_shape_group(&mut self, gr_id: &GroupId) -> Option<&Vec<ShapeId>> {
        self.groups_pool.get(gr_id)
    }

    pub fn get_shape(&self, sh_id: &ShapeId) -> Option<&Box<dyn Shape>> {
        self.shapes_pool.get(sh_id)
    }
    pub fn get_shape_mut(&mut self, sh_id: &ShapeId) -> Option<&mut Box<dyn Shape>> {
        self.shapes_pool.get_mut(sh_id)
    }

    pub fn get_all_shapes(&self) -> &ShapesPool {
        &self.shapes_pool
    }
    pub fn get_all_shapes_mut(&mut self) -> &mut ShapesPool {
        &mut self.shapes_pool
    }

    pub fn get_shape_position(&self, sh_id: &ShapeId) -> WPos {
        self.shapes_pool.get(sh_id).unwrap().get_pos()
    }
    pub fn magnet_to_point(
        &self,
        pick_pos: &mut WPos,
        excluded_sh_id: Option<ShapeId>,
        magnet_distance: f64,
    ) {
        // Test all all shapes points but not the one that is excluded
        for (sh_id, shape) in self.shapes_pool.iter() {
            let exclude = if let Some(exc_sh_id) = excluded_sh_id {
                if *sh_id == exc_sh_id {
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if !exclude {
                shape.magnet_to_point(pick_pos, magnet_distance);
            }
        }
    }
    pub fn is_point_on_shape(&self, sh_id: &ShapeId, pt_pos: &WPos, precision: f64) -> bool {
        let shape = self.shapes_pool.get(sh_id).unwrap();
        shape.is_shape_under_pick_pos(&pt_pos, precision)
    }
}

pub struct ShapesPool(pub HashMap<ShapeId, Box<dyn Shape>>);
impl std::ops::Deref for ShapesPool {
    type Target = HashMap<ShapeId, Box<dyn Shape>>;
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
        ShapesPool(HashMap::new())
    }
}

pub struct GroupsPool(HashMap<GroupId, Vec<ShapeId>>);
impl std::ops::Deref for GroupsPool {
    type Target = HashMap<GroupId, Vec<ShapeId>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::ops::DerefMut for GroupsPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl GroupsPool {
    pub fn new() -> GroupsPool {
        GroupsPool(HashMap::new())
    }
    pub fn create_id(&mut self) -> GroupId {
        let group_id = GroupId::new_id();
        group_id
    }
    pub fn insert_shape_id(&mut self, group_id: &GroupId, sh_id: &ShapeId) {
        if let Some(sh_ids) = self.get_mut(group_id) {
            sh_ids.push(*sh_id);
        } else {
            self.0.insert(*group_id, vec![*sh_id]);
        }
    }
    pub fn get_shapes_ids(&self, group_id: &GroupId) -> Option<Vec<ShapeId>> {
        self.0.get(group_id).cloned()
    }
}
