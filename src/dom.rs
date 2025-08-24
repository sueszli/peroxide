use wasm_bindgen::{JsCast, prelude::*};
use web_sys::*;

thread_local! {
    static DOC: Document = web_sys::window().unwrap().document().unwrap();
}

pub fn document() -> Document {
    DOC.with(|d| d.clone())
}

pub fn window() -> Window {
    web_sys::window().unwrap()
}

pub fn get_element_by_id<T: JsCast>(id: &str) -> Option<T> {
    document().get_element_by_id(id)?.dyn_into::<T>().ok()
}

pub fn get_canvas_context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas.get_context("2d").ok()??.dyn_into::<CanvasRenderingContext2d>().ok()
}

pub fn setup_canvas_scaling(canvas: &HtmlCanvasElement, context: &CanvasRenderingContext2d) {
    let window = window();
    let device_pixel_ratio = window.device_pixel_ratio();
    let display_width = (canvas.client_width() as f64 * device_pixel_ratio) as u32;
    let display_height = (canvas.client_height() as f64 * device_pixel_ratio) as u32;

    canvas.set_width(display_width);
    canvas.set_height(display_height);

    let _ = context.scale(device_pixel_ratio, device_pixel_ratio);
    context.set_image_smoothing_enabled(false);
}

pub fn onkeydown<F: 'static + FnMut(KeyboardEvent)>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut(KeyboardEvent)>);
    element.add_event_listener_with_callback("keydown", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}

pub fn onkeyup<F: 'static + FnMut(KeyboardEvent)>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut(KeyboardEvent)>);
    element.add_event_listener_with_callback("keyup", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}

pub fn onclick<F: 'static + FnMut()>(element: &Element, function: F) {
    let callback = Closure::wrap(Box::new(function) as Box<dyn FnMut()>);
    element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref()).unwrap();
    callback.forget();
}

pub fn set_text_content(element: &Element, text: &str) {
    element.set_text_content(Some(text));
}

pub fn set_attribute(element: &Element, name: &str, value: &str) {
    let _ = element.set_attribute(name, value);
}

pub fn set_inner_html(element: &Element, html: &str) {
    element.set_inner_html(html);
}

pub fn create_element(tag: &str) -> Element {
    document().create_element(tag).unwrap()
}

pub fn append_child(parent: &Element, child: &Element) {
    let _ = parent.append_child(child);
}

pub fn set_id(element: &Element, id: &str) {
    element.set_id(id);
}

pub fn get_textarea_value(id: &str) -> Option<String> {
    let textarea: HtmlTextAreaElement = get_element_by_id(id)?;
    Some(textarea.value())
}

pub fn set_textarea_inner_html(id: &str, content: &str) -> Result<(), JsValue> {
    let textarea: HtmlTextAreaElement = get_element_by_id(id).ok_or("Element not found")?;
    textarea.set_inner_html(content);
    Ok(())
}

#[cfg(test)]
#[allow(dead_code, unused_variables, unused_imports)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_create_element() {
        let div = create_element("div");
        assert_eq!(div.tag_name(), "DIV");
    }

    #[wasm_bindgen_test]
    fn test_set_and_get_id() {
        let div = create_element("div");
        set_id(&div, "test-id");
        assert_eq!(div.id(), "test-id");
    }

    #[wasm_bindgen_test]
    fn test_set_text_content() {
        let div = create_element("div");
        set_text_content(&div, "Hello World");
        assert_eq!(div.text_content().unwrap(), "Hello World");
    }

    #[wasm_bindgen_test]
    fn test_set_attribute() {
        let div = create_element("div");
        set_attribute(&div, "data-test", "value");
        assert_eq!(div.get_attribute("data-test").unwrap(), "value");
    }

    #[wasm_bindgen_test]
    fn test_set_inner_html() {
        let div = create_element("div");
        set_inner_html(&div, "<span>test</span>");
        assert_eq!(div.inner_html(), "<span>test</span>");
    }

    #[wasm_bindgen_test]
    fn test_append_child() {
        let parent = create_element("div");
        let child = create_element("span");
        set_text_content(&child, "child");

        append_child(&parent, &child);
        assert_eq!(parent.children().length(), 1);
        assert_eq!(parent.inner_html(), "<span>child</span>");
    }

    #[wasm_bindgen_test]
    fn test_get_element_by_id_success() {
        let div = create_element("div");
        set_id(&div, "test-element");
        document().body().unwrap().append_child(&div).unwrap();

        let found: Option<Element> = get_element_by_id("test-element");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id(), "test-element");

        div.remove();
    }

    #[wasm_bindgen_test]
    fn test_get_element_by_id_not_found() {
        let found: Option<Element> = get_element_by_id("non-existent-id");
        assert!(found.is_none());
    }

    #[wasm_bindgen_test]
    fn test_get_canvas_context() {
        let canvas: HtmlCanvasElement = create_element("canvas").dyn_into().unwrap();
        let context = get_canvas_context(&canvas);
        assert!(context.is_some());
    }

    #[wasm_bindgen_test]
    fn test_setup_canvas_scaling() {
        let canvas: HtmlCanvasElement = create_element("canvas").dyn_into().unwrap();
        set_attribute(&canvas, "style", "width: 800px; height: 600px;");
        document().body().unwrap().append_child(&canvas).unwrap();

        let context = get_canvas_context(&canvas).unwrap();
        setup_canvas_scaling(&canvas, &context);

        // canvas dimensions should be set based on device pixel ratio
        assert!(canvas.width() > 0);
        assert!(canvas.height() > 0);

        canvas.remove();
    }

    #[wasm_bindgen_test]
    fn test_get_textarea_value() {
        let textarea: HtmlTextAreaElement = create_element("textarea").dyn_into().unwrap();
        set_id(&textarea, "test-textarea");
        textarea.set_value("test value");
        document().body().unwrap().append_child(&textarea).unwrap();

        let value = get_textarea_value("test-textarea");
        assert_eq!(value, Some("test value".to_string()));

        textarea.remove();
    }

    #[wasm_bindgen_test]
    fn test_get_textarea_value_not_found() {
        let value = get_textarea_value("non-existent-textarea");
        assert!(value.is_none());
    }

    #[wasm_bindgen_test]
    fn test_set_textarea_inner_html() {
        let textarea: HtmlTextAreaElement = create_element("textarea").dyn_into().unwrap();
        set_id(&textarea, "test-textarea-html");
        document().body().unwrap().append_child(&textarea).unwrap();

        let result = set_textarea_inner_html("test-textarea-html", "test content");
        assert!(result.is_ok());
        assert_eq!(textarea.inner_html(), "test content");

        textarea.remove();
    }

    #[wasm_bindgen_test]
    fn test_set_textarea_inner_html_not_found() {
        let result = set_textarea_inner_html("non-existent-textarea", "content");
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    fn test_onclick_event_handler() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let button = create_element("button");
        set_id(&button, "test-button");
        document().body().unwrap().append_child(&button).unwrap();

        let clicked = Rc::new(RefCell::new(false));
        let clicked_clone = clicked.clone();

        onclick(&button, move || {
            *clicked_clone.borrow_mut() = true;
        });

        // simulate click event
        let event = web_sys::MouseEvent::new("click").unwrap();
        button.dispatch_event(&event).unwrap();

        // give time for the event to process
        assert!(*clicked.borrow());

        button.remove();
    }

    #[wasm_bindgen_test]
    fn test_window_function() {
        let win = window();
        assert!(win.is_instance_of::<Window>());
    }

    #[wasm_bindgen_test]
    fn test_document_function() {
        let doc = document();
        assert!(doc.is_instance_of::<Document>());
    }
}
