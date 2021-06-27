mod diagnostics;
mod inspect;

use polar_core::polar;
use wasm_bindgen::prelude::*;

use serde_wasm_bindgen::to_value;

type JsResult<T> = Result<T, wasm_bindgen::JsValue>;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// Wrapper for the `polar_core::Polar` type.
/// Used as the API interface for all the analytics
#[wasm_bindgen]
pub struct Polar(polar::Polar);

#[wasm_bindgen]
impl Polar {
    #[wasm_bindgen(constructor)]
    pub fn wasm_new() -> Self {
        console_error_panic_hook::set_once();
        Self(polar::Polar::new())
    }

    /// Loads a file into the knowledge base.
    ///
    /// In comparison to the `Polar` in the core, this
    /// will first remove the file.
    #[wasm_bindgen(js_class = Polar, js_name = load)]
    pub fn load(&self, src: &str, filename: &str) -> JsResult<()> {
        let old = self.0.remove_file(filename);
        self.0.load(src, Some(filename.to_string())).map_err(|e| {
            if let Some(old_src) = old {
                self.0
                    .load(&old_src, Some(filename.to_string()))
                    .expect("failed to reload old policy after new policy loading failed");
            }
            e.to_string().into()
        })
    }

    #[wasm_bindgen(js_class = Polar, js_name = clearRules)]
    pub fn clear_rules(&self) {
        self.0.clear_rules()
    }

    #[wasm_bindgen(js_class = Polar, js_name = getRuleInfo)]
    pub fn get_rule_info(&self) -> JsValue {
        let kb = self.0.kb.read().unwrap();
        to_value(&inspect::get_rule_info(&kb)).unwrap()
    }

    #[wasm_bindgen(js_class = Polar, js_name = getParseErrors)]
    pub fn get_parse_errors(&self, src: &str) -> JsValue {
        to_value(&diagnostics::find_parse_errors(&src)).unwrap()
    }

    #[wasm_bindgen(js_class = Polar, js_name = getUnusedRules)]
    pub fn get_unused_rules(&self, src: &str) -> JsValue {
        let kb = self.0.kb.read().unwrap();
        to_value(&diagnostics::find_unused_rules(&kb, src)).unwrap()
    }
}
