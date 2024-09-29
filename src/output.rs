#[macro_export]
macro_rules! display {
    ($($tee:tt)*) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "wasm")] {
                log::info!($($tee)*)
            } else {
                print!($($tee)*)
            }
        }
    };
}

#[macro_export]
macro_rules! display_error {
    ($($tee:tt)*) => {
        cfg_if::cfg_if! {
            if #[cfg(feature = "wasm")] {
                log::error!($($tee)*)
            } else {
                eprintln!($($tee)*)
            }
        }
    };
}