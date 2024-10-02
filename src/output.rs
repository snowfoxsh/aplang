#[cfg(feature = "wasm")]
#[macro_export]
macro_rules! display {
    ($($tee:tt)*) => {
        $crate::wasm::OUT.with(|out| {
            if let Some(ref callback) = *out.borrow() {
                let this = wasm_bindgen::JsValue::NULL;

                callback.call2(
                    &this,
                    &wasm_bindgen::JsValue::from_str(format!($($tee)*).as_str()),
                    &wasm_bindgen::JsValue::from_bool(false),
                )
                .expect("could not make call to stdout");
            };
        })
    };
}

#[cfg(not(feature = "wasm"))]
#[macro_export]
macro_rules! display {
    ($($tee:tt)*) => {
        print!($($tee)*)
    };
}

#[cfg(feature = "wasm")]
#[macro_export]
macro_rules! display_error {
    ($($tee:tt)*) => {
        $crate::wasm::OUT.with(|out| {
            if let Some(ref callback) = *out.borrow() {
                let this = wasm_bindgen::JsValue::NULL;

                callback.call2(
                    &this,
                    &wasm_bindgen::JsValue::from_str(format!($($tee)*).as_str()),
                    &wasm_bindgen::JsValue::from_bool(true),
                )
                .expect("could not make call to stdout");
            };
        })
    };
}

#[cfg(not(feature = "wasm"))]
#[macro_export]
macro_rules! display_error {
    ($($tee:tt)*) => {
        eprint!($($tee)*)
    };
}