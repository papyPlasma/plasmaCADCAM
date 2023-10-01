use crate::math::*;
use crate::shapes::{
    ConstructionType, CubicBezier, Ellipse, Group, LayerType, Line, QuadBezier, Rectangle,
    ShapeParameters,
};
use js_sys::Array;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::f64::consts::PI;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use web_sys::{
    console, CanvasRenderingContext2d, Document, Element, Event, FileList, FileReader,
    HtmlCanvasElement, HtmlElement, HtmlInputElement, KeyboardEvent, MouseEvent, Path2d,
    WheelEvent, Window,
};

//console::log_1(&format!("{:?}", xxx).into());
//console::log_1(&"ddd".into());

pub type ElementCallback = Box<dyn Fn(Rc<RefCell<PlayingArea>>, Event) + 'static>;

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
    pub pool: DataPool,
    //
    pub window: Window,
    pub document: Document,
    pub body: HtmlElement,
    pub canvas: HtmlCanvasElement,
    pub ctx: CanvasRenderingContext2d,

    // DOM
    #[allow(dead_code)]
    contex_menu: HtmlElement,
    user_icons: HashMap<&'static str, Option<Element>>,
    tooltip: HtmlElement,
    settings_panel: HtmlElement,
    modal_backdrop: HtmlElement,
    apply_settings_button: HtmlElement,
    settings_width_input: HtmlInputElement,
    settings_height_input: HtmlInputElement,
    // Mouse position on worksheet
    mouse_worksheet_position: HtmlElement,
    _viewgrid_element: HtmlElement,
    _snapgrid_element: HtmlElement,

    root_shape_id: usize,
    current_drawing_shape_id: Option<usize>,
    current_selected_shapes_ids: Vec<usize>,
    current_selected_point_id: Option<usize>,

    icon_selected: &'static str,
    selection_area: Option<[WPoint; 2]>,
    ctrl_or_meta_pressed: bool,
    //
    mouse_state: MouseState,
    mouse_previous_pos_canvas: CXY,
    mouse_previous_pos_word: WPoint,
    mouse_down_world_coord: WPoint,

    working_area: WPoint,
    global_scale: f64,
    canvas_offset: CXY,
    working_area_visual_grid: f64,
    working_area_snap_grid: f64,

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
    pub pattern_dashed: JsValue,
    pub pattern_solid: JsValue,

    shape_parameters: ShapeParameters,
}

// Initialization
pub fn create_playing_area(window: Window) -> Result<Rc<RefCell<PlayingArea>>, JsValue> {
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("should have a body on document");
    let canvas = document
        .get_element_by_id("myCanvas")
        .expect("should have myCanvas on the page")
        .dyn_into::<HtmlCanvasElement>()?;
    let ctx = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;
    // ctx.scale(1., -1.)?;
    let contex_menu = document
        .get_element_by_id("contextMenu")
        .expect("should have contextMenu on the page")
        .dyn_into::<HtmlElement>()?;
    let tooltip = document
        .get_element_by_id("tooltip")
        .expect("should have tooltip on the page")
        .dyn_into::<HtmlElement>()?;
    let settings_panel = document
        .get_element_by_id("settingsPanel")
        .expect("should have settingsPanel on the page")
        .dyn_into::<HtmlElement>()?;
    let modal_backdrop = document
        .get_element_by_id("modalBackdrop")
        .expect("should have modalBackdrop on the page")
        .dyn_into::<HtmlElement>()?;

    let apply_settings_button = document
        .get_element_by_id("applyWorksheetSettings")
        .expect("should have applyWorksheetSettings on settingsPanel")
        .dyn_into::<HtmlElement>()?;
    let settings_width_input: HtmlInputElement = document
        .get_element_by_id("worksheetWidthInput")
        .expect("should have settings_width_input on settingsPanel")
        .dyn_into()?;
    let settings_height_input: HtmlInputElement = document
        .get_element_by_id("worksheetHeightInput")
        .expect("should have settings_height_input on settingsPanel")
        .dyn_into()?;
    let mouse_worksheet_position: HtmlElement = document
        .get_element_by_id("status-info-worksheet-pos")
        .expect("should have status-info-worksheet-pos on the page")
        .dyn_into()?;
    let viewgrid_element: HtmlElement = document
        .get_element_by_id("status-viewgrid")
        .expect("should have status-viewgrid on the page")
        .dyn_into()?;
    let snapgrid_element: HtmlElement = document
        .get_element_by_id("status-snapgrid")
        .expect("should have status-snapgrid on the page")
        .dyn_into()?;

    let mut user_icons: HashMap<&'static str, Option<Element>> = HashMap::new();
    user_icons.insert("icon-arrow", None);
    user_icons.insert("icon-selection", None);
    user_icons.insert("icon-line", None);
    user_icons.insert("icon-quadbezier", None);
    user_icons.insert("icon-cubicbezier", None);
    user_icons.insert("icon-rectangle", None);
    user_icons.insert("icon-ellipse", None);
    user_icons.insert("icon-cog", None);

    let document_element = document
        .document_element()
        .ok_or("should have a document element")?;
    let style = window
        .get_computed_style(&document_element)
        .unwrap()
        .unwrap();

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
    let solid_pattern = Array::new();
    dash_pattern.push(&JsValue::from_f64(3.0));
    dash_pattern.push(&JsValue::from_f64(3.0));

    // Calculation starting parameters
    let (canvas_width, canvas_height) = { (canvas.width() as f64, canvas.height() as f64) };
    // let head_position = WXY { wx: 10., wy: 10. };
    let working_area = WPoint {
        wx: 1000.,
        wy: 500.,
    };
    settings_width_input.set_value(&working_area.wx.to_string());
    settings_height_input.set_value(&working_area.wy.to_string());

    let working_area_visual_grid = 10.;
    let working_area_snap_grid = 1.;

    let canvas_offset = CXY {
        cx: (canvas_width - working_area.wx) / 2.,
        cy: (canvas_height - working_area.wy) / 2.,
    };
    let global_scale = 1.;

    let shape_parameters = ShapeParameters {
        grab_handle_precision: 3.,
        handles_size: 6.,
        highlight_size: 12.,
    };

    let mut pool = DataPool::new();
    let root_shape_id = Group::new(&mut pool, &shape_parameters);

    let playing_area = Rc::new(RefCell::new(PlayingArea {
        pool,
        window,
        document,
        body,
        canvas,
        ctx,
        //
        contex_menu,
        user_icons,
        tooltip,
        settings_panel,
        modal_backdrop,
        apply_settings_button,
        settings_width_input,
        settings_height_input,
        mouse_worksheet_position,
        _viewgrid_element: viewgrid_element,
        _snapgrid_element: snapgrid_element,

        root_shape_id,
        current_selected_shapes_ids: vec![],
        current_drawing_shape_id: None,
        current_selected_point_id: None,

        icon_selected: "icon-arrow",
        selection_area: None,
        ctrl_or_meta_pressed: false,
        mouse_state: MouseState::NoButton,
        mouse_previous_pos_canvas: CXY::default(),
        mouse_previous_pos_word: WPoint::default(),
        mouse_down_world_coord: WPoint::default(),

        // Real word dimensions
        working_area,

        // Zoom
        global_scale,
        canvas_offset,
        working_area_visual_grid,
        working_area_snap_grid,

        // Drawing colors
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

        shape_parameters,
    }));

    init_window(playing_area.clone())?;
    init_menu(playing_area.clone())?;
    init_canvas(playing_area.clone())?;
    init_context_menu(playing_area.clone())?;
    init_icons(playing_area.clone())?;
    init_settings_panel(playing_area.clone())?;
    init_status(playing_area.clone())?;

    resize_area(playing_area.clone());
    render(playing_area.clone());

    Ok(playing_area)
}
fn init_window(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    // Resize event
    let pa_cloned1 = pa.clone();
    let pa_cloned2 = pa.clone();
    let closure = Closure::<dyn FnMut(_)>::new(move |event: Event| {
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
    let closure = Closure::<dyn FnMut(_)>::new(move |event: Event| {
        on_window_click(pa_cloned1.clone(), event);
    });
    pa_cloned2
        .borrow_mut()
        .window
        .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    closure.forget();

    Ok(())
}
fn init_settings_panel(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    let pa_ref = pa.borrow_mut();
    set_callback(
        pa.clone(),
        "click".into(),
        &pa_ref.apply_settings_button,
        Box::new(on_apply_settings_click),
    )?;
    set_callback(
        pa.clone(),
        "click".into(),
        &pa_ref.modal_backdrop,
        Box::new(on_modal_backdrop_click),
    )?;
    Ok(())
}
fn init_icons(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    let mut pa_ref = pa.borrow_mut();
    let document = pa_ref.document.clone();
    for (element_name, element_to_set) in pa_ref.user_icons.iter_mut() {
        if let Some(element) = get_element(&document, element_name).ok() {
            *element_to_set = Some(element);
            set_callback(
                pa.clone(),
                "click".into(),
                &element_to_set.as_ref().unwrap(),
                Box::new(on_icon_click),
            )?;
            set_callback(
                pa.clone(),
                "mouseover".into(),
                &element_to_set.as_ref().unwrap(),
                Box::new(on_icon_mouseover),
            )?;
            set_callback(
                pa.clone(),
                "mouseout".into(),
                &element_to_set.as_ref().unwrap(),
                Box::new(on_icon_mouseout),
            )?;
        }
    }

    Ok(())
}
fn init_context_menu(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    let pa_ref = pa.borrow_mut();
    let document = pa_ref.document.clone();
    let action_group = document.get_element_by_id("action-group").unwrap();
    set_callback(
        pa.clone(),
        "click".into(),
        &action_group,
        Box::new(on_context_menu_group_click),
    )?;
    Ok(())
}
fn init_canvas(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    let mut element = &pa.borrow().canvas;
    set_callback(
        pa.clone(),
        "contextmenu".into(),
        element,
        Box::new(on_context_menu),
    )?;
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
fn init_menu(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    let pa_mut = pa.borrow_mut();
    let document = pa_mut.document.clone();

    let load_element = document.get_element_by_id("load-option").unwrap();
    let load_element: HtmlElement = load_element.dyn_into::<HtmlElement>()?;

    let save_element = document.get_element_by_id("save-option").unwrap();
    let save_element: HtmlElement = save_element.dyn_into::<HtmlElement>()?;

    let file_input = document.get_element_by_id("file-input").unwrap();
    let file_input: HtmlElement = file_input.dyn_into::<HtmlElement>()?;

    let file_input_clone = file_input.clone();
    let on_load = Closure::wrap(Box::new(move || {
        // Trigger a click event on the file input element to open the file dialog
        file_input_clone.click();
    }) as Box<dyn FnMut()>);

    let on_save = Closure::wrap(Box::new(move || {
        // Your save action here
        console::log_1(&"Save clicked".into());
    }) as Box<dyn FnMut()>);

    load_element.add_event_listener_with_callback("click", on_load.as_ref().unchecked_ref())?;
    on_load.forget(); // Leaks memory, but we need to do this to keep the callback alive

    save_element.add_event_listener_with_callback("click", on_save.as_ref().unchecked_ref())?;
    on_save.forget(); // Leaks memory, but we need to do this to keep the callback alive

    drop(pa_mut);
    // Set up an event listener to handle file selection
    let on_file_select = Closure::wrap(Box::new(move || {
        // console::log_1(&"File selected".into());
        let pa_clone = pa.clone();
        // Get the files from the file input element
        if let Some(file_input) = document.get_element_by_id("file-input") {
            let file_input: HtmlInputElement = file_input.dyn_into().unwrap();
            let files = js_sys::Reflect::get(&file_input.into(), &"files".into())
                .unwrap()
                .dyn_into::<FileList>()
                .unwrap();
            if let Some(file) = files.get(0) {
                // let file_name = file.name();
                let file_reader = FileReader::new().unwrap();

                let on_load = Closure::wrap(Box::new(move |event: Event| {
                    let target = event.target().unwrap();
                    let file_reader: FileReader = target.dyn_into().unwrap();
                    let result = file_reader.result().unwrap();
                    if let Some(content) = result.as_string() {
                        // console::log_1(&format!("content: {:?}", content).into());
                        let shape_id = convert_svg_to_shapes(pa_clone.clone(), content.clone());
                        let mut pa_mut = pa_clone.borrow_mut();
                        let root_shape_id = pa_mut.root_shape_id;
                        pa_mut
                            .pool
                            .get_shape_mut(root_shape_id)
                            .unwrap()
                            .add_child_shape_id(shape_id);
                        drop(pa_clone.borrow_mut());
                        render(pa_clone.clone());
                    }
                }) as Box<dyn FnMut(_)>);

                file_reader
                    .add_event_listener_with_callback("load", on_load.as_ref().unchecked_ref())
                    .unwrap();
                on_load.forget(); // Avoid memory leak

                file_reader.read_as_text(&file).unwrap();
            }
        }
    }) as Box<dyn FnMut()>);

    file_input
        .add_event_listener_with_callback("change", on_file_select.as_ref().unchecked_ref())?;
    on_file_select.forget(); // Leaks memory, but we need to do this to keep the callback alive

    Ok(())
}
fn init_status(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    let pa_ref = pa.borrow_mut();
    let _document = pa_ref.document.clone();

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

fn convert_svg_to_shapes(pa: Rc<RefCell<PlayingArea>>, svg_data: String) -> usize {
    let mut pa_mut = pa.borrow_mut();
    let snap_distance = pa_mut.working_area_snap_grid;
    let shape_parameters = pa_mut.shape_parameters;

    let mut shapes_ids = Vec::new();

    for event in svg::parser::Parser::new(&svg_data).into_iter() {
        // console::log_1(&format!("event: {:?}", event).into());
        match event {
            svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) => {
                let data = attributes.get("d").unwrap();
                let data = svg::node::element::path::Data::parse(data).unwrap();
                let mut current_position = WPoint::default();
                let mut start_position = WPoint::default();
                let mut last_quad_control_point: Option<WPoint> = None;
                let mut last_cubic_control_point: Option<WPoint> = None;

                for command in data.iter() {
                    let command_clone = command.clone();
                    use svg::node::element::path::*;
                    match command_clone {
                        Command::Move(postype, params) => {
                            if params.len() == 2 {
                                current_position = match postype {
                                    Position::Absolute => WPoint {
                                        wx: params[0] as f64,
                                        wy: params[1] as f64,
                                    },
                                    Position::Relative => WPoint {
                                        wx: params[0] as f64 + current_position.wx,
                                        wy: params[1] as f64 + current_position.wy,
                                    },
                                };
                                start_position = current_position;
                                last_quad_control_point = None;
                                last_cubic_control_point = None;
                            }
                        }
                        Command::Line(postype, params) => {
                            if params.len() % 2 == 0 {
                                let nb_curves = params.len() / 2;
                                for curve in 0..nb_curves {
                                    let end_point = WPoint {
                                        wx: params[2 * curve] as f64,
                                        wy: params[2 * curve + 1] as f64,
                                    };
                                    let new_position = match postype {
                                        Position::Absolute => end_point,
                                        Position::Relative => current_position + end_point,
                                    };
                                    let shape_id = Line::new(
                                        &mut pa_mut.pool,
                                        &current_position,
                                        &new_position,
                                        &shape_parameters,
                                        snap_distance,
                                    );
                                    shapes_ids.push(shape_id);
                                    current_position = new_position;
                                    last_quad_control_point = None;
                                    last_cubic_control_point = None;
                                }
                            }
                        }
                        Command::HorizontalLine(postype, params) => {
                            for curve in 0..params.len() {
                                let end_point = WPoint {
                                    wx: params[curve] as f64,
                                    wy: current_position.wy,
                                };
                                let new_position = match postype {
                                    Position::Absolute => end_point,
                                    Position::Relative => current_position + end_point,
                                };
                                let shape_id = Line::new(
                                    &mut pa_mut.pool,
                                    &current_position,
                                    &new_position,
                                    &shape_parameters,
                                    snap_distance,
                                );
                                shapes_ids.push(shape_id);
                                current_position = new_position;
                                last_quad_control_point = None;
                                last_cubic_control_point = None;
                            }
                        }
                        Command::VerticalLine(postype, params) => {
                            for curve in 0..params.len() {
                                let end_point = WPoint {
                                    wx: current_position.wx,
                                    wy: params[curve] as f64,
                                };
                                let new_position = match postype {
                                    Position::Absolute => end_point,
                                    Position::Relative => current_position + end_point,
                                };
                                let shape_id = Line::new(
                                    &mut pa_mut.pool,
                                    &current_position,
                                    &new_position,
                                    &shape_parameters,
                                    snap_distance,
                                );
                                shapes_ids.push(shape_id);
                                current_position = new_position;
                                last_quad_control_point = None;
                                last_cubic_control_point = None;
                            }
                        }
                        Command::QuadraticCurve(postype, params) => {
                            if params.len() % 4 == 0 {
                                let nb_curves = params.len() / 4;
                                for curve in 0..nb_curves {
                                    let mut control_point = WPoint {
                                        wx: params[4 * curve] as f64,
                                        wy: params[4 * curve + 1] as f64,
                                    };
                                    let end_point = WPoint {
                                        wx: params[4 * curve + 2] as f64,
                                        wy: params[4 * curve + 3] as f64,
                                    };
                                    let new_position = match postype {
                                        Position::Absolute => end_point,
                                        Position::Relative => {
                                            control_point += current_position;
                                            current_position + end_point
                                        }
                                    };
                                    let shape_id = QuadBezier::new(
                                        &mut pa_mut.pool,
                                        &current_position,
                                        &control_point,
                                        &new_position,
                                        &shape_parameters,
                                        snap_distance,
                                    );
                                    shapes_ids.push(shape_id);
                                    current_position = new_position;
                                    last_quad_control_point = Some(control_point);
                                    last_cubic_control_point = None;
                                }
                            }
                        }
                        Command::SmoothQuadraticCurve(postype, params) => {
                            if params.len() % 2 == 0 {
                                let nb_curves = params.len() / 2;
                                for curve in 0..nb_curves {
                                    let control_point =
                                        if let Some(last_ctrl_pt) = last_quad_control_point {
                                            current_position + (current_position - last_ctrl_pt)
                                        } else {
                                            current_position
                                        };
                                    let end_point = WPoint {
                                        wx: params[2 * curve] as f64,
                                        wy: params[2 * curve + 1] as f64,
                                    };
                                    let new_position = match postype {
                                        Position::Absolute => end_point,
                                        Position::Relative => current_position + end_point,
                                    };
                                    let shape_id = QuadBezier::new(
                                        &mut pa_mut.pool,
                                        &current_position,
                                        &control_point,
                                        &new_position,
                                        &shape_parameters,
                                        snap_distance,
                                    );
                                    shapes_ids.push(shape_id);
                                    current_position = new_position;
                                    last_quad_control_point = Some(control_point);
                                    last_cubic_control_point = None;
                                }
                            }
                        }
                        Command::CubicCurve(postype, params) => {
                            if params.len() % 6 == 0 {
                                let nb_curves = params.len() / 6;
                                for curve in 0..nb_curves {
                                    let mut control_point1 = WPoint {
                                        wx: params[6 * curve] as f64,
                                        wy: params[6 * curve + 1] as f64,
                                    };
                                    let mut control_point2 = WPoint {
                                        wx: params[6 * curve + 2] as f64,
                                        wy: params[6 * curve + 3] as f64,
                                    };
                                    let end_point = WPoint {
                                        wx: params[6 * curve + 4] as f64,
                                        wy: params[6 * curve + 5] as f64,
                                    };
                                    let new_position = match postype {
                                        Position::Absolute => end_point,
                                        Position::Relative => {
                                            control_point1 += current_position;
                                            control_point2 += current_position;
                                            current_position + end_point
                                        }
                                    };
                                    let shape_id = CubicBezier::new(
                                        &mut pa_mut.pool,
                                        &current_position,
                                        &control_point1,
                                        &control_point2,
                                        &new_position,
                                        &shape_parameters,
                                        snap_distance,
                                    );
                                    shapes_ids.push(shape_id);
                                    current_position = new_position;
                                    last_quad_control_point = None;
                                    last_cubic_control_point = Some(control_point2);
                                }
                            }
                        }
                        Command::SmoothCubicCurve(postype, params) => {
                            if params.len() % 4 == 0 {
                                let nb_curves = params.len() / 4;
                                for curve in 0..nb_curves {
                                    let control_point1 =
                                        if let Some(last_ctrl_pt) = last_cubic_control_point {
                                            current_position + (current_position - last_ctrl_pt)
                                        } else {
                                            current_position
                                        };
                                    let mut control_point2 = WPoint {
                                        wx: params[4 * curve] as f64,
                                        wy: params[4 * curve + 1] as f64,
                                    };
                                    let end_point = WPoint {
                                        wx: params[4 * curve + 2] as f64,
                                        wy: params[4 * curve + 3] as f64,
                                    };
                                    let new_position = match postype {
                                        Position::Absolute => end_point,
                                        Position::Relative => {
                                            control_point2 += current_position;
                                            current_position + end_point
                                        }
                                    };
                                    let shape_id = CubicBezier::new(
                                        &mut pa_mut.pool,
                                        &current_position,
                                        &control_point1,
                                        &control_point2,
                                        &new_position,
                                        &shape_parameters,
                                        snap_distance,
                                    );
                                    shapes_ids.push(shape_id);
                                    current_position = new_position;
                                    last_quad_control_point = None;
                                    last_cubic_control_point = Some(control_point2);
                                }
                            }
                        }
                        Command::EllipticalArc(_postype, _params) => {}
                        Command::Close => {
                            let shape_id = Line::new(
                                &mut pa_mut.pool,
                                &current_position,
                                &start_position,
                                &shape_parameters,
                                snap_distance,
                            );
                            shapes_ids.push(shape_id);
                            current_position = start_position;
                            last_quad_control_point = None;
                            last_cubic_control_point = None;
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let shape_group_id = Group::new(&mut pa_mut.pool, &shape_parameters);
    for shape_id in shapes_ids.iter() {
        let shape = pa_mut.pool.get_shape_mut(*shape_id).unwrap();
        shape.set_parent(*shape_id);
        let shape_group = pa_mut.pool.get_shape_mut(shape_group_id).unwrap();
        shape_group.add_child_shape_id(*shape_id);
    }
    shape_group_id
}

///////////////
// Canvas events: mouse, keyboard and context menu
fn on_mouse_down(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        if mouse_event.buttons() == MouseState::LeftDown as u16 {
            let mut pa_ref = pa.borrow_mut();
            if let Some(context_menu) = pa_ref.document.get_element_by_id("contextMenu") {
                if let Some(html_element) =
                    wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlElement>(&context_menu)
                {
                    // Hide the context menu when clicking elsewhere
                    html_element
                        .style()
                        .set_property("display", "none")
                        .unwrap();
                }
            }

            pa_ref.mouse_state = MouseState::LeftDown;

            // Get mouse position relative to the canvas
            let rect = pa_ref.canvas.get_bounding_client_rect();
            let mouse_pos_canvas = CXY {
                cx: mouse_event.client_x() as f64 - rect.left(),
                cy: mouse_event.client_y() as f64 - rect.top(),
            };

            let scale = pa_ref.global_scale;
            let offset = pa_ref.canvas_offset;

            let mouse_pos_world = mouse_pos_canvas.to_world(scale, offset);
            pa_ref.mouse_previous_pos_word = mouse_pos_world;
            pa_ref.mouse_down_world_coord = mouse_pos_world;

            let shape_parameters = pa_ref.shape_parameters;
            let snap_distance = pa_ref.working_area_snap_grid;
            let mut start = mouse_pos_world;
            snap_to_snap_grid(&mut start, pa_ref.working_area_snap_grid);

            match pa_ref.icon_selected {
                "icon-arrow" => {
                    let mut current_selected_point_id = None;
                    let mut current_selected_shapes_ids = vec![];

                    let opick_pt_id = pa_ref.pool.get_point_id_from_position(
                        &mouse_pos_world,
                        shape_parameters.grab_handle_precision,
                    );
                    if let Some(pick_pt_id) = opick_pt_id {
                        for shape_containing_pt_id in
                            pa_ref.pool.get_shapes_ids_containing_pt_id(pick_pt_id)
                        {
                            let shape = pa_ref.pool.get_shape_mut(pick_pt_id).unwrap();
                            if shape.is_selected() {
                                current_selected_point_id = Some(pick_pt_id);
                            } else {
                                shape.set_selected();
                            }
                            current_selected_shapes_ids.push(shape_containing_pt_id);
                        }
                    }
                    pa_ref.current_selected_shapes_ids = current_selected_shapes_ids;
                    pa_ref.current_selected_point_id = current_selected_point_id;
                }
                "icon-selection" => {
                    pa_ref.selection_area = Some([mouse_pos_world, mouse_pos_world])
                }
                "icon-line" => {
                    let shape_id = Line::new(
                        &mut pa_ref.pool,
                        &start,
                        &start,
                        &shape_parameters,
                        snap_distance,
                    );
                    let shape = pa_ref.pool.get_shape(shape_id).unwrap();
                    pa_ref.current_selected_point_id = Some(shape.get_pts_ids()[1]);
                    pa_ref.current_drawing_shape_id = Some(shape_id);
                }
                "icon-quadbezier" => {
                    let shape_id = QuadBezier::new(
                        &mut pa_ref.pool,
                        &start,
                        &start,
                        &start,
                        &shape_parameters,
                        snap_distance,
                    );
                    let shape = pa_ref.pool.get_shape(shape_id).unwrap();
                    pa_ref.current_selected_point_id = Some(shape.get_pts_ids()[2]);
                    pa_ref.current_drawing_shape_id = Some(shape_id);
                }
                "icon-cubicbezier" => {
                    let shape_id = CubicBezier::new(
                        &mut pa_ref.pool,
                        &start,
                        &start,
                        &start,
                        &start,
                        &shape_parameters,
                        snap_distance,
                    );
                    let shape = pa_ref.pool.get_shape(shape_id).unwrap();
                    pa_ref.current_selected_point_id = Some(shape.get_pts_ids()[3]);
                    pa_ref.current_drawing_shape_id = Some(shape_id);
                }
                "icon-rectangle" => {
                    let shape_id = Rectangle::new(
                        &mut pa_ref.pool,
                        &start,
                        0.,
                        0.,
                        &shape_parameters,
                        snap_distance,
                    );
                    let shape = pa_ref.pool.get_shape(shape_id).unwrap();
                    pa_ref.current_selected_point_id = Some(shape.get_pts_ids()[2]);
                    pa_ref.current_drawing_shape_id = Some(shape_id);
                }
                "icon-ellipse" => {
                    let shape_id = Ellipse::new(
                        &mut pa_ref.pool,
                        &start,
                        &WPoint::default(),
                        0.,
                        2. * PI,
                        &shape_parameters,
                        snap_distance,
                    );
                    let shape = pa_ref.pool.get_shape(shape_id).unwrap();
                    pa_ref.current_selected_point_id = Some(shape.get_pts_ids()[1]);
                    pa_ref.current_drawing_shape_id = Some(shape_id);
                }
                _ => (),
            }

            // Update display mouse world position
            pa_ref
                .mouse_worksheet_position
                .set_text_content(Some(&format!(
                    "( {:?} , {:?} ) - ( {:?} , {:?} )",
                    mouse_pos_world.wx.round() as i32,
                    mouse_pos_world.wy.round() as i32,
                    (mouse_pos_world.wx - pa_ref.mouse_down_world_coord.wx).round() as i32,
                    (mouse_pos_world.wy - pa_ref.mouse_down_world_coord.wy).round() as i32
                )));

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

        let scale = pa_ref.global_scale;
        let world_offset = pa_ref.canvas_offset;

        let mouse_delta_canvas = mouse_pos_canvas - pa_ref.mouse_previous_pos_canvas;

        let mouse_pos_world = mouse_pos_canvas.to_world(scale, world_offset);
        let delta_pos_world = mouse_pos_world - pa_ref.mouse_previous_pos_word;

        if let MouseState::LeftDown = mouse_state {
            match pa_ref.icon_selected {
                "icon-arrow" => {
                    if let Some(current_selected_point_id) = pa_ref.current_selected_point_id {
                        let shapes_containing_pt_id = pa_ref
                            .pool
                            .get_shapes_ids_containing_pt_id(current_selected_point_id);
                        {
                            pa_ref.pool.move_point(
                                current_selected_point_id,
                                &shapes_containing_pt_id,
                                &mouse_pos_world,
                            );
                        }
                    } else {
                        // move the canvas if no object was selected
                        pa_ref.canvas_offset += mouse_delta_canvas;
                    }
                }
                "icon-selection" => {
                    if let Some(sa) = pa_ref.selection_area.as_mut() {
                        sa[1] += delta_pos_world
                    }
                }
                "icon-line" | "icon-quadbezier" | "icon-cubicbezier" | "icon-ellipse"
                | "icon-rectangle" => {
                    let pt_selected = pa_ref.current_selected_point_id.unwrap();
                    let shape_id = pa_ref.current_drawing_shape_id.unwrap();
                    pa_ref
                        .pool
                        .move_point(pt_selected, &vec![shape_id], &mouse_pos_world);
                }
                _ => (),
            }

            // Update display mouse world position
            pa_ref
                .mouse_worksheet_position
                .set_text_content(Some(&format!(
                    "( {:?} , {:?} ) - ( {:?} , {:?} )",
                    mouse_pos_world.wx.round() as i32,
                    mouse_pos_world.wy.round() as i32,
                    (mouse_pos_world.wx - pa_ref.mouse_down_world_coord.wx).round() as i32,
                    (mouse_pos_world.wy - pa_ref.mouse_down_world_coord.wy).round() as i32
                )));
        } else {
            pa_ref
                .mouse_worksheet_position
                .set_text_content(Some(&format!(
                    "( {:?} , {:?} )",
                    mouse_pos_world.wx.round() as i32,
                    mouse_pos_world.wy.round() as i32
                )));
        }

        pa_ref.mouse_previous_pos_canvas = mouse_pos_canvas;
        pa_ref.mouse_previous_pos_word = mouse_pos_world;

        drop(pa_ref);
        render(pa.clone());
    }
}
fn on_mouse_up(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(_mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        let mut pa_mut = pa.borrow_mut();
        pa_mut.mouse_state = MouseState::NoButton;

        match pa_mut.icon_selected {
            "icon-selection" => {
                let selection_area = pa_mut.selection_area.clone();
                if let Some(sa_raw) = selection_area {
                    let mut bb_outer = sa_raw;
                    reorder_corners(&mut bb_outer);
                    pa_mut.pool.select_shapes_bounded_by_rectangle(bb_outer);
                }
                pa_mut.selection_area = None;
            }
            "icon-line" | "icon-quadbezier" | "icon-cubicbezier" | "icon-ellipse"
            | "icon-rectangle" => {
                // Here we are drawing a new shape
                let oshape_id = pa_mut.current_drawing_shape_id;
                if let Some(shape_id) = oshape_id {
                    pa_mut.pool.get_shape_mut(shape_id).unwrap().init_done();

                    // let mut shapes = None;
                    // for shape_in_pool in &mut pa_mut.shapes {
                    //     // Grouping if any
                    //     if let Some(highlight) = shape_in_pool.get_highlight() {
                    //         shapes = Some((
                    //             (shape_in_pool.get_id(), highlight),
                    //             (current_drawing_shape.get_id(), usize::End),
                    //         ));

                    //         // reset highlighting
                    //         shape_in_pool.remove_highlight();

                    //         // only one group for two handles
                    //         break;
                    //     }
                    // }

                    // if let Some((shape1, shape2)) = shapes {
                    //     pa_mut.groups.add(shape1, shape2);
                    // };
                    pa_mut.current_selected_point_id = None;
                    pa_mut.current_drawing_shape_id = None;
                }
            }
            _ => (),
        }
        go_to_arrow_tool(&mut pa_mut);
        drop(pa_mut);
        render(pa.clone());
    }
}
fn on_mouse_wheel(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(wheel_event) = event.dyn_into::<WheelEvent>() {
        wheel_event.prevent_default();
        let mut pa_ref = pa.borrow_mut();
        let zoom_factor = 0.05;

        let old_scale = pa_ref.global_scale;

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
        pa_ref.global_scale = new_scale;
        drop(pa_ref);
        render(pa);
    }
}
fn on_mouse_enter(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.mouse_state = MouseState::NoButton;
}
fn on_mouse_leave(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.mouse_state = MouseState::NoButton;
}
fn on_keydown(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
        //console::log_1(&format!("{:?}", keyboard_event.key()).into());
        let mut pa_ref = pa.borrow_mut();
        // if keyboard_event.key() == "Delete" || keyboard_event.key() == "Backspace" {
        //     pa_ref
        //         .all_shapes
        //         .retain(|shape| !shape.has_shape_or_point_selected())
        // }
        // if keyboard_event.key() == "Control" || keyboard_event.key() == "Meta" {
        //     pa_ref.ctrl_or_meta_pressed = true;
        // }
        // if keyboard_event.key() == "s" {
        //     if let ToolSelected::Arrow = pa_ref.icon_selected {
        //         pa_ref.icon_selected = ToolSelected::Selection;
        //         deselect_icons(&pa_ref);
        //         select_icon(&pa_ref, &"icon-selection");
        //     }
        // }
        // if keyboard_event.key() == "l" {
        //     if let ToolSelected::Arrow = pa_ref.icon_selected {
        //         pa_ref.icon_selected = ToolSelected::DrawLine;
        //         deselect_icons(&pa_ref);
        //         select_icon(&pa_ref, &"icon-line");
        //     }
        // }
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
fn on_context_menu(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    let pa_ref = pa.borrow_mut();
    // Prevent the default context menu from appearing
    event.prevent_default();
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        if let Some(context_menu) = pa_ref.document.get_element_by_id("contextMenu") {
            if let Ok(html_element) = context_menu.dyn_into::<web_sys::HtmlElement>() {
                // Position the context menu at the right-click position
                html_element
                    .style()
                    .set_property("top", &format!("{}px", mouse_event.client_y()))
                    .unwrap();
                html_element
                    .style()
                    .set_property("left", &format!("{}px", mouse_event.client_x()))
                    .unwrap();

                // Show the context menu
                html_element
                    .style()
                    .set_property("display", "block")
                    .unwrap();
            }
        }
    }
}
fn on_context_menu_group_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let pa_ref = pa.borrow_mut();
    if let Some(context_menu) = pa_ref.document.get_element_by_id("contextMenu") {
        if let Some(html_element) =
            wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlElement>(&context_menu)
        {
            // Hide the context menu when clicking elsewhere
            html_element
                .style()
                .set_property("display", "none")
                .unwrap();
        }
    }
}
///////////////
/// Settings panel events
fn on_apply_settings_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let mut pa_ref = pa.borrow_mut();

    let width_str = pa_ref.settings_width_input.value();
    let height_str = pa_ref.settings_height_input.value();
    let width: f64 = width_str.parse().unwrap_or(0.0);
    let height: f64 = height_str.parse().unwrap_or(0.0);
    pa_ref
        .settings_panel
        .style()
        .set_property("display", "none")
        .unwrap();
    pa_ref
        .modal_backdrop
        .style()
        .set_property("display", "none")
        .unwrap();

    pa_ref.working_area = WPoint {
        wx: width,
        wy: height,
    };

    drop(pa_ref);
    resize_area(pa.clone());
    render(pa.clone());
}
fn on_modal_backdrop_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    let pa_ref = pa.borrow_mut();
    pa_ref
        .settings_panel
        .style()
        .set_property("display", "none")
        .unwrap();
    pa_ref
        .modal_backdrop
        .style()
        .set_property("display", "none")
        .unwrap();
    pa_ref
        .settings_width_input
        .set_value(&pa_ref.working_area.wx.to_string());
    pa_ref
        .settings_height_input
        .set_value(&pa_ref.working_area.wy.to_string());
}
///////////////
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
    pa_ref.global_scale = dx.min(dy);
}
fn on_window_resize(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    resize_area(pa.clone());
    render(pa.clone());
}
fn on_window_click(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    // let pa_ref = pa.borrow_mut();
    // if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
    //     // Not a right-click
    //     if mouse_event.buttons() == 1 {
    //         let target = event.target().unwrap();
    //         let target = target.dyn_into::<web_sys::Node>().unwrap();
    //         if !pa_ref.settings_panel.contains(Some(&target)) {
    //             pa_ref
    //                 .settings_panel
    //                 .style()
    //                 .set_property("display", "none")
    //                 .unwrap();
    //             pa_ref
    //                 .modal_backdrop
    //                 .style()
    //                 .set_property("display", "none")
    //                 .unwrap();
    //         }
    //     }
    // }
}
///////////////
// Icons events
fn on_icon_click(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    let mut pa_ref = pa.borrow_mut();
    if let Some(target) = event.target() {
        if let Some(element) = wasm_bindgen::JsCast::dyn_ref::<Element>(&target) {
            if let Some(id) = element.get_attribute("id") {
                if let Some(key) = pa_ref.user_icons.keys().find(|&&k| k == id) {
                    if key == &"icon-cog" {
                        pa_ref
                            .settings_panel
                            .style()
                            .set_property("display", "block")
                            .unwrap();
                        pa_ref
                            .modal_backdrop
                            .style()
                            .set_property("display", "block")
                            .unwrap();
                    } else {
                        pa_ref.icon_selected = key;
                        deselect_icons(&pa_ref);
                        select_icon(&pa_ref, &id);
                    }
                }
            }
        }
    }
}
fn on_icon_mouseover(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    let pa_ref = pa.borrow();
    if let Some(target) = event.target() {
        if let Some(element) = wasm_bindgen::JsCast::dyn_ref::<Element>(&target) {
            if let Some(data_tooltip) = element.get_attribute("data-tooltip") {
                let tooltip_html = &pa_ref.tooltip;
                tooltip_html.set_inner_text(&data_tooltip);
                tooltip_html
                    .style()
                    .set_property("display", "block")
                    .unwrap();
                if let Some(mouse_event) = event.dyn_ref::<MouseEvent>() {
                    let x = mouse_event.page_x();
                    let y = mouse_event.page_y();
                    tooltip_html
                        .style()
                        .set_property("left", &format!("{}px", x + 10))
                        .unwrap();
                    tooltip_html
                        .style()
                        .set_property("top", &format!("{}px", y + 10))
                        .unwrap();
                }
            }
        }
    }
}
fn on_icon_mouseout(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    pa.borrow_mut()
        .tooltip
        .style()
        .set_property("display", "none")
        .expect("Failed to set display property");
}

// Helpers
fn go_to_arrow_tool(pa_ref: &mut RefMut<'_, PlayingArea>) {
    pa_ref.icon_selected = "icon-arrow";
    deselect_icons(&pa_ref);
    select_icon(&pa_ref, "icon-arrow");
}
fn select_icon(pa_ref: &RefMut<'_, PlayingArea>, name: &str) {
    if let Some(element) = pa_ref.user_icons.get(name).unwrap().clone() {
        if let Ok(html_element) = element.dyn_into::<HtmlElement>() {
            html_element
                .set_attribute("class", "icon icon-selected")
                .expect("Failed to set class attribute");
        }
    }
}
fn deselect_icons(pa_ref: &RefMut<'_, PlayingArea>) {
    for (key, oelement) in pa_ref.user_icons.iter() {
        if key != &"icon-cog" {
            if let Some(element) = oelement {
                // let element_cloned = element.clone();
                element
                    .set_attribute("class", "icon")
                    .expect("Failed to set class attribute");
            }
        }
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
    let pa_ref = pa.borrow();

    // Clear the canvas
    raw_draw_clear_canvas(&pa_ref);
    drop(pa_ref);

    // Then draw all
    draw_all(pa.clone());
}

fn draw_all(pa: Rc<RefCell<PlayingArea>>) {
    draw_grid(pa.clone());
    draw_working_area(pa.clone());
    draw_content(pa.clone());
    draw_selection(pa.clone());
}

fn draw_working_area(pa: Rc<RefCell<PlayingArea>>) {
    use ConstructionType::*;
    let pa_ref = pa.borrow();

    // Draw working area
    let mut cst = Vec::new();
    let wa = pa_ref.working_area;

    cst.push(Layer(LayerType::Worksheet));
    // Title
    cst.push(Text(
        WPoint {
            wx: wa.wx / 3.,
            wy: -20.,
        },
        "Working sheet".into(),
    ));
    // Arrows
    cst.push(Move(WPoint { wx: 0., wy: -10. }));
    cst.push(Line(WPoint { wx: 100., wy: -10. }));
    cst.push(Line(WPoint { wx: 90., wy: -15. }));
    cst.push(Line(WPoint { wx: 90., wy: -5. }));
    cst.push(Line(WPoint { wx: 100., wy: -10. }));
    cst.push(Move(WPoint { wx: -10., wy: 0. }));
    cst.push(Line(WPoint { wx: -10., wy: 100. }));
    cst.push(Line(WPoint { wx: -15., wy: 90. }));
    cst.push(Line(WPoint { wx: -5., wy: 90. }));
    cst.push(Line(WPoint { wx: -10., wy: 100. }));
    cst.push(Text(WPoint { wx: 40., wy: -20. }, "X".into()));
    cst.push(Text(WPoint { wx: -30., wy: 50. }, "Y".into()));

    // Border
    cst.push(Move(WPoint { wx: 0., wy: 0. }));
    cst.push(Line(WPoint { wx: 0., wy: wa.wy }));
    cst.push(Line(WPoint {
        wx: wa.wx,
        wy: wa.wy,
    }));
    cst.push(Line(WPoint { wx: wa.wx, wy: 0. }));
    cst.push(Line(WPoint { wx: 0., wy: 0. }));

    raw_draw(&pa_ref, &cst);
}

fn draw_grid(pa: Rc<RefCell<PlayingArea>>) {
    use ConstructionType::*;
    let pa_ref = pa.borrow();

    let wa = pa_ref.working_area;
    let w_grid_spacing = pa_ref.working_area_visual_grid;

    let mut cst = Vec::new();
    cst.push(Layer(LayerType::Grid));

    // Vertical grid lines
    let mut wx = 0.;
    while wx <= wa.wx {
        cst.push(Move(WPoint { wx: wx, wy: 0. }));
        cst.push(Line(WPoint { wx: wx, wy: wa.wy }));
        raw_draw(&pa_ref, &cst);
        wx += w_grid_spacing;
    }

    // Horizontal grid lines
    let mut cst = Vec::new();
    let mut wy = 0.;
    while wy <= wa.wy {
        cst.push(Move(WPoint { wx: 0., wy: wy }));
        cst.push(Line(WPoint { wx: wa.wx, wy: wy }));
        raw_draw(&pa_ref, &cst);
        wy += w_grid_spacing;
    }
}

fn draw_content(pa: Rc<RefCell<PlayingArea>>) {
    let pa_ref = pa.borrow();
    //console::log_1(&"ddd".into());
    // Draw all shapes
    for shape_id in pa_ref.pool.get_all_shapes_ids().iter() {
        let shape = pa_ref.pool.get_shape(*shape_id).unwrap();
        raw_draw(&pa_ref, &shape.get_shape_construction(&pa_ref.pool));
        // raw_draw(&pa_ref, &&shape.get_handles_construction());
        // raw_draw(&pa_ref, &shape.get_highlight_construction());
        // raw_draw(&pa_ref, &shape.get_helpers_construction());
    }

    // Draw the current drawing shape
    if let Some(shape_id) = pa_ref.current_drawing_shape_id {
        let shape = pa_ref.pool.get_shape(shape_id).unwrap();
        raw_draw(&pa_ref, &shape.get_shape_construction(&pa_ref.pool));
        // raw_draw(&pa_ref, &&shape.get_handles_construction());
        // raw_draw(&pa_ref, &shape.get_highlight_construction());
        // raw_draw(&pa_ref, &shape.get_helpers_construction());
    }

    // // If using a selection area, draw it
    // // draw_selection(&pa_ref);
}

fn draw_selection(pa: Rc<RefCell<PlayingArea>>) {
    use ConstructionType::*;
    let pa_ref = pa.borrow();

    if let Some(sa) = pa_ref.selection_area {
        let bl = sa[0];
        let tr = sa[1];
        if bl.wx != tr.wx && bl.wy != tr.wy {
            let mut cst = Vec::new();
            cst.push(Layer(LayerType::SelectionTool));
            cst.push(Move(WPoint {
                wx: bl.wx,
                wy: bl.wy,
            }));
            cst.push(Line(WPoint {
                wx: bl.wx,
                wy: tr.wy,
            }));
            cst.push(Line(WPoint {
                wx: tr.wx,
                wy: tr.wy,
            }));
            cst.push(Line(WPoint {
                wx: tr.wx,
                wy: bl.wy,
            }));
            cst.push(Line(WPoint {
                wx: bl.wx,
                wy: bl.wy,
            }));
            raw_draw(&pa_ref, &cst);
        }
    }
}

fn raw_draw(pa_ref: &Ref<'_, PlayingArea>, cst: &Vec<ConstructionType>) {
    let p = Path2d::new().unwrap();
    let scale = pa_ref.global_scale;
    let offset = pa_ref.canvas_offset;

    for prim in cst.iter() {
        use ConstructionType::*;
        match prim {
            Layer(layer_type) => {
                use LayerType::*;
                let (fill_color, color, line_dash, line_width) = match layer_type {
                    Worksheet => (
                        &pa_ref.fill_color,
                        &pa_ref.worksheet_color,
                        &pa_ref.pattern_solid,
                        2.,
                    ),
                    Dimension => (
                        &pa_ref.fill_color,
                        &pa_ref.dimension_color,
                        &pa_ref.pattern_solid,
                        1.,
                    ),
                    GeometryHelpers => (
                        &pa_ref.fill_color,
                        &pa_ref.geohelper_color,
                        &pa_ref.pattern_dashed,
                        1.,
                    ),
                    Origin => (
                        &pa_ref.fill_color,
                        &pa_ref.origin_color,
                        &pa_ref.pattern_solid,
                        1.,
                    ),
                    Grid => (
                        &pa_ref.fill_color,
                        &pa_ref.grid_color,
                        &pa_ref.pattern_solid,
                        1.,
                    ),
                    SelectionTool => (
                        &pa_ref.fill_color,
                        &pa_ref.selection_color,
                        &pa_ref.pattern_dashed,
                        1.,
                    ),
                    Selected => (
                        &pa_ref.fill_color,
                        &pa_ref.selected_color,
                        &pa_ref.pattern_solid,
                        2.,
                    ),
                    Handle(_) => (
                        &pa_ref.fill_color,
                        &pa_ref.worksheet_color,
                        &pa_ref.pattern_solid,
                        1.,
                    ),
                    Highlight => (
                        &pa_ref.highlight_color,
                        &pa_ref.worksheet_color,
                        &pa_ref.pattern_solid,
                        1.,
                    ),
                };
                pa_ref.ctx.set_line_dash(line_dash).unwrap();
                pa_ref.ctx.set_line_width(line_width);
                pa_ref.ctx.set_stroke_style(&color.into());
                pa_ref.ctx.set_fill_style(&fill_color.into());
            }
            Move(w_end) => {
                let c_end = w_end.to_canvas(scale, offset);
                p.move_to(c_end.cx, c_end.cy);
            }
            Line(w_end) => {
                let c_end = w_end.to_canvas(scale, offset);
                p.line_to(c_end.cx, c_end.cy);
            }
            QuadBezier(w_ctrl, w_end) => {
                let c_ctrl = w_ctrl.to_canvas(scale, offset);
                let c_end = w_end.to_canvas(scale, offset);
                p.quadratic_curve_to(c_ctrl.cx, c_ctrl.cy, c_end.cx, c_end.cy);
            }
            CubicBezier(w_ctrl1, w_crtl2, w_end) => {
                let c_ctrl1 = w_ctrl1.to_canvas(scale, offset);
                let c_ctrl2 = w_crtl2.to_canvas(scale, offset);
                let c_end = w_end.to_canvas(scale, offset);
                p.bezier_curve_to(
                    c_ctrl1.cx, c_ctrl1.cy, c_ctrl2.cx, c_ctrl2.cy, c_end.cx, c_end.cy,
                );
            }
            Rectangle(w_start, w_dimensions, fill) => {
                let c_start = w_start.to_canvas(scale, offset);
                let c_dimensions = *w_dimensions * scale;
                if *fill {
                    pa_ref.ctx.fill();
                    pa_ref
                        .ctx
                        .fill_rect(c_start.cx, c_start.cy, c_dimensions.wx, c_dimensions.wy);
                } else {
                    p.rect(c_start.cx, c_start.cy, c_dimensions.wx, c_dimensions.wy);
                }
            }
            Ellipse(w_center, radius, rotation, start_angle, end_angle, fill) => {
                let c_center = w_center.to_canvas(scale, offset);
                if *fill {
                    pa_ref.ctx.begin_path();
                    let _ = pa_ref.ctx.ellipse(
                        c_center.cx,
                        c_center.cy,
                        radius.wx * scale,
                        radius.wy * scale,
                        *rotation,
                        *start_angle,
                        *end_angle,
                    );
                    pa_ref.ctx.fill();
                } else {
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
            }
            Text(w_pos, txt) => {
                let c_pos = w_pos.to_canvas(scale, offset);
                pa_ref.ctx.set_font("20px sans-serif");
                pa_ref.ctx.set_fill_style(&"black".into());
                pa_ref.ctx.fill_text(txt, c_pos.cx, c_pos.cy).unwrap();
            }
        }
    }
    pa_ref.ctx.begin_path(); // Begin a new path
    pa_ref.ctx.stroke_with_path(&p);
}

fn raw_draw_clear_canvas(pa_ref: &Ref<'_, PlayingArea>) {
    pa_ref.ctx.set_stroke_style(&"#F00".into());
    let background_color = &pa_ref.background_color;
    pa_ref.ctx.set_fill_style(&background_color.into());

    pa_ref.ctx.fill();
    let (canvas_width, canvas_height) =
        { (pa_ref.canvas.width() as f64, pa_ref.canvas.height() as f64) };
    pa_ref
        .ctx
        .fill_rect(0., 0., canvas_width as f64, canvas_height as f64);
}
