use std::collections::{HashMap, HashSet};

use kurbo::Point;

use crate::pools::{BindingsPool, VerticesPool};
use crate::types::{VIds, VertexId};

use gomez::nalgebra::{Dyn, IsContiguous};
use gomez::{Domain, Problem, SolverDriver, System};

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum Binding {
    Fixed(BindFixed),
    FixedX(BindFixedX),
    FixedY(BindFixedY),
    Vertical(BindVertical),
    Horizontal(BindHorizontal),
    Parallel(BindParallel),
    SamePos(BindSamePos),
    Distance(BindDistance),
}
#[allow(dead_code)]
impl Binding {
    pub fn get_id(&self) -> VIds {
        match self {
            Binding::Fixed(b) => b.id,
            Binding::FixedX(b) => b.id,
            Binding::FixedY(b) => b.id,
            Binding::Vertical(b) => b.id,
            Binding::Horizontal(b) => b.id,
            Binding::Parallel(b) => b.id,
            Binding::SamePos(b) => b.id,
            Binding::Distance(b) => b.id,
        }
    }
    pub fn get_v_ids(&self, v_ids: &mut HashSet<VertexId>) {
        match self {
            Binding::Fixed(b) => {
                v_ids.insert(b.id.0);
            }
            Binding::FixedX(b) => {
                v_ids.insert(b.id.0);
            }
            Binding::FixedY(b) => {
                v_ids.insert(b.id.0);
            }
            Binding::Vertical(b) => {
                v_ids.insert(b.id.0);
                v_ids.insert(b.id.1);
            }
            Binding::Horizontal(b) => {
                v_ids.insert(b.id.0);
                v_ids.insert(b.id.1);
            }
            Binding::Parallel(b) => {
                v_ids.insert(b.id.0);
                v_ids.insert(b.id.1);
                v_ids.insert(b.id.2);
                v_ids.insert(b.id.3);
            }
            Binding::SamePos(b) => {
                v_ids.insert(b.id.0);
                v_ids.insert(b.id.1);
            }
            Binding::Distance(b) => {
                v_ids.insert(b.id.0);
                v_ids.insert(b.id.1);
            }
        };
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindFixed {
    pub id: VIds,
    pub fixed_value: Point,
}
impl BindFixed {
    pub fn bind(&self, vals: &[f64; 2]) -> [f64; 2] {
        [vals[0] - self.fixed_value.x, vals[1] - self.fixed_value.y]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindFixedX {
    pub id: VIds,
    pub fixed_value: f64,
}
impl BindFixedX {
    pub fn bind(&self, vals: &[f64; 2]) -> f64 {
        vals[0] - self.fixed_value
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindFixedY {
    pub id: VIds,
    pub fixed_value: f64,
}
impl BindFixedY {
    pub fn bind(&self, vals: &[f64; 2]) -> f64 {
        vals[1] - self.fixed_value
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindVertical {
    pub id: VIds,
}
impl BindVertical {
    pub fn bind(&self, vals: &[f64; 4]) -> f64 {
        vals[0] - vals[2]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindHorizontal {
    pub id: VIds,
}
impl BindHorizontal {
    pub fn bind(&self, vals: &[f64; 4]) -> f64 {
        vals[1] - vals[3]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindParallel {
    pub id: VIds,
}
impl BindParallel {
    pub fn bind(&self, vals: &[f64; 8]) -> f64 {
        (vals[6] - vals[4]) * (vals[3] - vals[1]) - (vals[7] - vals[5]) * (vals[2] - vals[0])
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindSamePos {
    pub id: VIds,
}
impl BindSamePos {
    pub fn bind(&self, vals: &[f64; 4]) -> [f64; 2] {
        [vals[0] - vals[2], vals[1] - vals[3]]
    }
}

#[derive(Copy, Clone, Debug)]
pub struct BindDistance {
    pub id: VIds,
    pub sq_distance_value: f64,
}
impl BindDistance {
    pub fn bind(&self, vals: &[f64; 4]) -> f64 {
        ((vals[3] - vals[1]).powi(2) + ((vals[2] - vals[0]).powi(2)) - self.sq_distance_value).abs()
    }
}

pub struct Eq2DConstraints<'a> {
    lut: Vec<(VertexId, f64)>,
    inv_lut: HashMap<VertexId, usize>,
    bindings_pool: &'a BindingsPool,
}

impl<'a> Eq2DConstraints<'a> {
    pub fn new(bindings_pool: &'a mut BindingsPool, v_pool: &VerticesPool) -> Eq2DConstraints<'a> {
        // Store the two values of each vertex from the bindings_pool
        // linearly on a vec for the solving, along with the vertex id for the bindings
        let mut lut = vec![];
        let mut inv_lut = HashMap::new();
        {
            // Get all vertices ids that are binded, NO DUPLICATE
            let mut v_ids = HashSet::new();
            bindings_pool
                .values()
                .for_each(|bind| bind.get_v_ids(&mut v_ids));

            v_ids.iter().for_each(|v_id| {
                lut.push((*v_id, v_pool[v_id].pt.x));
                lut.push((*v_id, v_pool[v_id].pt.y));
            });

            println!("lut: {:?}", lut);
        }

        lut.iter()
            .enumerate()
            .step_by(2)
            .for_each(|(idx, (v_id, _))| _ = inv_lut.insert(*v_id, idx));

        println!("inv_lut: {:?}", inv_lut);

        Eq2DConstraints {
            lut,
            inv_lut,
            bindings_pool,
        }
    }

    pub fn solve(&mut self, v_pool: &mut VerticesPool) -> Result<(), String> {
        let mut init = vec![];
        for (_, value) in self.lut.iter() {
            init.push(*value);
        }
        println!("init: {:?}", init);
        let mut solver = SolverDriver::builder(self).with_initial(init).build();
        let tolerance = 1e-6;
        let (vals, norm) = solver
            .find(|state| {
                println!(
                    "iter = {}\t||r(x)|| = {}\tx = {:?}",
                    state.iter(),
                    state.norm(),
                    state.x()
                );
                // println!("iter = {}", state.iter(),);
                state.norm() <= tolerance || state.iter() >= 100
            })
            .map_err(|error| format!("{error}"))?;

        println!("vals: {:?} ", vals);

        self.inv_lut.iter().for_each(|(v_id, idx)| {
            let v = v_pool.get_mut(v_id).unwrap();
            v.pt.x = vals[*idx];
            v.pt.y = vals[*idx + 1];
        });

        if norm <= tolerance {
            Ok(())
        } else {
            Err("did not converge".to_string())
        }
    }
}

impl<'a> Problem for Eq2DConstraints<'a> {
    type Field = f64;
    fn domain(&self) -> Domain<Self::Field> {
        Domain::unconstrained(self.lut.len())
    }
}

impl<'a> System for Eq2DConstraints<'a> {
    fn eval<Sx, Srx>(
        &self,
        x: &gomez::nalgebra::Vector<Self::Field, Dyn, Sx>,
        rx: &mut gomez::nalgebra::Vector<Self::Field, Dyn, Srx>,
    ) where
        Sx: gomez::nalgebra::storage::Storage<Self::Field, Dyn> + IsContiguous,
        Srx: gomez::nalgebra::storage::StorageMut<Self::Field, Dyn>,
    {
        let mut idx_rx = 0;
        self.bindings_pool.iter().for_each(|(_, bind)| match bind {
            Binding::Fixed(b) => {
                let bind = b.bind(&[x[self.inv_lut[&b.id.0]], x[self.inv_lut[&b.id.0] + 1]]);
                rx[idx_rx] = bind[0];
                idx_rx += 1;
                rx[idx_rx] = bind[1];
                idx_rx += 1;
            }
            Binding::FixedX(b) => {
                rx[idx_rx] = b.bind(&[x[self.inv_lut[&b.id.0]], x[self.inv_lut[&b.id.0] + 1]]);
                idx_rx += 1;
            }
            Binding::FixedY(b) => {
                rx[idx_rx] = b.bind(&[x[self.inv_lut[&b.id.0]], x[self.inv_lut[&b.id.0] + 1]]);
                idx_rx += 1;
            }
            Binding::Vertical(b) => {
                rx[idx_rx] = b.bind(&[
                    x[self.inv_lut[&b.id.0]],
                    x[self.inv_lut[&b.id.0] + 1],
                    x[self.inv_lut[&b.id.1]],
                    x[self.inv_lut[&b.id.1] + 1],
                ]);
                idx_rx += 1;
            }
            Binding::Horizontal(b) => {
                rx[idx_rx] = b.bind(&[
                    x[self.inv_lut[&b.id.0]],
                    x[self.inv_lut[&b.id.0] + 1],
                    x[self.inv_lut[&b.id.1]],
                    x[self.inv_lut[&b.id.1] + 1],
                ]);
                idx_rx += 1;
            }
            Binding::Parallel(b) => {
                rx[idx_rx] = b.bind(&[
                    x[self.inv_lut[&b.id.0]],
                    x[self.inv_lut[&b.id.0] + 1],
                    x[self.inv_lut[&b.id.1]],
                    x[self.inv_lut[&b.id.1] + 1],
                    x[self.inv_lut[&b.id.2]],
                    x[self.inv_lut[&b.id.2] + 1],
                    x[self.inv_lut[&b.id.3]],
                    x[self.inv_lut[&b.id.3] + 1],
                ]);
                idx_rx += 1;
            }
            Binding::SamePos(b) => {
                let bind = b.bind(&[
                    x[self.inv_lut[&b.id.0]],
                    x[self.inv_lut[&b.id.0] + 1],
                    x[self.inv_lut[&b.id.1]],
                    x[self.inv_lut[&b.id.1] + 1],
                ]);
                rx[idx_rx] = bind[0];
                idx_rx += 1;
                rx[idx_rx] = bind[1];
                idx_rx += 1;
            }
            Binding::Distance(b) => {
                rx[idx_rx] = b.bind(&[
                    x[self.inv_lut[&b.id.0]],
                    x[self.inv_lut[&b.id.0] + 1],
                    x[self.inv_lut[&b.id.1]],
                    x[self.inv_lut[&b.id.1] + 1],
                ]);
                idx_rx += 1;
            }
        });
    }
}
