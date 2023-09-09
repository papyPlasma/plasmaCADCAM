use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{
    console, CanvasRenderingContext2d, Document, Element, Event, HtmlCanvasElement, HtmlElement,
    MouseEvent, Window,
};

pub type ElementCallback = Box<dyn Fn(Rc<RefCell<PlayingArea>>, Event) + 'static>;

pub enum ToolSelected {
    Pointer,
    Selection,
    DrawLine,
    DrawQuadBezier,
    DrawCubicBezier,
    DrawCircle,
    DrawSquare,
}
#[allow(dead_code)]
pub struct PlayingArea {
    pub window: Window,
    pub document: Document,
    pub body: HtmlElement,
    pub canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    pub tool_selected: ToolSelected,
    //
    pressed: bool,
    // DOM
    contex_menu: HtmlElement,
    user_icons: HashMap<String, Element>,
}

// Canvas events
pub fn on_mouse_down(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        pa.borrow_mut().ctx.begin_path();
        pa.borrow_mut()
            .ctx
            .move_to(mouse_event.offset_x() as f64, mouse_event.offset_y() as f64);
        pa.borrow_mut().pressed = true;
    }
}
pub fn on_mouse_move(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        if pa.borrow_mut().pressed {
            pa.borrow_mut()
                .ctx
                .line_to(mouse_event.offset_x() as f64, mouse_event.offset_y() as f64);
            pa.borrow_mut().ctx.stroke();
            pa.borrow_mut().ctx.begin_path();
            pa.borrow_mut()
                .ctx
                .move_to(mouse_event.offset_x() as f64, mouse_event.offset_y() as f64);
        }
    }
}
pub fn on_mouse_up(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        pa.borrow_mut().pressed = false;
        pa.borrow_mut()
            .ctx
            .line_to(mouse_event.offset_x() as f64, mouse_event.offset_y() as f64);
        pa.borrow_mut().ctx.stroke();
    }
}
#[allow(dead_code)]
pub fn on_mouse_enter(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
#[allow(dead_code)]
pub fn on_mouse_leave(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
#[allow(dead_code)]
pub fn on_mouse_wheel(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
#[allow(dead_code)]
pub fn on_keydown(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
#[allow(dead_code)]
pub fn on_context_menu(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
// Window events
#[allow(dead_code)]
pub fn on_window_resize(_pa: Rc<RefCell<PlayingArea>>, _event: Event) {}
#[allow(dead_code)]
pub fn on_window_click(pa: Rc<RefCell<PlayingArea>>, event: Event) {
    /*No button: 0
    Left button (Primary): 1
    Right button (Secondary): 2
    Middle button (Auxiliary): 4 */
    if let Ok(mouse_event) = event.clone().dyn_into::<MouseEvent>() {
        if mouse_event.buttons() == 1 {
            // Not a right-click
            pa.borrow_mut()
                .contex_menu
                .style()
                .set_property("display", "none")
                .expect("failed to set display property");
        }
    }
}
// Icons events
#[allow(dead_code)]
pub fn on_pointer_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    pa.borrow_mut().tool_selected = ToolSelected::Pointer;
    select_icon(pa);
    console::log_1(&"AA click".into());
}
pub fn on_pointer_mouseover(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    select_icon(pa);
    console::log_1(&"AA mouseover".into());
}
pub fn on_selection_click(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    pa.borrow_mut().tool_selected = ToolSelected::Pointer;
    select_icon(pa);
    console::log_1(&"BB click".into());
}
pub fn on_selection_mouseover(pa: Rc<RefCell<PlayingArea>>, _event: Event) {
    select_icon(pa);
    console::log_1(&"BB mouseover".into());
}

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

    let playing_area = Rc::new(RefCell::new(PlayingArea {
        window,
        document,
        body,
        canvas,
        ctx,
        tool_selected: ToolSelected::Pointer,
        //
        pressed: false,
        //
        contex_menu,
        user_icons: HashMap::new(),
    }));

    init_window(playing_area.clone())?;
    init_canvas(playing_area.clone())?;
    init_icons(playing_area.clone())?;

    Ok(playing_area)
}

pub fn init_window(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
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

pub fn init_icons(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
    // Pointer icon
    let element_name = "icon-pointer";
    let mut element = get_element(&pa.borrow().document, element_name)?;
    pa.borrow_mut()
        .user_icons
        .insert(element_name.into(), element.clone());
    set_callback(
        pa.clone(),
        "click".into(),
        &mut element,
        Box::new(on_pointer_click),
    )?;
    set_callback(
        pa.clone(),
        "mouseover".into(),
        &mut element,
        Box::new(on_pointer_mouseover),
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

    Ok(())
}

pub fn init_canvas(pa: Rc<RefCell<PlayingArea>>) -> Result<(), JsValue> {
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

    Ok(())
}

pub fn set_callback(
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

//
fn select_icon(pa: Rc<RefCell<PlayingArea>>) {
    deselect_icons(pa);
}
fn deselect_icons(_pa: Rc<RefCell<PlayingArea>>) {
    //
}

pub fn get_element(document: &Document, element_id: &str) -> Result<Element, JsValue> {
    let element = document
        .get_element_by_id(element_id)
        .ok_or_else(|| JsValue::from_str("should have element on the page"))?
        .dyn_into()
        .map_err(|_| JsValue::from_str("should be an HtmlElement"))?;
    Ok(element)
}
