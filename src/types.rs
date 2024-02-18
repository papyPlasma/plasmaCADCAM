// sA macro to provide `println!(..)`-style syntax for `console.log` logging.
// macro_rules! log {
//     ( $( $t:tt )* ) => {
//         web_sys::console::log_1(&format!( $( $t )* ).into());
//     }
// }

use js_sys::Array;
use kurbo::{BezPath, Point};

use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::prelude::*;
use web_sys::CssStyleDeclaration;

pub struct DrawStyles {
    // Drawing colors
    worksheet_color: String,
    dimension_color: String,
    geohelper_color: String,
    origin_color: String,
    grid_color: String,
    selection_color: String,
    selected_color: String,
    background_color: String,
    fill_color: String,
    highlight_color: String,
    // line patterns
    pattern_dashed: JsValue,
    pattern_solid: JsValue,
}
impl DrawStyles {
    pub fn build(style: CssStyleDeclaration) -> Result<DrawStyles, JsValue> {
        let worksheet_color = style.get_property_value("--canvas-worksheet-color")?;
        let dimension_color = style.get_property_value("--canvas-dimension-color")?;
        let geohelper_color = style.get_property_value("--canvas-geohelper-color")?;
        let origin_color = style.get_property_value("--canvas-origin-color")?;
        let grid_color = style.get_property_value("--canvas-grid-color")?;
        let selection_color = style.get_property_value("--canvas-selection-color")?;
        let selected_color = style.get_property_value("--canvas-selected-color")?;
        let background_color = style.get_property_value("--canvas-background-color")?;
        let fill_color = style.get_property_value("--canvas-fill-color")?;
        let highlight_color = style.get_property_value("--canvas-highlight-color")?;
        let dash_pattern = Array::new();
        dash_pattern.push(&JsValue::from_f64(3.0));
        dash_pattern.push(&JsValue::from_f64(3.0));
        let solid_pattern = Array::new();
        Ok(DrawStyles {
            worksheet_color,
            dimension_color,
            geohelper_color,
            origin_color,
            grid_color,
            selection_color,
            selected_color,
            background_color,
            fill_color,
            highlight_color,
            pattern_dashed: JsValue::from(dash_pattern),
            pattern_solid: JsValue::from(solid_pattern),
        })
    }
    pub fn get_default_styles(&self, cbp: &ConstructionBezierPath) -> (&str, &str, &JsValue, f64) {
        use ConstructionLayer::*;
        let (fill_color, color, line_dash, line_width) = match cbp.layer {
            Worksheet => {
                if let ConstructionPattern::Selected = cbp.pattern {
                    (
                        &self.selected_color,
                        &self.selected_color,
                        &self.pattern_solid,
                        1.,
                    )
                } else {
                    if let ConstructionPattern::Highlighted = cbp.pattern {
                        (
                            &self.highlight_color,
                            &self.highlight_color,
                            &self.pattern_solid,
                            1.,
                        )
                    } else {
                        if let ConstructionPattern::OnConstruction = cbp.pattern {
                            (
                                &self.geohelper_color,
                                &self.geohelper_color,
                                &self.pattern_solid,
                                1.,
                            )
                        } else {
                            (
                                &self.fill_color,
                                &self.worksheet_color,
                                &self.pattern_solid,
                                1.,
                            )
                        }
                    }
                }
            }
            Dimension => (
                &self.fill_color,
                &self.dimension_color,
                &self.pattern_solid,
                1.,
            ),
            GeometryHelpers => (
                &self.fill_color,
                &self.geohelper_color,
                &self.pattern_dashed,
                1.,
            ),
            Origin => (
                &self.fill_color,
                &self.origin_color,
                &self.pattern_solid,
                1.,
            ),
            Grid => (&self.fill_color, &self.grid_color, &self.pattern_solid, 1.),
        };
        (fill_color, color, line_dash, line_width)
    }
    pub fn get_background_color(&self) -> &str {
        &self.background_color
    }
    pub fn get_selected_color(&self) -> &str {
        &self.selected_color
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ConstructionLayer {
    Worksheet,
    Dimension,
    GeometryHelpers,
    Origin,
    Grid,
}
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum ConstructionPattern {
    Normal,
    OnConstruction,
    Selected,
    Highlighted,
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConstructionBezierPath {
    pub layer: ConstructionLayer,
    pub pattern: ConstructionPattern,
    pub path: BezPath,
    pub filled: bool,
}

#[derive(Clone, Debug)]
pub enum Geobjects {
    Vertex(VertexId),
    Shape(ShapeId),
}
impl Geobjects {}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub id: VertexId,
    pub pt: Point,
    pub saved_pt: Point,
    pub magnetic: bool,
    pub draggable: bool,
    pub selected: bool,
}
impl Vertex {
    pub fn new(pt: impl Into<Point> + Clone) -> Vertex {
        let id = VertexId::new_id();
        Vertex {
            id,
            pt: pt.clone().into(),
            saved_pt: pt.into(),
            magnetic: true,
            draggable: true,
            selected: false,
        }
    }
    pub fn magnetic(mut self, magnetic: bool) -> Self {
        self.magnetic = magnetic;
        self
    }
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }
    pub fn set_selection(mut self, selection: bool) -> Self {
        self.selected = selection;
        self
    }
    pub fn dist_sq(&self, v: &Vertex) -> f64 {
        (v.pt.y - self.pt.y).powi(2) + (v.pt.x - self.pt.x).powi(2)
    }
    pub fn is_near_pos(&self, pt: &Point, grab_handle_precision: f64) -> bool {
        self.pt.distance(*pt) < grab_handle_precision / 2.
    }
}

static COUNTER_GROUPS: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct GroupId(usize);
impl GroupId {
    pub fn new_id() -> GroupId {
        GroupId(COUNTER_GROUPS.fetch_add(1, Ordering::Relaxed))
    }
}
impl Deref for GroupId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for GroupId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

static COUNTER_BINDINGS: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct BindingId(usize);
impl BindingId {
    pub fn new_id() -> BindingId {
        BindingId(COUNTER_BINDINGS.fetch_add(1, Ordering::Relaxed))
    }
}
impl Deref for BindingId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for BindingId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

static COUNTER_SHAPES: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct ShapeId(usize);
impl ShapeId {
    pub fn new_id() -> ShapeId {
        ShapeId(COUNTER_SHAPES.fetch_add(1, Ordering::Relaxed))
    }
}
impl Deref for ShapeId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ShapeId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

static COUNTER_VERTICES: AtomicUsize = AtomicUsize::new(0);
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct VertexId(usize);
impl VertexId {
    pub fn new_id() -> VertexId {
        VertexId(COUNTER_VERTICES.fetch_add(1, Ordering::Relaxed))
    }
}
impl Deref for VertexId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for VertexId {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
