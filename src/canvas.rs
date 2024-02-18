// #![cfg(not(test))]
// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

use crate::math::*;
use crate::pools::BindingsPool;
use crate::pools::ShapesPool;
use crate::pools::VerticesPool;
use crate::prefab;
use crate::shape::ApiShapes;
use crate::shape::Shapes;
use crate::types::*;

use kurbo::{BezPath, PathEl, Point};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    CanvasRenderingContext2d, Document, Element, Event, FileList, FileReader, HtmlCanvasElement,
    HtmlElement, HtmlInputElement, KeyboardEvent, MouseEvent, WheelEvent, Window,
};

pub type RefPA = Rc<RefCell<PlayingArea>>;
pub type ElementCallback = Box<dyn Fn(RefPA, Event) + 'static>;

#[derive(Debug, Copy, Clone)]
#[repr(u16)]
enum JSMouseState {
    JSNoButton = 0,
    JSLeftDown = 1,
    #[allow(dead_code)]
    JSRightDown = 2,
    #[allow(dead_code)]
    JSMiddleDown = 4,
}
#[derive(Debug, Copy, Clone)]
enum MouseState {
    NoButton,
    LeftDown,
    LeftDownMoved,
    RightDown,
    RightDownMoved,
    MiddleDown,
    MiddleDownMoved,
}

#[derive(Default)]
struct KeysStates {
    crtl_pressed: bool,
    shift_pressed: bool,
}

pub struct PlayingArea {
    v_pool: VerticesPool,
    sh_pool: ShapesPool,
    b_pool: BindingsPool,
    geos_selected: Vec<Geobjects>,
    geo_highlighted: Option<Geobjects>,
    geo_on_construction: Option<Geobjects>,
    //
    draw_vertex: Option<VertexId>,
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
    draw_styles: DrawStyles,

    // Mouse position on worksheet
    mouse_worksheet_position: HtmlElement,
    _viewgrid_element: HtmlElement,
    _snapgrid_element: HtmlElement,

    pos: Point,
    pos_dwn: Point,
    canvas_mouse_pos_ms_dwn: Point,
    magnet_distance: f64,
    grab_handle_precision: f64,
    size_handle: f64,

    icon_selected: &'static str,
    selection_area: Option<[Point; 2]>,
    keys_states: KeysStates,
    //
    mouse_state: MouseState,

    working_area: Point,
    global_scale: f64,
    canvas_offset: Point,
    canvas_offset_ms_dwn: Point,
    working_area_visual_grid: f64,
    working_area_snap_grid: f64,
    // Playing area static draw
    // grid: ShapesPool,
}
impl PlayingArea {
    pub fn add_vertex(&mut self, pos: &Point) -> Vertex {
        self.v_pool.add(*pos)
    }
    pub fn move_vertex(&mut self, v_id: &VertexId, dpos: &Point) {
        let v = self.v_pool.get_mut(&v_id).unwrap();
        v.pt = v.saved_pt + (dpos.x, dpos.y);
    }
    pub fn end_move_vertex(&mut self, v_id: &VertexId) {
        let v = self.v_pool.get_mut(&v_id).unwrap();
        v.saved_pt = v.pt;
    }
    pub fn remove_vertex(&mut self, v_id: &VertexId) {
        self.v_pool.remove(v_id);
    }
    // pub fn get_vs_selected(&mut self) {
    //     self.vs_selected = vec![];
    //     for v in self.v_pool.values() {
    //         if v.selected {
    //             self.vs_selected.push(v.id);
    //         }
    //     }
    // }
    // pub fn get_ss_selected(&mut self) {
    //     self.shs_selected = vec![];
    //     for s in self.sh_pool.values() {
    //         if s.is_selected() {
    //             self.shs_selected.push(s.get_id());
    //         }
    //     }
    // }
    pub fn clear_shapes_selections(&mut self) {
        for s in self.sh_pool.values_mut() {
            s.set_selected(false);
            let vs = s.get_vertices_ids();
            for v_id in vs.iter() {
                let v = self.v_pool.get_mut(v_id).unwrap();
                v.selected = false;
            }
        }
    }
    fn no_geobject_under_pos(&self) -> bool {
        let mut nothing = true;
        for v in self.v_pool.values() {
            if v.is_near_pos(&self.pos, self.grab_handle_precision) {
                nothing = false;
                break;
            }
        }
        if nothing {
            for sh in self.sh_pool.values() {
                if sh.is_near_pos(&self.pos, self.grab_handle_precision, &self.v_pool) {
                    nothing = false;
                    break;
                }
            }
        }
        nothing
    }
    pub fn geobject_under_pos(&mut self) -> Option<Geobjects> {
        if self.no_geobject_under_pos() {
            return None;
        }
        for sh in self.sh_pool.values() {
            if sh.is_near_pos(&self.pos, self.grab_handle_precision, &self.v_pool) {
                return Some(Geobjects::Shape(sh.get_id()));
            }
        }
        for v in self.v_pool.values_mut() {
            if v.is_near_pos(&self.pos, self.grab_handle_precision) {
                return Some(Geobjects::Vertex(v.id));
            }
        }
        None
    }
    pub fn highlight_geobject_under_pos(&mut self) {
        self.geo_highlighted = self.geobject_under_pos();
    }
    pub fn toggle_geobject_selection(&mut self, geo: &Geobjects) {
        match geo {
            Geobjects::Vertex(v_id) => {
                let v = self.v_pool.get_mut(v_id).unwrap();
                v.selected = !v.selected;
            }
            Geobjects::Shape(sh_id) => {
                let sh = self.sh_pool.get_mut(sh_id).unwrap();
                sh.set_selected(!sh.is_selected());
            }
        }
    }
    pub fn move_geobject_construction(&mut self, dpos: &Point) {
        if let Some(geo) = self.geo_on_construction.clone() {
            match geo {
                Geobjects::Vertex(v_id) => {
                    self.move_vertex(&v_id, dpos);
                }
                Geobjects::Shape(sh_id) => {
                    self.move_vertex(
                        &self.sh_pool.get(&sh_id).unwrap().get_vextex_construction(),
                        dpos,
                    );
                }
            }
        };
    }
    pub fn end_move_geobject_construction(&mut self) {
        if let Some(geo) = self.geo_on_construction.clone() {
            match geo {
                Geobjects::Vertex(v_id) => {
                    self.end_move_vertex(&v_id);
                }
                Geobjects::Shape(sh_id) => {
                    self.end_move_vertex(
                        &self.sh_pool.get(&sh_id).unwrap().get_vextex_construction(),
                    );
                }
            }
        };
    }

    pub fn pos_magnet_to_vertex(&mut self, v_id_excl: &VertexId) {
        for (v_id, v) in self.v_pool.iter() {
            if v_id != v_id_excl {
                if self.pos.distance(v.pt) < self.magnet_distance {
                    self.pos = v.pt;
                    break;
                }
            }
        }
    }
    pub fn get_geobject_construction_pattern(&self, geo: Geobjects) -> ConstructionPattern {
        if let Some(geo_cst) = self.geo_on_construction.clone() {
            match geo_cst {
                Geobjects::Vertex(v_id_cst) => match geo {
                    Geobjects::Vertex(v_id) => {
                        if v_id == v_id_cst {
                            return ConstructionPattern::OnConstruction;
                        }
                    }
                    Geobjects::Shape(_) => (),
                },
                Geobjects::Shape(sh_id_cst) => match geo {
                    Geobjects::Vertex(_) => (),
                    Geobjects::Shape(sh_id) => {
                        if sh_id == sh_id_cst {
                            return ConstructionPattern::OnConstruction;
                        }
                    }
                },
            }
        }
        let cst_ptrn = if let Some(geo_high) = self.geo_highlighted.clone() {
            match geo_high {
                Geobjects::Vertex(v_id_high) => match geo {
                    Geobjects::Vertex(v_id) => {
                        if v_id == v_id_high {
                            ConstructionPattern::Highlighted
                        } else {
                            ConstructionPattern::Normal
                        }
                    }
                    Geobjects::Shape(_) => ConstructionPattern::Normal,
                },
                Geobjects::Shape(sh_id_high) => match geo {
                    Geobjects::Vertex(_) => ConstructionPattern::Normal,
                    Geobjects::Shape(sh_id) => {
                        if sh_id == sh_id_high {
                            ConstructionPattern::Highlighted
                        } else {
                            ConstructionPattern::Normal
                        }
                    }
                },
            }
        } else {
            ConstructionPattern::Normal
        };

        match geo {
            Geobjects::Vertex(v_id) => {
                if self.v_pool.get(&v_id).unwrap().selected {
                    ConstructionPattern::Selected
                } else {
                    cst_ptrn
                }
            }
            Geobjects::Shape(sh_id) => {
                if self.sh_pool.get(&sh_id).unwrap().is_selected() {
                    ConstructionPattern::Selected
                } else {
                    cst_ptrn
                }
            }
        }
    }
}
///////////////
// Initialization
pub fn create_playing_area(window: Window) -> Result<(), JsValue> {
    log!("Creating playing area");
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
    user_icons.insert("icon-scissors", None);
    user_icons.insert("icon-text-fix", None);
    user_icons.insert("icon-cog", None);

    let document_element = document
        .document_element()
        .ok_or("should have a document element")?;
    let styles = window
        .get_computed_style(&document_element)
        .unwrap()
        .unwrap();

    // Calculation starting parameters
    let (canvas_width, canvas_height) = { (canvas.width() as f64, canvas.height() as f64) };
    // let head_position = WXY { wx: 10., wy: 10. };
    let working_area = Point::new(500., 500.);
    settings_width_input.set_value(&working_area.x.to_string());
    settings_height_input.set_value(&working_area.y.to_string());

    let working_area_visual_grid = 10.;
    let working_area_snap_grid = 1.;

    let canvas_offset = Point {
        x: (canvas_width - working_area.x) / 2.,
        y: (canvas_height - working_area.y) / 2.,
    };
    let global_scale = 1.0;

    let v_pool = VerticesPool::new();
    let s_pool = ShapesPool::new();
    let b_pool = BindingsPool::new();

    let playing_area = Rc::new(RefCell::new(PlayingArea {
        v_pool,
        sh_pool: s_pool,
        b_pool,
        geos_selected: vec![],
        geo_highlighted: None,
        geo_on_construction: None,
        draw_vertex: None,
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
        draw_styles: DrawStyles::build(styles)?,
        mouse_worksheet_position,
        _viewgrid_element: viewgrid_element,
        _snapgrid_element: snapgrid_element,

        magnet_distance: 5.,
        grab_handle_precision: 2.5,
        size_handle: 10.,

        pos: Point::default(),
        pos_dwn: Point::default(),

        icon_selected: "icon-arrow",
        selection_area: None,
        keys_states: KeysStates::default(),
        mouse_state: MouseState::NoButton,

        canvas_mouse_pos_ms_dwn: Point::default(),

        // Real word dimensions
        working_area,

        // Zoom
        global_scale,
        canvas_offset,
        canvas_offset_ms_dwn: Point::default(),
        working_area_visual_grid,
        working_area_snap_grid,
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

    Ok(())
}
fn init_window(pa: RefPA) -> Result<(), JsValue> {
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
fn init_settings_panel(pa: RefPA) -> Result<(), JsValue> {
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
fn init_icons(pa: RefPA) -> Result<(), JsValue> {
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
fn init_context_menu(pa: RefPA) -> Result<(), JsValue> {
    let pa_ref = pa.borrow_mut();
    let document = pa_ref.document.clone();
    let ctx_menu_bind_vertex_to = document
        .get_element_by_id("ctx-menu-bind-vertex-to")
        .unwrap();
    set_callback(
        pa.clone(),
        "click".into(),
        &ctx_menu_bind_vertex_to,
        Box::new(on_context_menu_bind_vertex_to),
    )?;
    let action_group = document.get_element_by_id("action-group").unwrap();
    set_callback(
        pa.clone(),
        "click".into(),
        &action_group,
        Box::new(on_context_menu_group_click),
    )?;
    let delete_group = document.get_element_by_id("action-delete").unwrap();
    set_callback(
        pa.clone(),
        "click".into(),
        &delete_group,
        Box::new(on_context_menu_delete_click),
    )?;
    Ok(())
}
fn init_canvas(pa: RefPA) -> Result<(), JsValue> {
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
fn init_menu(pa: RefPA) -> Result<(), JsValue> {
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
    }) as Box<dyn FnMut()>);

    load_element.add_event_listener_with_callback("click", on_load.as_ref().unchecked_ref())?;
    on_load.forget(); // Leaks memory, but we need to do this to keep the callback alive

    save_element.add_event_listener_with_callback("click", on_save.as_ref().unchecked_ref())?;
    on_save.forget(); // Leaks memory, but we need to do this to keep the callback alive

    drop(pa_mut);
    // Set up an event listener to handle file selection
    let on_file_select = Closure::wrap(Box::new(move || {
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
                        convert_svg_to_shapes(pa_clone.clone(), content);
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
fn init_status(pa: RefPA) -> Result<(), JsValue> {
    let pa_ref = pa.borrow_mut();
    let _document = pa_ref.document.clone();

    Ok(())
}
fn set_callback(
    pa: RefPA,
    event_str: String,
    element: &Element,
    callback: ElementCallback,
) -> Result<(), JsValue> {
    let event_str_cloned = event_str.clone();
    let callback = Box::new(move |pa: RefPA, e: Event| {
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
fn convert_svg_to_shapes(pa: RefPA, _svg_data: String) {
    let mut pa_mut = pa.borrow_mut();
    // let grp_id = pa_mut.data_pools.create_group_id();
    pa_mut.clear_shapes_selections();

    // for event in svg::parser::Parser::new(&svg_data).into_iter() {
    //     match event {
    //         svg::parser::Event::Tag(svg::node::element::tag::Path, _, attributes) => {
    //             let data = attributes.get("d").unwrap();
    //             let data = svg::node::element::path::Data::parse(data).unwrap();
    //             let mut current_position = Point::default();
    //             let mut start_position = Point::default();
    //             let mut last_quad_control_point: Option<Point> = None;
    //             let mut last_cubic_control_point: Option<Point> = None;
    //             for command in data.iter() {
    //                 let command_clone = command.clone();
    //                 use svg::node::element::path::*;
    //                 match command_clone {
    //                     Command::Move(postype, params) => {
    //                         if params.len() == 2 {
    //                             current_position = match postype {
    //                                 Position::Absolute => Point {
    //                                     x: params[0] as f64,
    //                                     y: params[1] as f64,
    //                                 },
    //                                 Position::Relative => Point {
    //                                     x: params[0] as f64 + current_position.x,
    //                                     y: params[1] as f64 + current_position.y,
    //                                 },
    //                             };
    //                             start_position = current_position;
    //                             last_quad_control_point = None;
    //                             last_cubic_control_point = None;
    //                         }
    //                     }
    //                     _ => (), // Command::Line(postype, params) => {
    //                              //     if params.len() % 2 == 0 {
    //                              //         let nb_curves = params.len() / 2;
    //                              //         for curve in 0..nb_curves {
    //                              //             let end_point = Point {
    //                              //                 wx: params[2 * curve] as f64,
    //                              //                 wy: params[2 * curve + 1] as f64,
    //                              //             };
    //                              //             let new_position = match postype {
    //                              //                 Position::Absolute => end_point,
    //                              //                 Position::Relative => current_position + end_point,
    //                              //             };
    //                              //             if let Some(shape) = Line::new(&current_position, &new_position)
    //                              //             {
    //                              //                 let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
    //                              //                 pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //                 pa_mut.data_pools.set_shape_group(&grp_id, &sh_id);
    //                              //             }

    //                              //             current_position = new_position;
    //                              //             last_quad_control_point = None;
    //                              //             last_cubic_control_point = None;
    //                              //         }
    //                              //     }
    //                              // }
    //                              // Command::HorizontalLine(postype, params) => {
    //                              //     for curve in 0..params.len() {
    //                              //         let end_point = Point {
    //                              //             wx: params[curve] as f64,
    //                              //             wy: current_position.y,
    //                              //         };
    //                              //         let new_position = match postype {
    //                              //             Position::Absolute => end_point,
    //                              //             Position::Relative => current_position + end_point,
    //                              //         };
    //                              //         if let Some(shape) = Line::new(&current_position, &new_position) {
    //                              //             let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
    //                              //             pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //             pa_mut.data_pools.set_shape_group(&grp_id, &sh_id);
    //                              //         }

    //                              //         current_position = new_position;
    //                              //         last_quad_control_point = None;
    //                              //         last_cubic_control_point = None;
    //                              //     }
    //                              // }
    //                              // Command::VerticalLine(postype, params) => {
    //                              //     for curve in 0..params.len() {
    //                              //         let end_point = Point {
    //                              //             wx: current_position.x,
    //                              //             wy: params[curve] as f64,
    //                              //         };
    //                              //         let new_position = match postype {
    //                              //             Position::Absolute => end_point,
    //                              //             Position::Relative => current_position + end_point,
    //                              //         };
    //                              //         if let Some(shape) = Line::new(&current_position, &new_position) {
    //                              //             let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
    //                              //             pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //             pa_mut.data_pools.set_shape_group(&grp_id, &sh_id);
    //                              //         }

    //                              //         current_position = new_position;
    //                              //         last_quad_control_point = None;
    //                              //         last_cubic_control_point = None;
    //                              //     }
    //                              // }
    //                              // Command::QuadraticCurve(postype, params) => {
    //                              //     if params.len() % 4 == 0 {
    //                              //         let nb_curves = params.len() / 4;
    //                              //         for curve in 0..nb_curves {
    //                              //             let mut control_point = Point {
    //                              //                 wx: params[4 * curve] as f64,
    //                              //                 wy: params[4 * curve + 1] as f64,
    //                              //             };
    //                              //             let end_point = Point {
    //                              //                 wx: params[4 * curve + 2] as f64,
    //                              //                 wy: params[4 * curve + 3] as f64,
    //                              //             };
    //                              //             let new_position = match postype {
    //                              //                 Position::Absolute => end_point,
    //                              //                 Position::Relative => {
    //                              //                     control_point += current_position;
    //                              //                     current_position + end_point
    //                              //                 }
    //                              //             };
    //                              //             if let Some(shape) = QuadBezier::new(
    //                              //                 &current_position,
    //                              //                 &control_point,
    //                              //                 &new_position,
    //                              //             ) {
    //                              //                 let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
    //                              //                 pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //                 pa_mut.data_pools.set_shape_group(&grp_id, &sh_id);
    //                              //             }

    //                              //             current_position = new_position;
    //                              //             last_quad_control_point = Some(control_point);
    //                              //             last_cubic_control_point = None;
    //                              //         }
    //                              //     }
    //                              // }
    //                              // Command::SmoothQuadraticCurve(postype, params) => {
    //                              //     if params.len() % 2 == 0 {
    //                              //         let nb_curves = params.len() / 2;
    //                              //         for curve in 0..nb_curves {
    //                              //             let control_point =
    //                              //                 if let Some(last_ctrl_pt) = last_quad_control_point {
    //                              //                     current_position + (current_position - last_ctrl_pt)
    //                              //                 } else {
    //                              //                     current_position
    //                              //                 };
    //                              //             let end_point = Point {
    //                              //                 wx: params[2 * curve] as f64,
    //                              //                 wy: params[2 * curve + 1] as f64,
    //                              //             };
    //                              //             let new_position = match postype {
    //                              //                 Position::Absolute => end_point,
    //                              //                 Position::Relative => current_position + end_point,
    //                              //             };
    //                              //             if let Some(shape) = QuadBezier::new(
    //                              //                 &current_position,
    //                              //                 &control_point,
    //                              //                 &new_position,
    //                              //             ) {
    //                              //                 let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
    //                              //                 pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //                 pa_mut.data_pools.set_shape_group(&grp_id, &sh_id);
    //                              //             }

    //                              //             current_position = new_position;
    //                              //             last_quad_control_point = Some(control_point);
    //                              //             last_cubic_control_point = None;
    //                              //         }
    //                              //     }
    //                              // }
    //                              // Command::CubicCurve(postype, params) => {
    //                              //     if params.len() % 6 == 0 {
    //                              //         let nb_curves = params.len() / 6;
    //                              //         for curve in 0..nb_curves {
    //                              //             let mut control_point1 = Point {
    //                              //                 wx: params[6 * curve] as f64,
    //                              //                 wy: params[6 * curve + 1] as f64,
    //                              //             };
    //                              //             let mut control_point2 = Point {
    //                              //                 wx: params[6 * curve + 2] as f64,
    //                              //                 wy: params[6 * curve + 3] as f64,
    //                              //             };
    //                              //             let end_point = Point {
    //                              //                 wx: params[6 * curve + 4] as f64,
    //                              //                 wy: params[6 * curve + 5] as f64,
    //                              //             };
    //                              //             let new_position = match postype {
    //                              //                 Position::Absolute => end_point,
    //                              //                 Position::Relative => {
    //                              //                     control_point1 += current_position;
    //                              //                     control_point2 += current_position;
    //                              //                     current_position + end_point
    //                              //                 }
    //                              //             };
    //                              //             if let Some(shape) = CubicBezier::new(
    //                              //                 &current_position,
    //                              //                 &control_point1,
    //                              //                 &control_point2,
    //                              //                 &new_position,
    //                              //             ) {
    //                              //                 let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
    //                              //                 pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //                 pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //                 pa_mut.data_pools.set_shape_group(&grp_id, &sh_id);
    //                              //             }
    //                              //             current_position = new_position;
    //                              //             last_quad_control_point = None;
    //                              //             last_cubic_control_point = Some(control_point2);
    //                              //         }
    //                              //     }
    //                              // }
    //                              // Command::SmoothCubicCurve(postype, params) => {
    //                              //     if params.len() % 4 == 0 {
    //                              //         let nb_curves = params.len() / 4;
    //                              //         for curve in 0..nb_curves {
    //                              //             let control_point1 =
    //                              //                 if let Some(last_ctrl_pt) = last_cubic_control_point {
    //                              //                     current_position + (current_position - last_ctrl_pt)
    //                              //                 } else {
    //                              //                     current_position
    //                              //                 };
    //                              //             let mut control_point2 = Point {
    //                              //                 wx: params[4 * curve] as f64,
    //                              //                 wy: params[4 * curve + 1] as f64,
    //                              //             };
    //                              //             let end_point = Point {
    //                              //                 wx: params[4 * curve + 2] as f64,
    //                              //                 wy: params[4 * curve + 3] as f64,
    //                              //             };
    //                              //             let new_position = match postype {
    //                              //                 Position::Absolute => end_point,
    //                              //                 Position::Relative => {
    //                              //                     control_point2 += current_position;
    //                              //                     current_position + end_point
    //                              //                 }
    //                              //             };
    //                              //             if let Some(shape) = CubicBezier::new(
    //                              //                 &current_position,
    //                              //                 &control_point1,
    //                              //                 &control_point2,
    //                              //                 &new_position,
    //                              //             ) {
    //                              //                 let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
    //                              //                 pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //                 pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //                 pa_mut.data_pools.set_shape_group(&grp_id, &sh_id);
    //                              //             }

    //                              //             current_position = new_position;
    //                              //             last_quad_control_point = None;
    //                              //             last_cubic_control_point = Some(control_point2);
    //                              //         }
    //                              //     }
    //                              // }
    //                              // Command::EllipticalArc(_postype, _params) => {}
    //                              // Command::Close => {
    //                              //     if let Some(shape) = Line::new(&current_position, &start_position) {
    //                              //         let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
    //                              //         pa_mut.data_pools.set_shape_selected(&sh_id, true);
    //                              //         pa_mut.data_pools.set_shape_group(&grp_id, &sh_id);
    //                              //     }

    //                              //     current_position = start_position;
    //                              //     last_quad_control_point = None;
    //                              //     last_cubic_control_point = None;
    //                              // }
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    // }
}

///////////////
// Canvas events: mouse, keyboard and context menu
fn on_mouse_down(pa: RefPA, event: Event) {
    let mut pa_mut = pa.borrow_mut();
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        if mouse_event.buttons() == JSMouseState::JSLeftDown as u16 {
            if let Some(context_menu) = pa_mut.document.get_element_by_id("contextMenu") {
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

            // Get mouse position relative to the canvas
            let rect = pa_mut.canvas.get_bounding_client_rect();
            let canvas_mouse_pos = Point {
                x: mouse_event.client_x() as f64 - rect.left(),
                y: mouse_event.client_y() as f64 - rect.top(),
            };

            // Save canvas offset for move
            pa_mut.canvas_offset_ms_dwn = pa_mut.canvas_offset;
            pa_mut.canvas_mouse_pos_ms_dwn = canvas_mouse_pos;

            pa_mut.pos = to_world(
                &canvas_mouse_pos,
                pa_mut.global_scale,
                &pa_mut.canvas_offset,
            );
            pa_mut.pos_dwn = pa_mut.pos;

            // Remove the pick point if any
            if let Some(v_id) = pa_mut.draw_vertex {
                pa_mut.remove_vertex(&v_id);
                pa_mut.draw_vertex = None;
            }

            match pa_mut.icon_selected {
                "icon-arrow" => {}
                "icon-selection" => pa_mut.selection_area = Some([pa_mut.pos, pa_mut.pos]),
                "icon-line" => {
                    pa_mut.clear_shapes_selections();
                    let pos = pa_mut.pos;
                    let snap_grid = pa_mut.working_area_snap_grid;
                    let va = pa_mut.add_vertex(&pos);
                    let vb = pa_mut.add_vertex(&(pos + (snap_grid, snap_grid)));
                    let sh_id = pa_mut.sh_pool.add_line(&va, &vb).get_id();
                    // The shape is in construction state, special case
                    pa_mut.geo_on_construction = Some(Geobjects::Shape(sh_id));
                }
                // "icon-quadbezier" => {
                //     pa_mut.data_pools.clear_shapes_selection();
                //     if let Some(shape) = QuadBezier::new(
                //         &pick_pos,
                //         &(pick_pos + snap_grid),
                //         &(pick_pos + 2. * snap_grid),
                //     ) {
                //         let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
                //         pa_mut.data_pools.set_shape_selected(&sh_id, true);
                //     }
                // }
                // "icon-cubicbezier" => {
                //     pa_mut.data_pools.clear_shapes_selection();
                //     if let Some(shape) = CubicBezier::new(
                //         &pick_pos,
                //         &(pick_pos + snap_grid),
                //         &(pick_pos + 2. * snap_grid),
                //         &(pick_pos + 3. * snap_grid),
                //     ) {
                //         let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
                //         pa_mut.data_pools.set_shape_selected(&sh_id, true);
                //     }
                // }
                "icon-rectangle" => {
                    pa_mut.clear_shapes_selections();
                    //
                }
                // "icon-ellipse" => {
                //     pa_mut.data_pools.clear_shapes_selection();
                //     let shape = EllipticArc::new(&pick_pos, &pick_pos, 0., 2. * PI, snap_grid);
                //     let sh_id = pa_mut.data_pools.insert_shape(Box::new(shape));
                //     pa_mut.data_pools.set_shape_selected(&sh_id, true);
                // }
                "icon-scissors" => {
                    pa_mut.clear_shapes_selections();
                    pa_mut.geobject_under_pos();
                    // if let Some((sh_id, bs_id)) = pa_mut
                    //     .s_pool
                    //     .get_shape_under_pos(&pick_pos, grab_handle_precision)
                    // {
                    //     log!("Picked some shape id: {:?}", sh_id);
                    //     // pa_mut
                    //     //     .data_pools
                    //     //     .cut_shape(&sh_id, &pick_pos, grab_handle_precision);
                    // }
                }
                _ => (),
            }
            // Update display mouse world position
            pa_mut
                .mouse_worksheet_position
                .set_text_content(Some(&format!(
                    "( {:?} , {:?} ) - ( {:?} , {:?} )",
                    pa_mut.pos.x.round() as i32,
                    pa_mut.pos.y.round() as i32,
                    (pa_mut.pos.x - pa_mut.pos_dwn.x).round() as i32,
                    (pa_mut.pos.y - pa_mut.pos_dwn.y).round() as i32
                )));

            pa_mut.mouse_state = MouseState::LeftDown;
        }
    }
    drop(pa_mut);
    render(pa.clone());
}
fn on_mouse_move(pa: RefPA, event: Event) {
    let mut pa_mut = pa.borrow_mut();
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        // let mouse_state = pa_mut.mouse_state.clone();

        // Get mouse position relative to the canvas
        let rect = pa_mut.canvas.get_bounding_client_rect();
        let canvas_mouse_pos = Point {
            x: mouse_event.client_x() as f64 - rect.left(),
            y: mouse_event.client_y() as f64 - rect.top(),
        };

        pa_mut.pos = to_world(
            &canvas_mouse_pos,
            pa_mut.global_scale,
            &pa_mut.canvas_offset,
        );

        let delta_pick_pos = pa_mut.pos - (pa_mut.pos_dwn.x, pa_mut.pos_dwn.y);

        // log!(
        //     "delta_pick_pos: ({:.2},{:.2})",
        //     delta_pick_pos.x,
        //     delta_pick_pos.y
        // );
        //pick_delta_pos.snap(snap_grid);
        pa_mut.highlight_geobject_under_pos();

        match pa_mut.mouse_state {
            MouseState::LeftDown | MouseState::LeftDownMoved => {
                match pa_mut.icon_selected {
                    "icon-arrow" => {
                        // Move Canvas if no selection
                        pa_mut.canvas_offset = (canvas_mouse_pos
                            - (
                                pa_mut.canvas_mouse_pos_ms_dwn.x,
                                pa_mut.canvas_mouse_pos_ms_dwn.y,
                            ))
                            + (pa_mut.canvas_offset_ms_dwn.x, pa_mut.canvas_offset_ms_dwn.y);
                    }
                    "icon-selection" => {
                        if let Some(sa) = pa_mut.selection_area.as_mut() {
                            sa[1] = delta_pick_pos
                        }
                    }
                    "icon-line" | "icon-quadbezier" | "icon-cubicbezier" | "icon-ellipse"
                    | "icon-rectangle" => {
                        if let Some(_) = pa_mut.geo_on_construction {
                            pa_mut.move_geobject_construction(&delta_pick_pos);
                        }
                    }
                    _ => (),
                }
                pa_mut.mouse_state = MouseState::LeftDownMoved;
            }
            _ => {
                if let Some(v_id) = pa_mut.draw_vertex {
                    pa_mut.pos_magnet_to_vertex(&v_id);
                    let pos = pa_mut.pos;
                    let v = pa_mut.v_pool.get_mut(&v_id).unwrap();
                    v.pt = pos;
                }
            }
        }

        // Display: update mouse world position
        if let MouseState::LeftDownMoved = pa_mut.mouse_state.clone() {
            pa_mut
                .mouse_worksheet_position
                .set_text_content(Some(&format!(
                    "( {:?} , {:?} ) - ( {:?} , {:?} )",
                    delta_pick_pos.x.round() as i32,
                    delta_pick_pos.y.round() as i32,
                    (delta_pick_pos.x - pa_mut.pos_dwn.x).round() as i32,
                    (delta_pick_pos.y - pa_mut.pos_dwn.y).round() as i32
                )));
        } else {
            pa_mut
                .mouse_worksheet_position
                .set_text_content(Some(&format!(
                    "( {:?} , {:?} )",
                    delta_pick_pos.x.round() as i32,
                    delta_pick_pos.y.round() as i32
                )));
        }
    }
    drop(pa_mut);
    render(pa.clone());
}
fn on_mouse_up(pa: RefPA, event: Event) {
    let mut pa_mut = pa.borrow_mut();
    if let Ok(_mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        match pa_mut.icon_selected {
            "icon-selection" => {
                let selection_area = pa_mut.selection_area.clone();
                if let Some(sa_raw) = selection_area {
                    let mut bb_outer = sa_raw;
                    reorder_corners(&mut bb_outer);
                    pa_mut.sh_pool.select_shapes_bounded_by_rectangle(bb_outer);
                }
                pa_mut.selection_area = None;
            }
            "icon-arrow" => {
                if let Some(geo) = pa_mut.geo_highlighted.clone() {
                    // If simple click
                    if let MouseState::LeftDown = pa_mut.mouse_state {
                        pa_mut.toggle_geobject_selection(&geo);
                    }
                }
            }
            "icon-line" | "icon-quadbezier" | "icon-cubicbezier" | "icon-ellipse"
            | "icon-rectangle" => {
                if let Some(_) = pa_mut.geo_on_construction {
                    pa_mut.end_move_geobject_construction();
                    // No more construction shape to process
                    pa_mut.geo_on_construction = None
                }
            }
            _ => (),
        }
        go_to_arrow_tool(&mut pa_mut);
    }
    pa_mut.mouse_state = MouseState::NoButton;
    drop(pa_mut);
    render(pa.clone());
}
fn on_mouse_wheel(pa: RefPA, event: Event) {
    if let Ok(wheel_event) = event.dyn_into::<WheelEvent>() {
        wheel_event.prevent_default();
        let mut pa_ref = pa.borrow_mut();
        let zoom_factor = 0.05;

        let old_scale = pa_ref.global_scale;

        // Get mouse position relative to the canvas
        let rect = pa_ref.canvas.get_bounding_client_rect();
        let canvas_mouse_pos = Point {
            x: wheel_event.client_x() as f64 - rect.left(),
            y: wheel_event.client_y() as f64 - rect.top(),
        };

        // Determine the new scale
        let new_scale = if wheel_event.delta_y() < 0. {
            // Zoom in
            (old_scale * (1.0 + zoom_factor)).min(10.0)
        } else {
            // Zoom out
            (old_scale / (1.0 + zoom_factor)).max(0.2)
        };

        let new_canvas_offset_x = pa_ref.canvas_offset.x
            - (new_scale - old_scale) * (canvas_mouse_pos.x - pa_ref.canvas_offset.x) / old_scale;
        let new_canvas_offset_y = pa_ref.canvas_offset.y
            - (new_scale - old_scale) * (canvas_mouse_pos.y - pa_ref.canvas_offset.y) / old_scale;

        pa_ref.canvas_offset = Point {
            x: new_canvas_offset_x,
            y: new_canvas_offset_y,
        };
        pa_ref.global_scale = new_scale;
        drop(pa_ref);
        render(pa);
    }
}
fn on_mouse_enter(pa: RefPA, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.mouse_state = MouseState::NoButton;
}
fn on_mouse_leave(pa: RefPA, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    pa_ref.mouse_state = MouseState::NoButton;
}
fn on_keydown(pa: RefPA, event: Event) {
    if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
        let mut pa_mut = pa.borrow_mut();

        if keyboard_event.key() == "Delete" || keyboard_event.key() == "Backspace" {
            pa_mut.sh_pool.delete_selected_shapes();
        }
        // if keyboard_event.key() == "Escape" {
        //     console::log_1(&"ddd".into());
        //     if pa_mut.icon_selected == "icon-line"
        //         || pa_mut.icon_selected == "icon-quadbezier"
        //         || pa_mut.icon_selected == "icon-cubicbezier"
        //         || pa_mut.icon_selected == "icon-rectangle"
        //         || pa_mut.icon_selected == "icon-ellipse"
        //     {
        //         console::log_1(&"eee".into());
        //         if let Some(shape_id) = pa_mut.cur_draw_item {
        //             pa_mut.pool.delete_shape(&shape_id);
        //         }
        //         deselect_icons(&pa_mut);
        //         select_icon(&pa_mut, &"icon-arrow");
        //         pa_mut.icon_selected = "icon-arrow";
        //         pa_mut.show_pick_point = false;
        //     }
        //     pa_mut.cur_sel_shapes_ids.clear();
        // }
        if keyboard_event.key() == "Control" || keyboard_event.key() == "Meta" {
            pa_mut.keys_states.crtl_pressed = true;
        }
        if keyboard_event.key() == "Shift" {
            pa_mut.keys_states.shift_pressed = true;
        }
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
        drop(pa_mut);
        render(pa.clone());
    }
}
fn on_keyup(pa: RefPA, event: Event) {
    if let Ok(keyboard_event) = event.dyn_into::<KeyboardEvent>() {
        let mut pa_mut = pa.borrow_mut();
        if keyboard_event.key() == "Control" || keyboard_event.key() == "Meta" {
            pa_mut.keys_states.crtl_pressed = false;
        }
        if keyboard_event.key() == "Shift" {
            pa_mut.keys_states.shift_pressed = false;
        }
    }
}
fn on_context_menu(pa: RefPA, event: Event) {
    let pa_ref = pa.borrow();
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
fn on_context_menu_bind_vertex_to(pa: RefPA, _event: Event) {
    let mut pa_ref = pa.borrow_mut();
    if let Some(context_menu) = pa_ref.document.get_element_by_id("contextMenu") {
        if let Some(html_element) =
            wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlElement>(&context_menu)
        {
            // Update the list of vertex selected
            //pa_ref.get_vs_selected();

            // Hide the context afer click
            html_element
                .style()
                .set_property("display", "none")
                .unwrap();
        }
    }
}
fn on_context_menu_group_click(pa: RefPA, _event: Event) {
    let pa_ref = pa.borrow_mut();
    if let Some(context_menu) = pa_ref.document.get_element_by_id("contextMenu") {
        if let Some(html_element) =
            wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlElement>(&context_menu)
        {
            // Hide the context afer click
            html_element
                .style()
                .set_property("display", "none")
                .unwrap();
        }
    }
}
fn on_context_menu_delete_click(pa: RefPA, _event: Event) {
    let mut pa_mut = pa.borrow_mut();
    if let Some(context_menu) = pa_mut.document.get_element_by_id("contextMenu") {
        if let Some(html_element) =
            wasm_bindgen::JsCast::dyn_ref::<web_sys::HtmlElement>(&context_menu)
        {
            // Hide the context afer click
            html_element
                .style()
                .set_property("display", "none")
                .unwrap();
            pa_mut.clear_shapes_selections();
            drop(pa_mut);
            render(pa.clone());
        }
    }
}

///////////////
/// Settings panel events
fn on_apply_settings_click(pa: RefPA, _event: Event) {
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

    pa_ref.working_area = Point {
        x: width,
        y: height,
    };

    drop(pa_ref);
    resize_area(pa.clone());
    render(pa.clone());
}
fn on_modal_backdrop_click(pa: RefPA, _event: Event) {
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
        .set_value(&pa_ref.working_area.x.to_string());
    pa_ref
        .settings_height_input
        .set_value(&pa_ref.working_area.y.to_string());
}

///////////////
// Window events
fn resize_area(pa: RefPA) {
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
    let canvas_offset = Point {
        x: (canvas_width as f64 - working_area.x).abs() / 4.,
        y: (canvas_height as f64 - working_area.y).abs() / 3.,
    };
    let dx = canvas_width as f64 / working_area.x / 0.3;
    let dy = canvas_height as f64 / working_area.y / 0.3;
    pa_ref.canvas_offset = canvas_offset;
    pa_ref.global_scale = dx.min(dy);
}
fn on_window_resize(pa: RefPA, _event: Event) {
    resize_area(pa.clone());
    render(pa.clone());
}
fn on_window_click(_pa: RefPA, _event: Event) {
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
fn on_icon_click(pa: RefPA, event: Event) {
    let mut pa_mut = pa.borrow_mut();
    if let Some(target) = event.target() {
        if let Some(element) = wasm_bindgen::JsCast::dyn_ref::<Element>(&target) {
            if let Some(id) = element.get_attribute("id") {
                if let Some(key) = pa_mut.user_icons.keys().find(|&&k| k == id) {
                    if key == &"icon-cog" {
                        pa_mut
                            .settings_panel
                            .style()
                            .set_property("display", "block")
                            .unwrap();
                        pa_mut
                            .modal_backdrop
                            .style()
                            .set_property("display", "block")
                            .unwrap();
                    } else {
                        pa_mut.icon_selected = key;
                        deselect_icons(&pa_mut);
                        select_icon(&pa_mut, &id);
                    }
                    match pa_mut.icon_selected {
                        "icon-line" | "icon-quadbezier" | "icon-cubicbezier" | "icon-ellipse"
                        | "icon-rectangle" => {
                            if let None = pa_mut.draw_vertex {
                                let v = pa_mut.add_vertex(&Point::ZERO);
                                pa_mut.draw_vertex = Some(v.id);
                            }
                        }
                        _ => pa_mut.draw_vertex = None,
                    }
                }
            }
        }
    }
}
fn on_icon_mouseover(pa: RefPA, event: Event) {
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
fn on_icon_mouseout(pa: RefPA, _event: Event) {
    pa.borrow_mut()
        .tooltip
        .style()
        .set_property("display", "none")
        .expect("Failed to set display property");
}

///////////////
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

///////////////
// Rendering
fn render(pa: RefPA) {
    let pa_ref = pa.borrow();

    // Clear the canvas
    raw_draw_clear_canvas(&pa_ref);
    drop(pa_ref);

    // Then draw all
    draw_all(pa.clone());
}
fn draw_all(pa: RefPA) {
    draw_grid(pa.clone());
    draw_working_area(pa.clone());
    draw_content(pa.clone());
    draw_selection_area(pa.clone());
}
fn draw_working_area(pa: RefPA) {
    let pa_ref = pa.borrow();
    // Draw working area

    let _wa = pa_ref.working_area;
    // Title
    // cst.push(CTText(Point::new(wa.x / 3., -20.), "Working sheet".into()));

    // let mut shape = ShapeType::new_line(Line::new(pick_pos, pick_pos + (snap_grid, snap_grid)));

    // // Arrows
    // let mut pos = Point::new(0., -10.);
    // prefab::arrow_right(pos, 100., &mut cst);
    // cst.push(CTText(Point::new(40., -20.), "X".into()));

    // pos = Point::new(-10., 0.);
    // prefab::arrow_down(pos, 100., &mut cst);
    // cst.push(CTText(Point::new(-30., 50.), "Y".into()));

    // // Border
    // use ConstructionPattern::*;
    // pos = Point::ZERO;
    // cst.push(CTSegment(NoSelection, pos, pos + (0., wa.y)));
    // cst.push(CTSegment(NoSelection, pos, pos + (wa.x, 0.)));
    // pos = Point::new(wa.x, wa.y);
    // cst.push(CTSegment(NoSelection, pos, pos + (-wa.x, 0.)));
    // cst.push(CTSegment(NoSelection, pos, pos + (0., -wa.y)));

    // draw_shape(&pa_ref, &cst);
}
fn draw_grid(pa: RefPA) {
    let pa_ref = pa.borrow();
    let wa = pa_ref.working_area;
    let w_grid_spacing = pa_ref.working_area_visual_grid;

    use PathEl::*;
    let mut v: Vec<PathEl> = vec![];
    // Vertical grid lines
    let mut wx = 0.;
    while wx <= wa.x {
        v.push(MoveTo(Point::new(wx, 0.)));
        v.push(LineTo(Point::new(wx, wa.y)));
        wx += w_grid_spacing
    }
    // Horizontal grid lines
    let mut wy = 0.;
    while wy <= wa.y {
        v.push(MoveTo(Point::new(0., wy)));
        v.push(LineTo(Point::new(wa.x, wy)));
        wy += w_grid_spacing;
    }
    draw_path(
        &pa_ref,
        &ConstructionBezierPath {
            layer: ConstructionLayer::Grid,
            pattern: ConstructionPattern::Normal,
            path: BezPath::from_vec(v),
            filled: false,
        },
    );
}
fn draw_content(pa: RefPA) {
    let pa_ref = pa.borrow();
    let scale = pa_ref.global_scale;
    let size_handle = pa_ref.size_handle;
    let tol = 0.01;

    let layer = ConstructionLayer::Worksheet;
    for sh in pa_ref.sh_pool.values() {
        let pattern = pa_ref.get_geobject_construction_pattern(Geobjects::Shape(sh.get_id()));
        // Draw the shape without the handles
        draw_path(
            &pa_ref,
            &ConstructionBezierPath {
                layer,
                pattern,
                path: sh.get_path(tol, &pa_ref.v_pool),
                filled: false,
            },
        );
        // Draw the handles
        sh.get_vertices_ids().iter().for_each(|v_id| {
            let v = pa_ref.v_pool.get(v_id).unwrap();
            let pattern = pa_ref.get_geobject_construction_pattern(Geobjects::Vertex(v.id));
            draw_path(
                &pa_ref,
                &ConstructionBezierPath {
                    layer,
                    pattern,
                    path: prefab::handle(v, size_handle, scale),
                    filled: true,
                },
            );
        });
        // Draw the geometry helpers
        //     // Draw the geometry helpers
        //     vcst = vec![];
        //     shape.get_helpers_construction(&mut vcst);
        //     raw_draw(&pa_ref, &vcst);
        // }
    }

    // Show pick point if requested
    if let Some(v_id) = pa_ref.draw_vertex {
        let v = pa_ref.v_pool.get(&v_id).unwrap();
        let pattern = ConstructionPattern::Normal;
        let path = prefab::handle(&v, size_handle, scale);
        draw_path(
            &pa_ref,
            &ConstructionBezierPath {
                layer,
                pattern,
                path,
                filled: false,
            },
        );
    }
}
fn draw_selection_area(_pa: RefPA) {
    // use ConstructionPattern::*;
    // let pa_ref = pa.borrow();
    // if let Some(sa) = pa_ref.selection_area {
    //     let bl = sa[0];
    //     let tr = sa[1];
    //     if bl.x != tr.x && bl.y != tr.y {
    //         let tl = Point::new(bl.x, tr.y);
    //         let br = Point::new(tr.x, bl.y);

    //         let mut cst = Vec::new();
    //         // cst.push(Move(bl));
    //         cst.push(CTSegment(NoSelection, bl, tl));
    //         cst.push(CTSegment(NoSelection, tl, tr));
    //         cst.push(CTSegment(NoSelection, tr, br));
    //         cst.push(CTSegment(NoSelection, br, bl));
    //         raw_draw(&pa_ref, &cst, ConstructionLayer::SelectionTool);
    //     }
    // }
}
fn set_style(pa_ref: &Ref<'_, PlayingArea>, cbp: &ConstructionBezierPath) {
    let (fill_color, stroke_color, stroke_style, stroke_width) =
        pa_ref.draw_styles.get_default_styles(cbp);
    pa_ref.ctx.set_font("20px sans-serif");
    pa_ref.ctx.set_line_dash(stroke_style).unwrap();
    pa_ref.ctx.set_line_width(stroke_width);
    pa_ref.ctx.set_stroke_style(&stroke_color.into());
    pa_ref.ctx.set_fill_style(&fill_color.into());
}
fn draw_path(pa_ref: &Ref<'_, PlayingArea>, cbp: &ConstructionBezierPath) {
    // let p = Path2d::new().unwrap();
    let scale = pa_ref.global_scale;
    let offset = pa_ref.canvas_offset;
    set_style(pa_ref, cbp);
    pa_ref.ctx.begin_path();

    for cst in cbp.path.iter() {
        match cst {
            PathEl::MoveTo(pt) => {
                let cpt = to_canvas(&pt, scale, &offset);
                pa_ref.ctx.move_to(cpt.x, cpt.y);
            }
            PathEl::LineTo(pt) => {
                let cpt = to_canvas(&pt, scale, &offset);
                pa_ref.ctx.line_to(cpt.x, cpt.y);
            }
            PathEl::QuadTo(pt1, pt2) => {
                let cpt1 = to_canvas(&pt1, scale, &offset);
                let cpt2 = to_canvas(&pt2, scale, &offset);
                pa_ref
                    .ctx
                    .quadratic_curve_to(cpt1.x, cpt1.y, cpt2.x, cpt2.y);
            }
            PathEl::CurveTo(pt1, pt2, pt3) => {
                let cpt1 = to_canvas(&pt1, scale, &offset);
                let cpt2 = to_canvas(&pt2, scale, &offset);
                let cpt3 = to_canvas(&pt3, scale, &offset);
                pa_ref
                    .ctx
                    .bezier_curve_to(cpt1.x, cpt1.y, cpt2.x, cpt2.y, cpt3.x, cpt3.y);
            }
            PathEl::ClosePath => (),
        }
    }
    if cbp.filled {
        pa_ref.ctx.fill();
    }
    pa_ref.ctx.close_path();
    pa_ref.ctx.stroke();
}
fn raw_draw_clear_canvas(pa_ref: &Ref<'_, PlayingArea>) {
    pa_ref.ctx.set_stroke_style(&"#F00".into());
    let background_color = pa_ref.draw_styles.get_background_color();
    pa_ref
        .ctx
        .set_fill_style(&background_color.to_string().into());

    pa_ref.ctx.fill();
    let (canvas_width, canvas_height) =
        { (pa_ref.canvas.width() as f64, pa_ref.canvas.height() as f64) };
    pa_ref
        .ctx
        .fill_rect(0., 0., canvas_width as f64, canvas_height as f64);
}
