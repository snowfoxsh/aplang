#[macro_export]
macro_rules! display {
    ($($tee:tt)*) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "wasm")] {
                OUT.with(|out| {
                    if let Some(ref callback) = *out.borrow() {
                        let this = wasm_bindgen::JsValue::NULL;

                        callback.call2(
                            &this,
                            &wasm_bindgen::JsValue::from_str(format!($($tee)*).as_str()),
                            &wasm_bindgen::JsValue::from_bool(false),
                        )
                        .expect("could not make call to stdout");
                    };
                });
            } else {
                print!($($tee)*)
            }
        }
    };
}

// fn display() {
//     OUT.with(|out| {
//         if let Some(ref callback) = *out.borrow() {
//             let this = JsValue::NULL;
//
//             callback.call2(
//                 &this,
//                 &JsValue::from_str(format!("hello").as_str()),
//                 &JsValue::from_bool(false)
//             ).expect("could not make call to stdout");
//         }
//     })
// }

#[macro_export]
macro_rules! display_error {
    ($($tee:tt)*) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "wasm")] {
                OUT.with(|out| {
                    if let Some(ref callback) = *out.borrow() {
                        let this = wasm_bindgen::JsValue::NULL;
                        callback.call2(
                            &this,
                            &wasm_bindgen::JsValue::from_str(format!($($tee)*).as_str()),
                            &wasm_bindgen::JsValue::from_bool(true),
                        )
                        .expect("could not make call to stdout");
                    };
                });
            } else {
                eprintln!($($tee)*)
            }
        }
    };
}