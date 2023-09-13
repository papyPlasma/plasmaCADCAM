use crate::math::*;
use crate::shapes::{SegmentSnapping, Shape, ShapeType, SnapType, XY};
use js_sys::Array;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    console, CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement, HtmlElement,
    MouseEvent, Path2d, WheelEvent, Window,
};

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
#[allow(dead_code)]
#[repr(u16)]
enum MouseState {
    NoButton = 0,
    LeftDown = 1,
    RightDown = 2,
    MiddleDown = 4,
}
#[allow(dead_code)]
pub struct PlayingArea {
    pub window: Window,
    pub document: Document,
    pub body: HtmlElement,
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,

    // DOM
    contex_menu: HtmlElement,
    user_icons: HashMap<String, Element>,
    //
    shapes: Vec<Shape>,
    current_shape: Option<Shape>,
    tool_selected: ToolSelected,
    //
    mouse_state: MouseState,
    mouse_previous_pos_abs: XY,
    mouse_previous_pos_rel: XY,
    // Zoom
    scale: f64,
    offset: XY,
    // Canvas parameters
    grid_spacing: f64,
    axis_color: &'static str,
    grid_color: &'static str,
    shape_color: &'static str,
    head_position: XY,
    handle_size: f64,
    //
    pub stroke_selected: String,
    pub stroke_default: String,
    pub stroke_light: String,
    pub dash_pattern: JsValue,
    pub solid_pattern: JsValue,
    //
    precision: f64,
    snap_val: f64,
}
impl PlayingArea {
    fn get_mouse_pos_rel(&self, mouse_pos_abs: &XY) -> XY {
        let rect = self.canvas.get_bounding_client_rect();
        let click_x = mouse_pos_abs.x - rect.left() - self.offset.x;
        let click_y = mouse_pos_abs.y - rect.top() - self.offset.y;
        let rel_x = click_x / self.scale;
        let rel_y = self.canvas.height() as f64 - click_y / self.scale;
        XY { x: rel_x, y: rel_y }
    }
}
// console::log_1(&format!("{:?}", tool_selected).into());
// Canvas events: mouse, keyboard and context menu
fn on_mouse_down(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        if mouse_event.buttons() == MouseState::LeftDown as u16 {
            let mut pa_ref = pa.borrow_mut();
            pa_ref.mouse_state = MouseState::LeftDown;
            let mouse_pos_abs = XY {
                x: mouse_event.client_x() as f64,
                y: mouse_event.client_y() as f64,
            };
            let mouse_pos_rel = pa_ref.get_mouse_pos_rel(&mouse_pos_abs);

            use ToolSelected::*;
            match pa_ref.tool_selected {
                Arrow => {
                    let precision = pa_ref.precision;
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.set_selection(&mouse_pos_rel, precision);
                    }
                }
                Selection => (),
                DrawLine => {
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.remove_selection();
                    }
                    let mut start = mouse_pos_rel;
                    snap_to_grid(&mut start, pa_ref.grid_spacing);
                    let snap_val = pa_ref.snap_val;
                    pa_ref.current_shape =
                        Some(Shape::new(ShapeType::Line(vec![start, start]), snap_val));
                }
                DrawQuadBezier => {
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.remove_selection();
                    }
                    let mut start = mouse_pos_rel;
                    snap_to_grid(&mut start, pa_ref.grid_spacing);
                    let snap_val = pa_ref.snap_val;
                    pa_ref.current_shape = Some(Shape::new(
                        ShapeType::QuadBezier(vec![start, start, start]),
                        snap_val,
                    ));
                }
                DrawCubicBezier => {
                    for shape in pa_ref.shapes.iter_mut() {
                        shape.remove_selection();
                    }
                    let mut start = mouse_pos_rel;
                    snap_to_grid(&mut start, pa_ref.grid_spacing);
                    let snap_val = pa_ref.snap_val;
                    pa_ref.current_shape = Some(Shape::new(
                        ShapeType::CubicBezier(vec![start, start, start, start]),
                        snap_val,
                    ));
                }
                DrawCircle => (),
                DrawSquare => (),
            }

            pa_ref.mouse_previous_pos_abs = mouse_pos_abs;
            pa_ref.mouse_previous_pos_rel = mouse_pos_rel;

            drop(pa_ref);
            render(pa.clone());
        }
    }
}
fn on_mouse_move(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        let mouse_state = pa.borrow_mut().mouse_state.clone();
        let mut pa_ref = pa.borrow_mut();
        let mouse_pos_abs = XY {
            x: mouse_event.client_x() as f64,
            y: mouse_event.client_y() as f64,
        };
        let delta_pos_abs = mouse_pos_abs - pa_ref.mouse_previous_pos_abs;
        let mouse_pos_rel = pa_ref.get_mouse_pos_rel(&mouse_pos_abs);
        let delta_pos_rel = mouse_pos_rel - pa_ref.mouse_previous_pos_rel;

        if let MouseState::LeftDown = mouse_state {
            use ToolSelected::*;
            match pa_ref.tool_selected {
                Arrow => {
                    let mut some_shape_selected = false;
                    for shape in pa_ref.shapes.iter_mut() {
                        if shape.get_handle_selected() > -2 {
                            shape.modify(&mouse_pos_rel, &delta_pos_rel);
                            some_shape_selected = true;
                        }
                    }

                    // move the canvas if no object was selected
                    if !some_shape_selected {
                        // Calculate how far the cursor has moved
                        // TBD
                        // Adjust offsets by this distance
                        pa_ref.offset += delta_pos_abs;
                    }
                }
                Selection => (),
                DrawLine | DrawQuadBezier | DrawCubicBezier | DrawCircle | DrawSquare => {
                    if let Some(shape) = pa_ref.current_shape.as_mut() {
                        shape.modify(&mouse_pos_rel, &delta_pos_rel);
                    }
                }
            }
        }
        pa_ref.mouse_previous_pos_abs = mouse_pos_abs;
        pa_ref.mouse_previous_pos_rel = mouse_pos_rel;
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
                let grid_spacing = pa_ref.grid_spacing;
                for shape in pa_ref.shapes.iter_mut() {
                    if shape.get_handle_selected() > -2 {
                        shape.snap(grid_spacing);
                    }
                }
            }
            Selection => (),
            DrawLine | DrawQuadBezier | DrawCubicBezier | DrawCircle | DrawSquare => {
                let grid_spacing = pa_ref.grid_spacing;
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
#[allow(dead_code)]
fn on_mouse_enter(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
#[allow(dead_code)]
fn on_mouse_leave(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
fn on_mouse_wheel(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(wheel_event) = event.dyn_into::<WheelEvent>() {
        wheel_event.prevent_default();
        let mut pa_ref = pa.borrow_mut();
        let zoom_factor = 0.1;
        let mut new_scale;

        // Get mouse position relative to the canvas
        let rect = pa_ref.canvas.get_bounding_client_rect();
        let pos = XY {
            x: wheel_event.client_x() as f64 - rect.left(),
            y: wheel_event.client_y() as f64 - rect.top(),
        };

        // Compute the transformation center: We compute the current mouse position in "world coordinates"
        let world = XY {
            x: (pos.x - pa_ref.offset.x) / pa_ref.scale,
            y: (pos.y - pa_ref.offset.y) / pa_ref.scale,
        };

        if wheel_event.delta_y() < 0. {
            // Zoom in
            new_scale = pa_ref.scale * (1. + zoom_factor);
            if new_scale > 10. {
                new_scale = 10.;
            }
        } else {
            // Zoom out
            new_scale = pa_ref.scale / (1. + zoom_factor);
            if new_scale < 1. {
                new_scale = 1.;
            }
        }

        // Compute the new offset after the scaling: The idea here is to first apply the scaling transformation
        // and then translate the world so that the point under the mouse remains in the same place
        pa_ref.offset.x = pos.x - world.x * new_scale;
        pa_ref.offset.y = pos.y - world.y * new_scale;

        pa_ref.scale = new_scale;

        drop(pa_ref);
        render(pa);
    }
}
#[allow(dead_code)]
fn on_keydown(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
#[allow(dead_code)]
fn on_context_menu(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}

// Window events
fn resize_area(pa: Rc<RefCell<PlayingArea>>) {
    let mut pa_ref = pa.borrow_mut();
    let (canvas_height, scale, window_inner_height) = {
        (
            pa_ref.canvas.height() as f64,
            pa_ref.scale,
            pa_ref.window.inner_height().unwrap().as_f64().unwrap(),
        )
    };
    pa_ref.offset.y = -(canvas_height - (window_inner_height - 110.)) * scale;
}
fn on_window_resize(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    //console::log_1(&format!("x:{:?} y:{:?}", canvas_width, canvas_height).into());
    //console::log_1(&"ddd".into());
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

    canvas.set_width(1500); // minus left-panel width
    canvas.set_height(3000); // minus top-menu height

    let document_element = document
        .document_element()
        .ok_or("should have a document element")?;
    let style = window
        .get_computed_style(&document_element)
        .unwrap()
        .unwrap();
    let stroke_selected = style.get_property_value("--canvas-stroke-selection")?;
    let stroke_default = style.get_property_value("--canvas-stroke-default")?;
    let stroke_light = style.get_property_value("--canvas-stroke-light")?;
    let dash_pattern = Array::new();
    let solid_pattern = Array::new();
    dash_pattern.push(&JsValue::from_f64(3.0));
    dash_pattern.push(&JsValue::from_f64(3.0));

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
        current_shape: None,
        tool_selected: ToolSelected::Arrow,
        mouse_state: MouseState::NoButton,
        mouse_previous_pos_abs: XY::default(),
        mouse_previous_pos_rel: XY::default(),
        // Zoom
        scale: 1.0,
        offset: XY::default(),
        grid_spacing: 10.,
        axis_color: "#000",
        grid_color: "#eee",
        shape_color: "#000",
        head_position: XY { x: 10., y: 10. },
        handle_size: 8.,
        //
        stroke_selected,
        stroke_default,
        stroke_light,
        dash_pattern: JsValue::from(dash_pattern),
        solid_pattern: JsValue::from(solid_pattern),
        //
        precision: 5.,
        snap_val: 4.,
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

    pa_ref
        .ctx
        .set_transform(1., 0., 0., 1., 0., 0.)
        .expect("couille dans l'potage");
    let (canvas_width, canvas_height, scale, offset) = {
        (
            pa_ref.canvas.width() as f64,
            pa_ref.canvas.height() as f64,
            pa_ref.scale,
            pa_ref.offset,
        )
    };
    pa_ref
        .ctx
        .clear_rect(0., 0., canvas_width as f64, canvas_height as f64);
    pa_ref
        .ctx
        .set_transform(scale, 0., 0., scale, offset.x, offset.y)
        .expect("couille dans l'potage");
    // Set the origin to the bottom left
    pa_ref
        .ctx
        .translate(0., canvas_height as f64)
        .expect("couille dans l'potage"); // Translate by the canvas height
    pa_ref.ctx.scale(1., -1.).expect("couille dans l'potage"); // Flip vertically

    drop(pa_ref);
    draw_all(pa.clone());
}
fn draw_all(pa: Rc<RefCell<PlayingArea>>) {
    draw_grid(pa.clone());
    draw_origin(pa.clone());
    draw_content(pa.clone());
}
fn draw_grid(pa: Rc<RefCell<PlayingArea>>) {
    let pa_ref = pa.borrow();

    let grid_color = pa_ref.grid_color.clone();
    let solid_pattern = pa_ref.solid_pattern.clone();
    pa_ref.ctx.set_stroke_style(&grid_color.into());
    pa_ref.ctx.set_line_dash(&solid_pattern).unwrap();
    pa_ref.ctx.set_line_width(1.);

    let (canvas_width, canvas_height, grid_spacing) = {
        (
            pa_ref.canvas.width() as f64,
            pa_ref.canvas.height() as f64,
            pa_ref.grid_spacing,
        )
    };

    // Vertical grid lines
    let mut x = 0.;
    while x <= canvas_width {
        pa_ref.ctx.begin_path();
        pa_ref.ctx.move_to(x, 0.);
        pa_ref.ctx.line_to(x, canvas_height as f64);
        pa_ref.ctx.stroke();
        x += grid_spacing;
    }

    // Horizontal grid lines
    let mut y = 0.;
    while y <= canvas_height {
        pa_ref.ctx.begin_path();
        pa_ref.ctx.move_to(0., y);
        pa_ref.ctx.line_to(canvas_width, y);
        pa_ref.ctx.stroke();
        y += grid_spacing;
    }
}
fn draw_origin(pa: Rc<RefCell<PlayingArea>>) {
    let pa_ref = pa.borrow_mut();

    let circle_radius = 10.;
    let cross_length = 15.; // Adjust as needed

    // Draw circle
    pa_ref.ctx.begin_path();
    pa_ref
        .ctx
        .arc(
            pa_ref.head_position.x,
            pa_ref.head_position.y,
            circle_radius,
            0.,
            2. * PI,
        )
        .unwrap();
    pa_ref.ctx.set_fill_style(&"#000".into()); // Black color for the circle
    pa_ref.ctx.fill();
    pa_ref.ctx.close_path();

    // Draw the rotated cross
    pa_ref.ctx.set_fill_style(&"#FFF".into()); // White color for the cross to contrast with the black circle
    pa_ref.ctx.set_line_width(2.); // Adjust as needed

    // Vertical line of the cross
    pa_ref.ctx.begin_path();
    pa_ref.ctx.move_to(
        pa_ref.head_position.x,
        -cross_length / 2. + pa_ref.head_position.y,
    );
    pa_ref.ctx.line_to(
        pa_ref.head_position.x,
        cross_length / 2. + pa_ref.head_position.y,
    );
    pa_ref.ctx.stroke();

    // Horizontal line of the cross
    pa_ref.ctx.begin_path();
    pa_ref.ctx.move_to(
        pa_ref.head_position.x - cross_length / 2.,
        pa_ref.head_position.y,
    );
    pa_ref.ctx.line_to(
        pa_ref.head_position.x + cross_length / 2.,
        pa_ref.head_position.y,
    );
    pa_ref.ctx.stroke();
}
fn draw_content(pa: Rc<RefCell<PlayingArea>>) {
    let pa_ref = pa.borrow();

    let shape_color = pa_ref.shape_color;

    pa_ref.ctx.set_line_width(1.);
    pa_ref.ctx.set_stroke_style(&shape_color.into());

    for shape in pa_ref.shapes.iter() {
        draw_shape(&pa_ref, shape);
    }

    if let Some(shape) = pa_ref.current_shape.as_ref() {
        draw_shape(&pa_ref, shape);
    }

    // const bl = this.selectionArea.bl;
    // const tr = this.selectionArea.tr;
    // if (bl.x !== tr.x && bl.y !== tr.y) {
    //     let p = new Path2D();
    //     this.ctx.strokeStyle = strokeLight;
    //     this.ctx.setLineDash([3, 3]);
    //     p.moveTo(bl.x, bl.y);
    //     p.lineTo(bl.x, tr.y);
    //     p.lineTo(tr.x, tr.y);
    //     p.lineTo(tr.x, bl.y);
    //     p.lineTo(bl.x, bl.y);
    //     this.ctx.stroke(p);
    //     this.ctx.setLineDash([]);
    //     this.ctx.strokeStyle = strokeDefault;
    // }
}

fn draw_shape(pa_ref: &Ref<'_, PlayingArea>, shape: &Shape) {
    // Shape draw
    let stroke_default = pa_ref.stroke_default.clone();
    let solid_pattern = pa_ref.solid_pattern.clone();
    pa_ref.ctx.set_stroke_style(&stroke_default.into());
    pa_ref.ctx.set_line_dash(&solid_pattern).unwrap();

    pa_ref.ctx.begin_path();
    pa_ref.ctx.stroke_with_path(&shape.get_path_shape());

    // Handles draw
    for (handle_pos, selected) in shape.get_handles_positions() {
        draw_handle(pa_ref, handle_pos, selected);
    }

    // Snap draw
    if shape.get_handle_selected() > -2 {
        for snap in shape.get_snaps().iter() {
            match snap.0 {
                SnapType::Geometry(idx1, idx2) => {
                    let pt1 = shape.get_handle(idx1);
                    let pt2 = shape.get_handle(idx2);

                    if let Some((pta, ptb)) = get_segment(&pt1, &pt2, snap.1) {
                        draw_dotted_line(pa_ref, &pta, &ptb);
                    }
                }
                SnapType::Middle(idx_middle, idxs) => {
                    if let SegmentSnapping::Middle = snap.1 {
                        let pt = shape.get_handle(idx_middle);
                        let pt1 = shape.get_handle(idxs[0]);
                        let pt2 = shape.get_handle(idxs[1]);
                        let pt_mid = (pt1 + pt2) / 2.;
                        draw_dotted_line(pa_ref, &pt, &pt_mid);
                    }
                }
            }
        }
    }
}
fn draw_dotted_line(pa_ref: &Ref<'_, PlayingArea>, start: &XY, end: &XY) {
    let stroke_light = pa_ref.stroke_light.clone();
    let dash_pattern = pa_ref.dash_pattern.clone();
    pa_ref.ctx.set_stroke_style(&stroke_light.into());
    pa_ref.ctx.set_line_dash(&dash_pattern).unwrap();
    let p = Path2d::new().unwrap();
    p.move_to(start.x, start.y);
    p.line_to(end.x, end.y);
    pa_ref.ctx.begin_path();
    pa_ref.ctx.stroke_with_path(&p);
}
fn draw_handle(pa_ref: &Ref<'_, PlayingArea>, handle_pos: XY, selected: bool) {
    let stroke_default = pa_ref.stroke_default.clone();
    let solid_pattern = pa_ref.solid_pattern.clone();
    pa_ref.ctx.set_stroke_style(&stroke_default.into());
    pa_ref.ctx.set_line_dash(&solid_pattern).unwrap();

    let p = Path2d::new().unwrap();
    if selected {
        pa_ref.ctx.set_fill_style(&"black".into());
    } else {
        pa_ref.ctx.set_fill_style(&"white".into());
    }

    // Define the path using the Path2d instance
    p.rect(
        handle_pos.x - pa_ref.handle_size / 2.,
        handle_pos.y - pa_ref.handle_size / 2.,
        pa_ref.handle_size,
        pa_ref.handle_size,
    );

    pa_ref.ctx.begin_path(); // Begin a new path
    pa_ref.ctx.fill_with_path_2d(&p); // Fill the path defined on the Path2d instance
    pa_ref.ctx.stroke_with_path(&p); // Stroke the path defined on the Path2d instance
}
