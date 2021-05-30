mod polar;

pub use polar::Polar;

type JsResult<T> = Result<T, wasm_bindgen::JsValue>;
