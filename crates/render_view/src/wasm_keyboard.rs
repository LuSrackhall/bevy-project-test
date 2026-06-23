use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

use bevy::prelude::*;
use wasm_bindgen::prelude::*;

/// WASM-only keyboard state captured from DOM events.
/// Uses Rc<RefCell> which is !Send — safe because WASM is single-threaded.
#[derive(Resource, Clone)]
pub struct WasmKeyboard {
    inner: Rc<RefCell<HashSet<String>>>,
    just_pressed: Rc<RefCell<HashSet<String>>>,
}

impl Default for WasmKeyboard {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(HashSet::new())),
            just_pressed: Rc::new(RefCell::new(HashSet::new())),
        }
    }
}

// SAFETY: WASM is single-threaded
unsafe impl Send for WasmKeyboard {}
unsafe impl Sync for WasmKeyboard {}

impl WasmKeyboard {
    pub fn just_pressed(&self, key: &str) -> bool {
        self.just_pressed.borrow().contains(key)
    }

    pub fn clear_just_pressed(&self) {
        self.just_pressed.borrow_mut().clear();
    }
}

pub fn setup_wasm_keyboard(mut commands: Commands) {
    let keyboard = WasmKeyboard::default();
    let pressed = keyboard.inner.clone();
    let just_pressed = keyboard.just_pressed.clone();

    let keydown_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let key = event.key();
        pressed.borrow_mut().insert(key.clone());
        just_pressed.borrow_mut().insert(key);
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    let pressed_up = keyboard.inner.clone();
    let keyup_closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
        let key = event.key();
        pressed_up.borrow_mut().remove(&key);
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(Some(canvas)) = document.query_selector("canvas") {
                let target: &web_sys::EventTarget = canvas.as_ref();
                let _ = target.add_event_listener_with_callback(
                    "keydown",
                    keydown_closure.as_ref().unchecked_ref(),
                );
                let _ = target.add_event_listener_with_callback(
                    "keyup",
                    keyup_closure.as_ref().unchecked_ref(),
                );
            }
        }
    }

    keydown_closure.forget();
    keyup_closure.forget();

    commands.insert_resource(keyboard);
}

pub fn clear_wasm_keyboard_just_pressed(kb: Option<Res<WasmKeyboard>>) {
    if let Some(kb) = kb {
        kb.clear_just_pressed();
    }
}
