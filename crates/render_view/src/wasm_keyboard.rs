use bevy::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use wasm_bindgen::prelude::*;

/// WASM-only keyboard state captured from a hidden HTML input element.
/// Avoids canvas focus issues by using a dedicated DOM input for keystroke capture.
#[derive(Resource, Clone)]
pub struct WasmKeyboard {
    just_pressed: Rc<RefCell<HashSet<String>>>,
    hidden_input: Rc<RefCell<Option<web_sys::HtmlInputElement>>>,
}

// SAFETY: WASM is single-threaded
unsafe impl Send for WasmKeyboard {}
unsafe impl Sync for WasmKeyboard {}

impl Default for WasmKeyboard {
    fn default() -> Self {
        Self {
            just_pressed: Rc::new(RefCell::new(HashSet::new())),
            hidden_input: Rc::new(RefCell::new(None)),
        }
    }
}

impl WasmKeyboard {
    pub fn just_pressed(&self, key: &str) -> bool {
        self.just_pressed.borrow().contains(key)
    }

    pub fn clear_just_pressed(&self) {
        self.just_pressed.borrow_mut().clear();
    }

    pub fn focus_hidden_input(&self) {
        if let Some(input) = self.hidden_input.borrow().as_ref() {
            input.set_value("");
            let _ = input.focus();
        }
    }
}

pub fn setup_wasm_keyboard(mut commands: Commands) {
    let keyboard = WasmKeyboard::default();

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(el) = document.create_element("input") {
                let input: web_sys::HtmlInputElement = el.unchecked_into();
                input.set_type("text");
                let _ = input.set_attribute("id", "wasm-keyboard-capture");
                let _ = input.set_attribute("autocomplete", "off");
                let _ = input.set_attribute("autocorrect", "off");
                let _ = input.set_attribute("autocapitalize", "off");
                let _ = input.set_attribute("spellcheck", "false");

                // Invisible but focusable
                let style = input.style();
                let _ = style.set_property("position", "fixed");
                let _ = style.set_property("left", "-9999px");
                let _ = style.set_property("top", "-9999px");
                let _ = style.set_property("width", "1px");
                let _ = style.set_property("height", "1px");
                let _ = style.set_property("opacity", "0");

                // Capture keydown
                let just_pressed = keyboard.just_pressed.clone();
                let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                    let key = event.key();
                    event.prevent_default();
                    just_pressed.borrow_mut().insert(key);
                })
                    as Box<dyn FnMut(web_sys::KeyboardEvent)>);

                let target: &web_sys::EventTarget = input.as_ref();
                let _ = target
                    .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref());
                closure.forget();

                if let Some(body) = document.body() {
                    let _ = body.append_child(&input);
                }

                *keyboard.hidden_input.borrow_mut() = Some(input);
            }
        }
    }

    commands.insert_resource(keyboard);
}

/// Re-focus hidden input each frame while seek panel input is active.
pub fn maintain_wasm_keyboard_focus(
    state: Res<crate::ui::hud::SeekPanelState>,
    kb: Option<Res<WasmKeyboard>>,
) {
    if state.input_active {
        if let Some(kb) = kb {
            kb.focus_hidden_input();
        }
    }
}

pub fn clear_wasm_keyboard_just_pressed(kb: Option<Res<WasmKeyboard>>) {
    if let Some(kb) = kb {
        kb.clear_just_pressed();
    }
}
