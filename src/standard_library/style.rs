use crate::interpreter::FunctionMap;
use crate::interpreter::Value;
use crate::{display, std_function};
use mapro::map;
use std::collections::HashMap;

pub(super) fn std_style() -> FunctionMap {
    let mut functions = FunctionMap::new();

    std_function!(functions => fn STYLE(style: Value::String) {
         let styles: HashMap<&str, &str> = map! {
            // default colors
            "clear" => "\x1b[0m",
            "default_color" => "\x1b[39m",
            "bg_default_color" => "\x1b[49m",

            // Foreground colors
            "black" => "\x1b[30m",
            "red" => "\x1b[31m",
            "green" => "\x1b[32m",
            "yellow" => "\x1b[33m",
            "blue" => "\x1b[34m",
            "magenta" => "\x1b[35m",
            "cyan" => "\x1b[36m",
            "white" => "\x1b[37m",
            "bright_black" => "\x1b[90m",
            "bright_red" => "\x1b[91m",
            "bright_green" => "\x1b[92m",
            "bright_yellow" => "\x1b[93m",
            "bright_blue" => "\x1b[94m",
            "bright_magenta" => "\x1b[95m",
            "bright_cyan" => "\x1b[96m",
            "bright_white" => "\x1b[97m",

            // Background colors
            "bg_black" => "\x1b[40m",
            "bg_red" => "\x1b[41m",
            "bg_green" => "\x1b[42m",
            "bg_yellow" => "\x1b[43m",
            "bg_blue" => "\x1b[44m",
            "bg_magenta" => "\x1b[45m",
            "bg_cyan" => "\x1b[46m",
            "bg_white" => "\x1b[47m",
            "bg_bright_black" => "\x1b[100m",
            "bg_bright_red" => "\x1b[101m",
            "bg_bright_green" => "\x1b[102m",
            "bg_bright_yellow" => "\x1b[103m",
            "bg_bright_blue" => "\x1b[104m",
            "bg_bright_magenta" => "\x1b[105m",
            "bg_bright_cyan" => "\x1b[106m",
            "bg_bright_white" => "\x1b[107m",

            // styles
            "bold" => "\x1b[1m",
            "faint" => "\x1b[2m",
            "underline" => "\x1b[4m",
            "blink" => "\x1b[5m",
        };

        let Some(code) = styles.get(style.to_ascii_lowercase().as_str()) else {
            return Ok(Value::Bool(false))
        };

        display!("{}", code);

        return Ok(Value::Bool(true))
    });

    std_function!(functions => fn CLEAR_STYLE() {

        // clear all styles
        display!("\x1b[0m");

        return Ok(Value::Null)
    });

    functions
}
