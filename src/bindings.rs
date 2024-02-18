use std::collections::HashSet;

use kurbo::Point;

use crate::types::{BindingId, VertexId};

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum Binding {
    Fixed(BindFixed),
    FixedX(BindFixedX),
    FixedY(BindFixedY),
    Vertical(BindVertical),
    Horizontal(BindHorizontal),
    Parallel(BindParallel),
    Distance(BindDistance),
    Error(BindError),
}
#[allow(dead_code)]
impl Binding {
    pub fn get_id(&self) -> BindingId {
        match self {
            Binding::Fixed(b) => b.id,
            Binding::FixedX(b) => b.id,
            Binding::FixedY(b) => b.id,
            Binding::Vertical(b) => b.id,
            Binding::Horizontal(b) => b.id,
            Binding::Parallel(b) => b.id,
            Binding::Distance(b) => b.id,
            Binding::Error(b) => b.id,
        }
    }
    pub fn get_v_ids(&self, v_ids: &mut HashSet<VertexId>) {
        match self {
            Binding::Fixed(b) => {
                v_ids.insert(b.v_id);
            }
            Binding::FixedX(b) => {
                v_ids.insert(b.v_id);
            }
            Binding::FixedY(b) => {
                v_ids.insert(b.v_id);
            }
            Binding::Vertical(b) => {
                v_ids.insert(b.va_id);
                v_ids.insert(b.vb_id);
            }
            Binding::Horizontal(b) => {
                v_ids.insert(b.va_id);
                v_ids.insert(b.vb_id);
            }
            Binding::Parallel(b) => {
                v_ids.insert(b.l1va_id);
                v_ids.insert(b.l1vb_id);
                v_ids.insert(b.l2va_id);
                v_ids.insert(b.l2vb_id);
            }
            Binding::Distance(b) => {
                v_ids.insert(b.va_id);
                v_ids.insert(b.vb_id);
            }
            Binding::Error(_) => (),
        };
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct BindError {
    pub id: BindingId,
}
#[allow(dead_code)]
impl BindError {
    pub fn new() -> Binding {
        return Binding::Error(BindError {
            id: BindingId::new_id(),
        });
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindFixed {
    pub id: BindingId,
    pub fixed_value: Point,
    pub v_id: VertexId,
}
impl BindFixed {
    pub fn bind(&self, vals: &[f64; 2]) -> [f64; 2] {
        [vals[0] - self.fixed_value.x, vals[1] - self.fixed_value.y]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindFixedX {
    pub id: BindingId,
    pub fixed_value: f64,
    pub v_id: VertexId,
}
impl BindFixedX {
    pub fn bind(&self, vals: &[f64; 2]) -> f64 {
        vals[0] - self.fixed_value
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindFixedY {
    pub id: BindingId,
    pub fixed_value: f64,
    pub v_id: VertexId,
}
impl BindFixedY {
    pub fn bind(&self, vals: &[f64; 2]) -> f64 {
        vals[1] - self.fixed_value
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindVertical {
    pub id: BindingId,
    pub va_id: VertexId,
    pub vb_id: VertexId,
}
impl BindVertical {
    pub fn bind(&self, vals: &[f64; 4]) -> f64 {
        vals[0] - vals[2]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindHorizontal {
    pub id: BindingId,
    pub va_id: VertexId,
    pub vb_id: VertexId,
}
impl BindHorizontal {
    pub fn bind(&self, vals: &[f64; 4]) -> f64 {
        vals[1] - vals[3]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindParallel {
    pub id: BindingId,
    pub l1va_id: VertexId,
    pub l1vb_id: VertexId,
    pub l2va_id: VertexId,
    pub l2vb_id: VertexId,
}
impl BindParallel {
    pub fn bind(&self, vals: &[f64; 8]) -> f64 {
        (vals[6] - vals[4]) * (vals[3] - vals[1]) - (vals[7] - vals[5]) * (vals[2] - vals[0])
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindDistance {
    pub id: BindingId,
    pub sq_distance_value: f64,
    pub va_id: VertexId,
    pub vb_id: VertexId,
}
impl BindDistance {
    pub fn bind(&self, vals: &[f64; 4]) -> f64 {
        ((vals[3] - vals[1]).powi(2) + ((vals[2] - vals[0]).powi(2)) - self.sq_distance_value).abs()
    }
}
