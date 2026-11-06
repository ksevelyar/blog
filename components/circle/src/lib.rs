#[cfg(target_arch = "wasm32")]
mod browser_component {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::*;
    use web_sys::{Element, Window};

    #[wasm_bindgen]
    pub fn mount(mount_element: Element) -> Result<(), JsValue> {
        let browser_window = web_sys::window()
            .ok_or_else(|| JsValue::from_str("the browser window is unavailable"))?;
        let browser_document = browser_window
            .document()
            .ok_or_else(|| JsValue::from_str("the browser document is unavailable"))?;
        let svg_namespace = "http://www.w3.org/2000/svg";
        let svg_element = browser_document.create_element_ns(Some(svg_namespace), "svg")?;
        svg_element.set_attribute("viewBox", "0 0 200 200")?;
        svg_element.set_attribute("width", "200")?;
        svg_element.set_attribute("height", "200")?;
        svg_element.set_attribute("role", "img")?;
        svg_element.set_attribute("aria-label", "Animated purple circle")?;
        svg_element.set_attribute("data-testid", "wasm-circle-svg")?;

        let circle_element = browser_document.create_element_ns(Some(svg_namespace), "circle")?;
        circle_element.set_attribute("cx", "100")?;
        circle_element.set_attribute("cy", "100")?;
        circle_element.set_attribute("r", "78")?;
        circle_element.set_attribute("fill", "#800080")?;
        circle_element.set_attribute("data-testid", "wasm-circle")?;
        svg_element.append_child(&circle_element)?;
        mount_element.append_child(&svg_element)?;

        animate_circle_fill(browser_window, circle_element)
    }

    fn animate_circle_fill(browser_window: Window, circle_element: Element) -> Result<(), JsValue> {
        let animation_callback: Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>> =
            Rc::new(RefCell::new(None));
        let callback_reference = Rc::clone(&animation_callback);
        let animation_window = browser_window.clone();

        *animation_callback.borrow_mut() = Some(Closure::new(move |animation_timestamp: f64| {
            let circle_fill = interpolate_circle_fill(animation_timestamp);
            let _ = circle_element.set_attribute("fill", &circle_fill);

            let callback_reference = callback_reference.borrow();
            if let Some(animation_callback) = callback_reference.as_ref() {
                animation_window
                    .request_animation_frame(animation_callback.as_ref().unchecked_ref())
                    .expect("requestAnimationFrame must remain available");
            }
        }));

        let animation_callback_reference = animation_callback.borrow();
        let animation_callback = animation_callback_reference
            .as_ref()
            .ok_or_else(|| JsValue::from_str("the circle animation callback was not created"))?;
        browser_window.request_animation_frame(animation_callback.as_ref().unchecked_ref())?;
        Ok(())
    }

    fn interpolate_circle_fill(animation_timestamp: f64) -> String {
        let purple_channels = [128_u8, 0_u8, 128_u8];
        let lavender_channels = [230_u8, 230_u8, 250_u8];
        let transition_cycle_milliseconds = 2400.0;
        let cycle_position =
            animation_timestamp % transition_cycle_milliseconds / transition_cycle_milliseconds;
        let lavender_progress = if cycle_position <= 0.5 {
            cycle_position * 2.0
        } else {
            (1.0 - cycle_position) * 2.0
        };
        let red_channel =
            interpolate_color_channel(purple_channels[0], lavender_channels[0], lavender_progress);
        let green_channel =
            interpolate_color_channel(purple_channels[1], lavender_channels[1], lavender_progress);
        let blue_channel =
            interpolate_color_channel(purple_channels[2], lavender_channels[2], lavender_progress);

        format!("#{red_channel:02X}{green_channel:02X}{blue_channel:02X}")
    }

    fn interpolate_color_channel(start_channel: u8, end_channel: u8, progress: f64) -> u8 {
        let start_value = f64::from(start_channel);
        let channel_difference = f64::from(end_channel) - start_value;
        (start_value + channel_difference * progress).round() as u8
    }
}
