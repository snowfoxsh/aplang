use cfg_if::cfg_if;
// 
// cfg_if! {
//     if #[cfg(feature = "wasm") {
//         
//     } else {
//         
//     };
// }
// 

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


// #[macro_export]
// macro_rules! display_error {
//     ($($tee:tt)*) => {
//         // cfg_if::cfg_if! {
// //             if #[cfg(feature = "wasm")] {
//                 $crate::wasm::OUT.with(|out| {
//                     if let Some(ref callback) = *out.borrow() {
//                         let this = wasm_bindgen::JsValue::NULL;
//                         callback.call2(
//                             &this,
//                             &wasm_bindgen::JsValue::from_str(format!($($tee)*).as_str()),
//                             &wasm_bindgen::JsValue::from_bool(true),
//                         )
//                         .expect("could not make call to stdout");
//                     };
//                 })
//             // } else {
//             //     eprintln!($($tee)*)
//             // }
//         // }
//     };
// }

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