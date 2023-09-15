use crate::math::*;
use crate::shapes::{ConstructionType, Shape, ShapeType};
use js_sys::Array;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    console, CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement, HtmlElement,
    KeyboardEvent, MouseEvent, Path2d, WheelEvent, Window,
};

//console::log_1(&format!("{:?}", xxx).into());
//console::log_1(&"ddd".into());

pub type ElementCallback = Box<dyn Fn(Rc<RefCell<PlayingArea>>, Event) + 'static>;

#[derive(Debug, Copy, Clone)]
pub enum ToolSelected {
    Arrow,
    Selection,
    DrawLine,
    DrawQuadBezier,
    DrawCubicBezier,
    DrawCircle,
    DrawSquare,
}

#[derive(Debug, Copy, Clone)]
pub enum LayerType {
    WorkPiece,
    Dimension,
    GeometryHelpers,
    Origin,
    Grid,
    Selection,
    Selected,
    Handle,
}

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
enum MouseState {
    NoButton = 0,
    LeftDown = 1,
    #[allow(dead_code)]
    RightDown = 2,
    #[allow(dead_code)]
    MiddleDown = 4,
}

pub struct PlayingArea {
    pub window: Window,
    pub document: Document,
    pub body: HtmlElement,
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,

    // DOM
    #[allow(dead_code)]
    contex_menu: HtmlElement,
    user_icons: HashMap<String, Element>,
    //
    shapes: Vec<Shape>,
    // shape_buffer_copy_paste: Vec<Shape>,
    current_shape: Option<Shape>,
    tool_selected: ToolSelected,
    selection_area: Option<[WXY; 2]>,
    ctrl_or_meta_pressed: bool,
    //
    mouse_state: MouseState,
    mouse_previous_pos_canvas: CXY,
    mouse_previous_pos_word: WXY,

    working_area: WXY,
    scale: f64,
    canvas_offset: CXY,
    working_area_grid_step: f64,
    editing_snap_step: f64,

    // Drawing colors
    workpiece_color: String,
    dimension_color: String,
    geohelper_color: String,
    origin_color: String,
    grid_color: String,
    selection_color: String,
    selected_color: String,
    background_color: String,

    // line patterns
    pub pattern_dashed: JsValue,
    pub pattern_solid: JsValue,

    head_position: WXY,
    visual_handle_size: f64,
    //
    precision: f64,
}

// Canvas events: mouse, keyboard and context menu
fn on_mouse_down(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        if mouse_event.buttons() == MouseState::LeftDown as u16 {
            let mut pa_ref = pa.borrow_mut();
            pa_ref.mouse_state = MouseState::LeftDown;

            // Get mouse position relative to the canvas
            let rect = pa_ref.canvas.get_bounding_client_rect();
            let mouse_pos_canvas = CXY {
                cx: mouse_event.client_x() as f64 - rect.left(),
                cy: mouse_event.client_y() as f64 - rect.top(),
            };

            let scale = pa_ref.scale;
            let offset = pa_ref.canvas_offset;

            let mouse_pos_world = mouse_pos_canvas.to_world(scale, offset);

            let snap_val = pa_ref.editing_snap_step / 2.;
            let visual_handle_size = pa_ref.visual_handle_size;

            use ToolSelected::*;
            match pa_ref.tool_selected {
                Arrow => {
                    let precision = pa_ref.precision;
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.set_selection(&mouse_pos_world, precision);
                    }
                }
                Selection => pa_ref.selection_area = Some([mouse_pos_world, mouse_pos_world]),
                DrawLine => {
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.remove_selection();
                    }
                    let mut start = mouse_pos_world;
                    snap_to_grid(&mut start, pa_ref.working_area_grid_step);
                    pa_ref.current_shape = Some(Shape::new(
                        ShapeType::Line(vec![start, start]),
                        snap_val,
                        visual_handle_size,
                    ));
                }
                DrawQuadBezier => {
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.remove_selection();
                    }
                    let mut start = mouse_pos_world;
                    snap_to_grid(&mut start, pa_ref.working_area_grid_step);
                    pa_ref.current_shape = Some(Shape::new(
                        ShapeType::QuadBezier(vec![start, start, start]),
                        snap_val,
                        visual_handle_size,
                    ));
                }
                DrawCubicBezier => {
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.remove_selection();
                    }
                    let mut start = mouse_pos_world;
                    snap_to_grid(&mut start, pa_ref.working_area_grid_step);
                    pa_ref.current_shape = Some(Shape::new(
                        ShapeType::CubicBezier(vec![start, start, start, start]),
                        snap_val,
                        visual_handle_size,
                    ));
                }
                DrawSquare => {
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.remove_selection();
                    }
                    let mut start = mouse_pos_world;
                    snap_to_grid(&mut start, pa_ref.working_area_grid_step);
                    pa_ref.current_shape = Some(Shape::new(
                        ShapeType::Square(vec![start, start]),
                        snap_val,
                        visual_handle_size,
                    ));
                }
                DrawCircle => {
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.remove_selection();
                    }
                    let mut start = mouse_pos_world;
                    snap_to_grid(&mut start, pa_ref.working_area_grid_step);
                    pa_ref.current_shape = Some(Shape::new(
                        ShapeType::Circle(vec![start, start]),
                        snap_val,
                        visual_handle_size,
                    ));
                }
            }

            // pa_ref.mouse_previous_pos_canvas = mouse_pos_canvas;
            pa_ref.mouse_previous_pos_word = mouse_pos_world;

            drop(pa_ref);
            render(pa.clone());
        }
    }
}
fn on_mouse_move(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        let mut pa_ref = pa.borrow_mut();
        let mouse_state = pa_ref.mouse_state.clone();

        // Get mouse position relative to the canvas
        let rect = pa_ref.canvas.get_bounding_client_rect();
        let mouse_pos_canvas = CXY {
            cx: mouse_event.client_x() as f64 - rect.left(),
            cy: mouse_event.client_y() as f64 - rect.top(),
        };

        let scale = pa_ref.scale;
        let world_offset = pa_ref.canvas_offset;

        let mouse_delta_canvas = mouse_pos_canvas - pa_ref.mouse_previous_pos_canvas;

        let mouse_pos_world = mouse_pos_canvas.to_world(scale, world_offset);
        let delta_pos_world = mouse_pos_world - pa_ref.mouse_previous_pos_word;

        if let MouseState::LeftDown = mouse_state {
            use ToolSelected::*;
            match pa_ref.tool_selected {
                Arrow => {
                    let mut some_shape_selected = false;
                    for shape in pa_ref.shapes.iter_mut() {
                        if shape.get_handle_selected() > -2 {
                            shape.modify(&mouse_pos_world, &delta_pos_world);
                            some_shape_selected = true;
                        }
                    }

                    // move the canvas if no object was selected
                    if !some_shape_selected {
                        pa_ref.canvas_offset += mouse_delta_canvas;
                    }
                }
                Selection => {
                    if let Some(sa) = pa_ref.selection_area.as_mut() {
                        sa[1] += delta_pos_world
                    }
                }
                DrawLine | DrawQuadBezier | DrawCubicBezier | DrawCircle | DrawSquare => {
                    if let Some(shape) = pa_ref.current_shape.as_mut() {
                        shape.modify(&mouse_pos_world, &delta_pos_world);
                    }
                }
            }
        }
        pa_ref.mouse_previous_pos_canvas = mouse_pos_canvas;
        pa_ref.mouse_previous_pos_word = mouse_pos_world;
        drop(pa_ref);
        render(pa.clone());
    }
}
fn on_mouse_up(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(_mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        let mut pa_ref = pa.borrow_mut();
        pa_ref.mouse_state = MouseState::NoButton;

        use ToolSelected::*;
        match pa_ref.tool_selected {
            Arrow => {
                let grid_spacing = pa_ref.working_area_grid_step;
                for shape in pa_ref.shapes.iter_mut() {
                    if shape.get_handle_selected() > -2 {
                        shape.snap(grid_spacing);
                    }
                }
            }
            Selection => {
                let selection_area = pa_ref.selection_area.clone();
                if let Some(sa_raw) = selection_area {
                    let bb_outer = reorder_corners(&[sa_raw[0], sa_raw[1]]);
                    for shape in &mut pa_ref.shapes {
                        let bb_inner: [WXY; 2] = shape.get_bounding_box();
                        if is_box_inside(&bb_outer, &bb_inner) {
                            shape.set_handle_selected(-1);
                        } else {
                            shape.set_handle_selected(-2);
                        }
                    }
                }
                pa_ref.selection_area = None;
            }
            DrawLine | DrawQuadBezier | DrawCubicBezier | DrawCircle | DrawSquare => {
                let grid_spacing = pa_ref.working_area_grid_step;
                let oshape = pa_ref.current_shape.clone();
                if let Some(mut shape) = oshape {
                    shape.snap(grid_spacing);
                    if shape.valid() {
                        pa_ref.shapes.push(shape);
                    }
                    pa_ref.current_shape = None;
                }
            }
        }
        go_to_arrow_tool(&mut pa_ref);
        drop(pa_ref);
        render(pa.clone());
    }
}
fn on_mouse_wheel(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(wheel_event) = event.dyn_into::<WheelEvent>() {
        wheel_event.prevent_default();
        let mut pa_ref = pa.borrow_mut();
        let zoom_factor = 0.05;

        let old_scale = pa_ref.scale;

        // Get mouse position relative to the canvas
        let rect = pa_ref.canvas.get_bounding_client_rect();
        let mouse_pos_canvas = CXY {
            cx: wheel_event.client_x() as f64 - rect.left(),
            cy: wheel_event.client_y() as f64 - rect.top(),
        };

        // Determine the new scale
        let new_scale = if wheel_event.delta_y() < 0. {
            // Zoom in
            (old_scale * (1.0 + zoom_factor)).min(10.0)
        } else {
            // Zoom out
            (old_scale / (1.0 + zoom_factor)).max(0.2)
        };

        let new_canvas_offset_x = pa_ref.canvas_offset.cx
            - (new_scale - old_scale) * (mouse_pos_canvas.cx - pa_ref.canvas_offset.cx) / old_scale;
        let new_canvas_offset_y = pa_ref.canvas_offset.cy
            - (new_scale - old_scale) * (mouse_pos_canvas.cy - pa_ref.canvas_offset.cy) / old_scale;

        pa_ref.canvas_offset = CXY {
            cx: new_canvas_offset_x,
            cy: new_canvas_offset_y,
        };
        pa_ref.scale = new_scale;
        drop(pa_ref);
        render(pa);
    }
}
#[allow(dead_code)]
fn on_mouse_enter(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
#[allow(dead_code)]
fn on_mouse_leave(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
fn on_keydown(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
        console::log_1(&format!("{:?}", keyboard_event.key()).into());
        let mut pa_ref = pa.borrow_mut();
        if keyboard_event.key() == "Delete" || keyboard_event.key() == "Backspace" {
            pa_ref
                .shapes
                .retain(|shape| shape.get_handle_selected() < -1)
        }
        if keyboard_event.key() == "Control" || keyboard_event.key() == "Meta" {
            pa_ref.ctrl_or_meta_pressed = true;
        }

        // if keyboard_event.key() == "c" {
        //     if pa_ref.ctrl_or_meta_pressed {
        //         let copy = pa_ref
        //             .shapes
        //             .iter()
        //             .filter(|shape| shape.get_handle_selected() < -1)
        //             .cloned()
        //             .collect();
        //         pa_ref.shape_buffer_copy_paste = copy;
        //     }
        // }

        // if event.key == "Shift" {
        //     if (editorState === 'pointer') {
        //         this.goToselectionMode();
        //     }
        // }
        drop(pa_ref);
        render(pa.clone());
    }
}
fn on_keyup(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
        // console::log_1(&format!("released {:?}", keyboard_event.key()).into());
        let mut pa_ref = pa.borrow_mut();
        if keyboard_event.key() == "Control" || keyboard_event.key() == "Meta" {
            pa_ref.ctrl_or_meta_pressed = false;
        }
        drop(pa_ref);
        render(pa.clone());
    }
}
#[allow(dead_code)]
fn on_context_menu(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}

// Window events
fn resize_area(pa: Rc<RefCell<PlayingArea>>) {
    let mut pa_ref = pa.borrow_mut();
    let (window_width, window_height) = {
        (
            pa_ref.window.inner_width().unwrap().as_f64().unwrap() as u32,
            pa_ref.window.inner_height().unwrap().as_f64().unwrap() as u32,
        )
    };
    let left_panel_width = pa_ref
        .document
        .get_element_by_id("left-panel")
        .unwrap()
        .get_bounding_client_rect()
        .width() as u32;
    let status_bar_height = pa_ref
        .document
        .get_element_by_id("status-bar")
        .unwrap()
        .get_bounding_client_rect()
        .height() as u32;
    let top_menu_height = pa_ref
        .document
        .get_element_by_id("top-menu")
        .unwrap()
        .get_bounding_client_rect()
        .height() as u32;

    let canvas_width = window_width - left_panel_width;
    let canvas_height = window_height - top_menu_height - status_bar_height;

    pa_ref
        .canvas
        .style()
        .set_property("margin-top", &format!("{}px", top_menu_height))
        .unwrap();
    pa_ref
        .canvas
        .style()
        .set_property("margin-left", &format!("{}px", left_panel_width))
        .unwrap();
    pa_ref.canvas.set_width(canvas_width);
    pa_ref.canvas.set_height(canvas_height);

    // Calculation starting parameters
    let working_area = pa_ref.working_area;
    let canvas_offset = CXY {
        cx: (canvas_width as f64 - working_area.wx).abs() / 2.,
        cy: (canvas_height as f64 - working_area.wy).abs() / 2.,
    };
    let dx = canvas_width as f64 / working_area.wx / 1.2;
    let dy = canvas_height as f64 / working_area.wy / 1.2;
    pa_ref.canvas_offset = canvas_offset;
    pa_ref.scale = dx.min(dy);
}
fn on_window_resize(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    resize_area(pa.clone());
    render(pa.clone());
}
fn on_window_click(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    // if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
    //     // Not a right-click
    //     if mouse_event.buttons() == 1 {
    //         pa.borrow_mut()
    //             .contex_menu
    //             .style()
    //             .set_property("display", "none")
    //             .expect("failed to set display property");
    //     }
    // }
}

// Icons event
fn go_to_arrow_tool(pa_ref: &mut RefMut<'_, PlayingArea>) {
    pa_ref.tool_selected = ToolSelected::Arrow;
    deselect_icons(&pa_ref);
    select_icon(&pa_ref, &"icon-arrow");
}
fn on_arrow_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    go_to_arrow_tool(&mut pa_ref);
    console::log_1(&"AAAA click".into());
}
fn on_arrow_mouseover(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
fn on_selection_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.tool_selected = ToolSelected::Selection;
    deselect_icons(&pa_ref);
    select_icon(&pa_ref, &"icon-selection");
    console::log_1(&"BB click".into());
}
fn on_selection_mouseover(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
fn on_line_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.tool_selected = ToolSelected::DrawLine;
    deselect_icons(&pa_ref);
    select_icon(&pa_ref, &"icon-line");
    console::log_1(&"CC click".into());
}
fn on_line_mouseover(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
fn on_quadbezier_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.tool_selected = ToolSelected::DrawQuadBezier;
    deselect_icons(&pa_ref);
    select_icon(&pa_ref, &"icon-quadbezier");
    console::log_1(&"DD click".into());
}
fn on_quadbezier_mouseover(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
fn on_cubicbezier_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.tool_selected = ToolSelected::DrawCubicBezier;
    deselect_icons(&pa_ref);
    select_icon(&pa_ref, &"icon-cubicbezier");
    console::log_1(&"EE click".into());
}
fn on_cubicbezier_mouseover(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
fn on_square_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.tool_selected = ToolSelected::DrawSquare;
    deselect_icons(&pa_ref);
    select_icon(&pa_ref, &"icon-square");
    console::log_1(&"FF click".into());
}
fn on_square_mouseover(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
fn on_circle_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.tool_selected = ToolSelected::DrawCircle;
    deselect_icons(&pa_ref);
    select_icon(&pa_ref, &"icon-circle");
    console::log_1(&"GG click".into());
}
fn on_circle_mouseover(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}

// Initialization
pub fn create_playing_area(window: Window) -> Result<Rc<RefCell<PlayingArea>>, JsValue> {
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("should have a body on document");
    let canvas = document
        .get_element_by_id("myCanvas")
        .expect("should have myCanvas on the page")
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;
    let contex_menu = document
        .get_element_by_id("contextMenu")
        .expect("should have contextMenu on the page")
        .dyn_into::<web_sys::HtmlElement>()?;

    let document_element = document
        .document_element()
        .ok_or("should have a document element")?;
    let style = window
        .get_computed_style(&document_element)
        .unwrap()
        .unwrap();

    let workpiece_color = style.get_property_value("--canvas-workpiece-color")?;
    let dimension_color = style.get_property_value("--canvas-dimension-color")?;
    let geohelper_color = style.get_property_value("--canvas-geohelper-color")?;
    let origin_color = style.get_property_value("--canvas-origin-color")?;
    let grid_color = style.get_property_value("--canvas-grid-color")?;
    let selection_color = style.get_property_value("--canvas-selection-color")?;
    let selected_color = style.get_property_value("--canvas-selected-color")?;
    let background_color = style.get_property_value("--canvas-background-color")?;

    let dash_pattern = Array::new();
    let solid_pattern = Array::new();
    dash_pattern.push(&JsValue::from_f64(3.0));
    dash_pattern.push(&JsValue::from_f64(3.0));

    // Calculation starting parameters
    let (canvas_width, canvas_height) = { (canvas.width() as f64, canvas.height() as f64) };
    let head_position = WXY { wx: 10., wy: 10. };
    let working_area = WXY {
        wx: 1000.,
        wy: 500.,
    };
    let working_area_grid_step = 10.;
    let working_area_snap_step = 2.;

    let canvas_offset = CXY {
        cx: (canvas_width - working_area.wx) / 2.,
        cy: (canvas_height - working_area.wy) / 2.,
    };
    let scale = 1.;

    let playing_area = Rc::new(RefCell::new(PlayingArea {
        window,
        document,
        body,
        canvas,
        ctx,
        //
        contex_menu,
        user_icons: HashMap::new(),
        shapes: Vec::new(),
        // shape_buffer_copy_paste: Vec::new(),
        current_shape: None,
        tool_selected: ToolSelected::Arrow,
        selection_area: None,
        ctrl_or_meta_pressed: false,
        mouse_state: MouseState::NoButton,
        mouse_previous_pos_canvas: CXY::default(),
        mouse_previous_pos_word: WXY::default(),

        // Real word dimensions
        working_area,

        // Zoom
        scale,
        canvas_offset,
        working_area_grid_step,
        editing_snap_step: working_area_snap_step,

        // Drawing colors
        workpiece_color,
        dimension_color,
        geohelper_color,
        origin_color,
        grid_color,
        selection_color,
        selected_color,
        background_color,

        head_position,
        visual_handle_size: 8.,

        pattern_dashed: JsValue::from(dash_pattern),
        pattern_solid: JsValue::from(solid_pattern),
        //
        precision: 5.,
    }));

    init_window(playing_area.clone())?;
    init_canvas(playing_area.clone())?;
    init_icons(playing_area.clone())?;

    resize_area(playing_area.clone());
    render(playing_area.clone());

    Ok(playing_area)
}
fn init_window(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    // Resize event
    let pa_cloned1 = pa.clone();
    let pa_cloned2 = pa.clone();
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
        on_window_resize(pa_cloned1.clone(), event);
    });
    pa_cloned2
        .borrow_mut()
        .window
        .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
    closure.forget();

    // Click event
    let pa_cloned1 = pa.clone();
    let pa_cloned2 = pa.clone();
    let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
        on_window_click(pa_cloned1.clone(), event);
    });
    pa_cloned2
        .borrow_mut()
        .window
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}
fn init_icons(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    // Arrow icon
    let element_name = "icon-arrow";
    let mut element = get_element(&pa.borrow().document, element_name)?;
    pa.borrow_mut()
        .user_icons
        .insert(element_name.into(), element.clone());
    set_callback(
        pa.clone(),
        "click".into(),
        &mut element,
        Box::new(on_arrow_click),
    )?;
    set_callback(
        pa.clone(),
        "mouseover".into(),
        &mut element,
        Box::new(on_arrow_mouseover),
    )?;

    // Selection icon
    let element_name = "icon-selection";
    let mut element = get_element(&pa.borrow().document, element_name)?;
    pa.borrow_mut()
        .user_icons
        .insert(element_name.into(), element.clone());
    set_callback(
        pa.clone(),
        "click".into(),
        &mut element,
        Box::new(on_selection_click),
    )?;
    set_callback(
        pa.clone(),
        "mouseover".into(),
        &mut element,
        Box::new(on_selection_mouseover),
    )?;

    // Line icon
    let element_name = "icon-line";
    let mut element = get_element(&pa.borrow().document, element_name)?;
    pa.borrow_mut()
        .user_icons
        .insert(element_name.into(), element.clone());
    set_callback(
        pa.clone(),
        "click".into(),
        &mut element,
        Box::new(on_line_click),
    )?;
    set_callback(
        pa.clone(),
        "mouseover".into(),
        &mut element,
        Box::new(on_line_mouseover),
    )?;

    // Quad Bézier icon
    let element_name = "icon-quadbezier";
    let mut element = get_element(&pa.borrow().document, element_name)?;
    pa.borrow_mut()
        .user_icons
        .insert(element_name.into(), element.clone());
    set_callback(
        pa.clone(),
        "click".into(),
        &mut element,
        Box::new(on_quadbezier_click),
    )?;
    set_callback(
        pa.clone(),
        "mouseover".into(),
        &mut element,
        Box::new(on_quadbezier_mouseover),
    )?;

    // Cubic Bézier icon
    let element_name = "icon-cubicbezier";
    let mut element = get_element(&pa.borrow().document, element_name)?;
    pa.borrow_mut()
        .user_icons
        .insert(element_name.into(), element.clone());
    set_callback(
        pa.clone(),
        "click".into(),
        &mut element,
        Box::new(on_cubicbezier_click),
    )?;
    set_callback(
        pa.clone(),
        "mouseover".into(),
        &mut element,
        Box::new(on_cubicbezier_mouseover),
    )?;

    // Square icon
    let element_name = "icon-square";
    let mut element = get_element(&pa.borrow().document, element_name)?;
    pa.borrow_mut()
        .user_icons
        .insert(element_name.into(), element.clone());
    set_callback(
        pa.clone(),
        "click".into(),
        &mut element,
        Box::new(on_square_click),
    )?;
    set_callback(
        pa.clone(),
        "mouseover".into(),
        &mut element,
        Box::new(on_square_mouseover),
    )?;

    // Circle icon
    let element_name = "icon-circle";
    let mut element = get_element(&pa.borrow().document, element_name)?;
    pa.borrow_mut()
        .user_icons
        .insert(element_name.into(), element.clone());
    set_callback(
        pa.clone(),
        "click".into(),
        &mut element,
        Box::new(on_circle_click),
    )?;
    set_callback(
        pa.clone(),
        "mouseover".into(),
        &mut element,
        Box::new(on_circle_mouseover),
    )?;

    Ok(())
}
fn init_canvas(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    let mut element = &pa.borrow().canvas;
    set_callback(
        pa.clone(),
        "mousedown".into(),
        element,
        Box::new(on_mouse_down),
    )?;
    set_callback(
        pa.clone(),
        "mousemove".into(),
        &mut element,
        Box::new(on_mouse_move),
    )?;
    set_callback(
        pa.clone(),
        "mouseup".into(),
        &mut element,
        Box::new(on_mouse_up),
    )?;
    set_callback(
        pa.clone(),
        "mouseenter".into(),
        &mut element,
        Box::new(on_mouse_enter),
    )?;
    set_callback(
        pa.clone(),
        "mouseleave".into(),
        &mut element,
        Box::new(on_mouse_leave),
    )?;
    set_callback(
        pa.clone(),
        "wheel".into(),
        &mut element,
        Box::new(on_mouse_wheel),
    )?;
    set_callback(
        pa.clone(),
        "keydown".into(),
        &mut element,
        Box::new(on_keydown),
    )?;
    set_callback(pa.clone(), "keyup".into(), &mut element, Box::new(on_keyup))?;
    Ok(())
}
fn set_callback(
    pa: Rc<RefCell<PlayingArea>>,
    event_str: String,
    element: &Element,
    callback: ElementCallback,
) -> Result<(), JsValue> {
    let event_str_cloned = event_str.clone();
    let callback: ElementCallback = Box::new(move |pa: Rc<RefCell<PlayingArea>>, e: Event| {
        if let Ok(mouse_event) = e.clone().dyn_into::<MouseEvent>() {
            if mouse_event.type_().as_str() == event_str_cloned {
                callback(pa.clone(), e);
            }
        } else {
            if let Ok(keyboard_event) = e.clone().dyn_into::<KeyboardEvent>() {
                if keyboard_event.type_().as_str() == event_str_cloned {
                    callback(pa.clone(), e);
                }
            }
        }
    });
    let closure = Closure::wrap(Box::new(move |event: Event| {
        callback(pa.clone(), event);
    }) as Box<dyn FnMut(Event)>);
    element
        .add_event_listener_with_callback(&event_str, closure.as_ref().unchecked_ref())
        .map_err(|e| JsValue::from_str(&format!("Failed to add event listener: {:?}", e)))?;
    closure.forget();

    Ok(())
}

// Helpers
fn select_icon(pa_ref: &RefMut<'_, PlayingArea>, name: &str) {
    let element = pa_ref.user_icons.get(name).unwrap().clone();
    if let Ok(html_element) = element.dyn_into::<HtmlElement>() {
        html_element
            .set_attribute("class", "icon icon-selected")
            .expect("Failed to set class attribute");
    }
}
fn deselect_icons(pa_ref: &RefMut<'_, PlayingArea>) {
    for element in pa_ref.user_icons.values() {
        let element_cloned = element.clone();
        element_cloned
            .set_attribute("class", "icon")
            .expect("Failed to set class attribute");
    }
}
fn get_element(document: &Document, element_id: &str) -> Result<Element, JsValue> {
    let element = document
        .get_element_by_id(element_id)
        .ok_or_else(|| JsValue::from_str("should have element on the page"))?
        .dyn_into()
        .map_err(|_| JsValue::from_str("should be an HtmlElement"))?;
    Ok(element)
}

// Rendering
fn render(pa: Rc<RefCell<PlayingArea>>) {
    let pa_ref = pa.borrow_mut();

    // First clear the drawing area
    pa_ref.ctx.set_stroke_style(&"#F00".into());
    let background_color = &pa_ref.background_color;
    pa_ref.ctx.set_fill_style(&background_color.into());

    pa_ref.ctx.fill();
    let (canvas_width, canvas_height) =
        { (pa_ref.canvas.width() as f64, pa_ref.canvas.height() as f64) };
    pa_ref
        .ctx
        .fill_rect(0., 0., canvas_width as f64, canvas_height as f64);

    drop(pa_ref);

    // Then draw all
    draw_all(pa.clone());
}
fn draw_all(pa: Rc<RefCell<PlayingArea>>) {
    draw_grid(pa.clone());
    draw_working_area(pa.clone());

    // draw_origin(pa.clone());
    draw_content(pa.clone());
}

fn draw_working_area(pa: Rc<RefCell<PlayingArea>>) {
    let pa_ref = pa.borrow();

    // Debug: check good canvas size
    let (canvas_width, canvas_height) =
        { (pa_ref.canvas.width() as f64, pa_ref.canvas.height() as f64) };
    let solid_pattern = pa_ref.pattern_solid.clone();
    pa_ref.ctx.set_line_dash(&solid_pattern).unwrap();
    pa_ref.ctx.set_line_width(1.);
    pa_ref.ctx.set_stroke_style(&"FFF".into());
    let p = Path2d::new().unwrap();
    p.move_to(1., 1.);
    p.line_to(1., canvas_height);
    p.line_to(canvas_width - 1., canvas_height - 1.);
    p.line_to(canvas_width - 1., 1.);
    p.line_to(1., 1.);
    pa_ref.ctx.begin_path();
    pa_ref.ctx.stroke_with_path(&p);

    // Draw working area
    let mut cst = Vec::new();
    let wa = pa_ref.working_area;
    cst.push(ConstructionType::Move(WXY { wx: 0., wy: 0. }));
    cst.push(ConstructionType::Line(WXY { wx: 0., wy: wa.wy }));
    cst.push(ConstructionType::Line(WXY {
        wx: wa.wx,
        wy: wa.wy,
    }));
    cst.push(ConstructionType::Line(WXY { wx: wa.wx, wy: 0. }));
    cst.push(ConstructionType::Line(WXY { wx: 0., wy: 0. }));

    raw_draw(&pa_ref, &cst, LayerType::WorkPiece);
}

fn draw_grid(pa: Rc<RefCell<PlayingArea>>) {
    let pa_ref = pa.borrow();

    let wa = pa_ref.working_area;
    let w_grid_spacing = pa_ref.working_area_grid_step;
    // Vertical grid lines
    let mut cst = Vec::new();
    let mut wx = 0.;
    while wx <= wa.wx {
        cst.push(ConstructionType::Move(WXY { wx: wx, wy: 0. }));
        cst.push(ConstructionType::Line(WXY { wx: wx, wy: wa.wy }));
        raw_draw(&pa_ref, &cst, LayerType::Grid);
        wx += w_grid_spacing;
    }

    // Horizontal grid lines
    let mut cst = Vec::new();
    let mut wy = 0.;
    while wy <= wa.wy {
        cst.push(ConstructionType::Move(WXY { wx: 0., wy: wy }));
        cst.push(ConstructionType::Line(WXY { wx: wa.wx, wy: wy }));
        raw_draw(&pa_ref, &cst, LayerType::Grid);
        wy += w_grid_spacing;
    }
}
// fn draw_origin(pa: Rc<RefCell<PlayingArea>>) {
//     let pa_ref = pa.borrow_mut();
//     let circle_radius = 10.;
//     let cross_length = 15.; // Adjust as needed
//     // Draw circle
//     pa_ref.ctx.begin_path();
//     pa_ref
//         .ctx
//         .arc(
//             pa_ref.head_position.wx,
//             pa_ref.head_position.wy,
//             circle_radius,
//             0.,
//             2. * PI,
//         )
//         .unwrap();
//     pa_ref.ctx.set_fill_style(&"#000".into()); // Black color for the circle
//     pa_ref.ctx.fill();
//     pa_ref.ctx.close_path();
//     // Draw the rotated cross
//     pa_ref.ctx.set_fill_style(&"#FFF".into()); // White color for the cross to contrast with the black circle
//     pa_ref.ctx.set_line_width(2.); // Adjust as needed
//     // Vertical line of the cross
//     pa_ref.ctx.begin_path();
//     pa_ref.ctx.move_to(
//         pa_ref.head_position.wx,
//         -cross_length / 2. + pa_ref.head_position.wy,
//     );
//     pa_ref.ctx.line_to(
//         pa_ref.head_position.wx,
//         cross_length / 2. + pa_ref.head_position.wy,
//     );
//     pa_ref.ctx.stroke();
//     // Horizontal line of the cross
//     pa_ref.ctx.begin_path();
//     pa_ref.ctx.move_to(
//         pa_ref.head_position.wx - cross_length / 2.,
//         pa_ref.head_position.wy,
//     );
//     pa_ref.ctx.line_to(
//         pa_ref.head_position.wx + cross_length / 2.,
//         pa_ref.head_position.wy,
//     );
//     pa_ref.ctx.stroke();
// }

fn draw_content(pa: Rc<RefCell<PlayingArea>>) {
    let pa_ref = pa.borrow();

    // Draw all shapes
    for shape in pa_ref.shapes.iter() {
        shape.get_construction();
        raw_draw(&pa_ref, &shape.get_construction(), LayerType::WorkPiece);
        if shape.is_selected() {
            raw_draw(
                &pa_ref,
                &shape.get_handles_construction(),
                LayerType::WorkPiece,
            );
            raw_draw(
                &pa_ref,
                &&shape.get_snap_construction(),
                LayerType::GeometryHelpers,
            );
        }
    }

    // Draw the current drawing shape
    if let Some(shape) = pa_ref.current_shape.as_ref() {
        shape.get_construction();
        raw_draw(&pa_ref, &shape.get_construction(), LayerType::WorkPiece);
        raw_draw(
            &pa_ref,
            &shape.get_handles_construction(),
            LayerType::WorkPiece,
        );
        raw_draw(
            &pa_ref,
            &&shape.get_snap_construction(),
            LayerType::GeometryHelpers,
        );
    }

    // If using a selection area, draw it
    // draw_selection(&pa_ref);
}

// fn draw_selection(pa_ref: &Ref<'_, PlayingArea>) {
//     if let Some(sa) = pa_ref.selection_area {
//         let bl = sa[0];
//         let tr = sa[1];
//         if bl.wx != tr.wx && bl.wy != tr.wy {
//             let stroke_light = pa_ref.stroke_light.clone();
//             let dash_pattern = pa_ref.pattern_dashed.clone();
//             pa_ref.ctx.set_stroke_style(&stroke_light.into());
//             pa_ref.ctx.set_line_dash(&dash_pattern).unwrap();
//             let p = Path2d::new().unwrap();
//             p.move_to(bl.wx, bl.wy);
//             p.line_to(bl.wx, tr.wy);
//             p.line_to(tr.wx, tr.wy);
//             p.line_to(tr.wx, bl.wy);
//             p.line_to(bl.wx, bl.wy);
//             pa_ref.ctx.begin_path();
//             pa_ref.ctx.stroke_with_path(&p);
//         }
//     }
// }
// fn draw_shape(pa_ref: &Ref<'_, PlayingArea>, shape: &Shape) {
//     // Shape draw
//     let stroke_default = pa_ref.stroke_default.clone();
//     let solid_pattern = pa_ref.pattern_solid.clone();
//     pa_ref.ctx.set_stroke_style(&stroke_default.into());
//     pa_ref.ctx.set_line_dash(&solid_pattern).unwrap();

//     pa_ref.ctx.begin_path();
//     pa_ref.ctx.stroke_with_path(&shape.get_construction());
//     // Handles draw
//     for (handle_pos, selected) in shape.get_handles_positions() {
//         draw_handle(pa_ref, handle_pos, selected);
//     }
//     // Snap draw
//     if shape.get_handle_selected() > -2 {
//         for snap in shape.get_snaps().iter() {
//             match snap.0 {
//                 SnapType::Geometry(idx1, idx2) => {
//                     let pt1 = shape.get_handle(idx1);
//                     let pt2 = shape.get_handle(idx2);
//                     if let Some((pta, ptb)) = get_segment(&pt1, &pt2, snap.1) {
//                         draw_construction_line(pa_ref, &pta, &ptb);
//                     }
//                 }
//                 SnapType::Middle(idx_middle, idxs) => {
//                     if let SegmentSnapping::Middle = snap.1 {
//                         let pt = shape.get_handle(idx_middle);
//                         let pt1 = shape.get_handle(idxs[0]);
//                         let pt2 = shape.get_handle(idxs[1]);
//                         let pt_mid = (pt1 + pt2) / 2.;
//                         draw_construction_line(pa_ref, &pt, &pt_mid);
//                     }
//                 }
//             }
//         }
//     }
// }
// fn draw_construction_line(pa_ref: &Ref<'_, PlayingArea>, start: &WXY, end: &WXY) {
//     let stroke_light = pa_ref.stroke_construction.clone();
//     let dash_pattern = pa_ref.pattern_dashed.clone();
//     pa_ref.ctx.set_stroke_style(&stroke_light.into());
//     pa_ref.ctx.set_line_dash(&dash_pattern).unwrap();
//     let p = Path2d::new().unwrap();
//     p.move_to(start.wx, start.wy);
//     p.line_to(end.wx, end.wy);
//     pa_ref.ctx.begin_path();
//     pa_ref.ctx.stroke_with_path(&p);
// }

fn raw_draw(pa_ref: &Ref<'_, PlayingArea>, cst: &Vec<ConstructionType>, layer: LayerType) {
    use LayerType::*;
    let (color, line_dash, line_width) = match layer {
        WorkPiece => (&pa_ref.workpiece_color, &pa_ref.pattern_solid, 1.),
        Dimension => (&pa_ref.dimension_color, &pa_ref.pattern_solid, 1.),
        GeometryHelpers => (&pa_ref.geohelper_color, &pa_ref.pattern_dashed, 1.),
        Origin => (&pa_ref.origin_color, &pa_ref.pattern_solid, 1.),
        Grid => (&pa_ref.grid_color, &pa_ref.pattern_solid, 1.),
        Selection => (&pa_ref.selection_color, &pa_ref.pattern_dashed, 1.),
        Selected => (&pa_ref.selected_color, &pa_ref.pattern_solid, 1.),
        Handle => (&pa_ref.workpiece_color, &pa_ref.pattern_solid, 1.),
    };
    pa_ref.ctx.set_line_dash(line_dash).unwrap();
    pa_ref.ctx.set_line_width(line_width);
    pa_ref.ctx.set_stroke_style(&color.into());
    pa_ref.ctx.set_fill_style(&color.into());

    let p = Path2d::new().unwrap();
    let scale = pa_ref.scale;
    let offset = pa_ref.canvas_offset;
    for prim in cst.iter() {
        use ConstructionType::*;
        match prim {
            Move(w_end) => {
                let c_end = w_end.to_canvas(scale, offset);
                p.move_to(c_end.cx, c_end.cy);
            }
            Line(w_end) => {
                let c_end = w_end.to_canvas(scale, offset);
                p.line_to(c_end.cx, c_end.cy);
            }
            Quadratic(w_ctrl, w_end) => {
                let c_ctrl = w_ctrl.to_canvas(scale, offset);
                let c_end = w_end.to_canvas(scale, offset);
                p.quadratic_curve_to(c_ctrl.cx, c_ctrl.cy, c_end.cx, c_end.cy);
            }
            Bezier(w_ctrl1, w_crtl2, w_end) => {
                let c_ctrl1 = w_ctrl1.to_canvas(scale, offset);
                let c_ctrl2 = w_crtl2.to_canvas(scale, offset);
                let c_end = w_end.to_canvas(scale, offset);
                p.bezier_curve_to(
                    c_ctrl1.cx, c_ctrl1.cy, c_ctrl2.cx, c_ctrl2.cy, c_end.cx, c_end.cy,
                );
            }
            Ellipse(w_center, radius, rotation, start_angle, end_angle) => {
                let c_center = w_center.to_canvas(scale, offset);
                let _ = p.ellipse(
                    c_center.cx,
                    c_center.cy,
                    radius.wx * scale,
                    radius.wy * scale,
                    *rotation,
                    *start_angle,
                    *end_angle,
                );
            }
            Rectangle(w_start, w_dimensions, fill) => {
                let c_start = w_start.to_canvas(scale, offset);
                let c_dimensions = *w_dimensions * scale;
                console::log_1(
                    &format!("w_dimensions {:?} {:?}", w_dimensions.wx, w_dimensions.wy).into(),
                );
                console::log_1(
                    &format!("c_dimensions {:?} {:?}", c_dimensions.wx, c_dimensions.wy).into(),
                );
                if *fill {
                    pa_ref.ctx.fill();
                    pa_ref
                        .ctx
                        .fill_rect(c_start.cx, c_start.cy, c_dimensions.wx, c_dimensions.wy);
                } else {
                    p.rect(c_start.cx, c_start.cy, c_dimensions.wx, c_dimensions.wy);
                }
            }
        }
    }
    pa_ref.ctx.begin_path(); // Begin a new path
    pa_ref.ctx.stroke_with_path(&p);
}
