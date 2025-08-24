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

#[cfg(test)]
#[allow(dead_code, unused_variables, unused_imports)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_create_element() {
        let div = document().create_element("div").unwrap();
        assert_eq!(div.tag_name(), "DIV");
    }

    #[wasm_bindgen_test]
    fn test_set_and_get_id() {
        let div = document().create_element("div").unwrap();
        div.set_id("test-id");
        assert_eq!(div.id(), "test-id");
    }

    #[wasm_bindgen_test]
    fn test_set_text_content() {
        let div = document().create_element("div").unwrap();
        div.set_text_content(Some("Hello World"));
        assert_eq!(div.text_content().unwrap(), "Hello World");
    }

    #[wasm_bindgen_test]
    fn test_set_attribute() {
        let div = document().create_element("div").unwrap();
        let _ = div.set_attribute("data-test", "value");
        assert_eq!(div.get_attribute("data-test").unwrap(), "value");
    }

    #[wasm_bindgen_test]
    fn test_set_inner_html() {
        let div = document().create_element("div").unwrap();
        div.set_inner_html("<span>test</span>");
        assert_eq!(div.inner_html(), "<span>test</span>");
    }

    #[wasm_bindgen_test]
    fn test_append_child() {
        let parent = document().create_element("div").unwrap();
        let child = document().create_element("span").unwrap();
        child.set_text_content(Some("child"));

        let _ = parent.append_child(&child);
        assert_eq!(parent.children().length(), 1);
        assert_eq!(parent.inner_html(), "<span>child</span>");
    }

    #[wasm_bindgen_test]
    fn test_get_element_by_id_success() {
        let div = document().create_element("div").unwrap();
        div.set_id("test-element");
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
        let canvas: HtmlCanvasElement = document().create_element("canvas").unwrap().dyn_into().unwrap();
        let context = get_canvas_context(&canvas);
        assert!(context.is_some());
    }

    #[wasm_bindgen_test]
    fn test_setup_canvas_scaling() {
        let canvas: HtmlCanvasElement = document().create_element("canvas").unwrap().dyn_into().unwrap();
        let _ = canvas.set_attribute("style", "width: 800px; height: 600px;");
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
        let textarea: HtmlTextAreaElement = document().create_element("textarea").unwrap().dyn_into().unwrap();
        textarea.set_id("test-textarea");
        textarea.set_value("test value");
        document().body().unwrap().append_child(&textarea).unwrap();

        let found_textarea: Option<HtmlTextAreaElement> = get_element_by_id("test-textarea");
        let value = found_textarea.map(|t| t.value());
        assert_eq!(value, Some("test value".to_string()));

        textarea.remove();
    }

    #[wasm_bindgen_test]
    fn test_get_textarea_value_not_found() {
        let found_textarea: Option<HtmlTextAreaElement> = get_element_by_id("non-existent-textarea");
        let value = found_textarea.map(|t| t.value());
        assert!(value.is_none());
    }

    #[wasm_bindgen_test]
    fn test_set_textarea_inner_html() {
        let textarea: HtmlTextAreaElement = document().create_element("textarea").unwrap().dyn_into().unwrap();
        textarea.set_id("test-textarea-html");
        document().body().unwrap().append_child(&textarea).unwrap();

        let found_textarea: Option<HtmlTextAreaElement> = get_element_by_id("test-textarea-html");
        let result = found_textarea.map(|t| {
            t.set_inner_html("test content");
        });
        assert!(result.is_some());
        assert_eq!(textarea.inner_html(), "test content");

        textarea.remove();
    }

    #[wasm_bindgen_test]
    fn test_set_textarea_inner_html_not_found() {
        let found_textarea: Option<HtmlTextAreaElement> = get_element_by_id("non-existent-textarea");
        let result = found_textarea.map(|t| {
            t.set_inner_html("content");
        });
        assert!(result.is_none());
    }

    #[wasm_bindgen_test]
    fn test_onclick_event_handler() {
        use std::cell::RefCell;
        use std::rc::Rc;

        let button = document().create_element("button").unwrap();
        button.set_id("test-button");
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
