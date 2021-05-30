use polar_core::polar;
use wasm_bindgen::prelude::*;

use crate::JsResult;

#[wasm_bindgen]
pub struct Polar(polar::Polar);

fn err_string(e: impl std::error::Error) -> JsValue {
    e.to_string().into()
}

#[wasm_bindgen]
impl Polar {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new() -> Self {
        console_error_panic_hook::set_once();
        Self(polar::Polar::new())
    }

    #[wasm_bindgen(js_class = Polar, js_name = load)]
    pub fn wasm_load(&self, src: &str, filename: Option<String>) -> JsResult<()> {
        self.0
            .load(src, filename)
            .map_err(err_string)
    }

    #[wasm_bindgen(js_class = Polar, js_name = clearRules)]
    pub fn wasm_clear_rules(&self) {
        self.0.clear_rules()
    }
}
