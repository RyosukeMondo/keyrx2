//! Modifier functions for Rhai DSL.
//!
//! Provides with_shift(), with_ctrl(), with_alt(), with_win(), with_mods() functions.

use crate::config::KeyCode;
use crate::parser::validators::parse_virtual_key;
use alloc::boxed::Box;
use alloc::format;
use rhai::Engine;

/// Temporary builder struct for physical modifier output.
/// Returned by with_shift(), with_ctrl(), etc.
/// Consumed by map() overload to create ModifiedOutput mapping.
#[derive(Clone, Debug)]
pub struct ModifiedKey {
    pub key: KeyCode,
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub win: bool,
}

/// Register modifier functions with the Rhai engine.
pub fn register_modifier_functions(engine: &mut Engine) {
    // Register ModifiedKey as Rhai type
    engine.register_type::<ModifiedKey>();

    // with_shift(key) - returns ModifiedKey with shift=true
    engine.register_fn(
        "with_shift",
        |key: &str| -> Result<ModifiedKey, Box<rhai::EvalAltResult>> {
            let vk = parse_virtual_key(key)
                .map_err(|e| format!("Invalid key in with_shift(): {}", e))?;
            Ok(ModifiedKey {
                key: vk,
                shift: true,
                ctrl: false,
                alt: false,
                win: false,
            })
        },
    );

    // with_ctrl(key) - returns ModifiedKey with ctrl=true
    engine.register_fn(
        "with_ctrl",
        |key: &str| -> Result<ModifiedKey, Box<rhai::EvalAltResult>> {
            let vk =
                parse_virtual_key(key).map_err(|e| format!("Invalid key in with_ctrl(): {}", e))?;
            Ok(ModifiedKey {
                key: vk,
                shift: false,
                ctrl: true,
                alt: false,
                win: false,
            })
        },
    );

    // with_alt(key) - returns ModifiedKey with alt=true
    engine.register_fn(
        "with_alt",
        |key: &str| -> Result<ModifiedKey, Box<rhai::EvalAltResult>> {
            let vk =
                parse_virtual_key(key).map_err(|e| format!("Invalid key in with_alt(): {}", e))?;
            Ok(ModifiedKey {
                key: vk,
                shift: false,
                ctrl: false,
                alt: true,
                win: false,
            })
        },
    );

    // with_win(key) - returns ModifiedKey with win=true
    engine.register_fn(
        "with_win",
        |key: &str| -> Result<ModifiedKey, Box<rhai::EvalAltResult>> {
            let vk =
                parse_virtual_key(key).map_err(|e| format!("Invalid key in with_win(): {}", e))?;
            Ok(ModifiedKey {
                key: vk,
                shift: false,
                ctrl: false,
                alt: false,
                win: true,
            })
        },
    );

    // with_mods(key, shift, ctrl, alt, win) - returns ModifiedKey with all specified modifiers
    engine.register_fn(
        "with_mods",
        |key: &str,
         shift: bool,
         ctrl: bool,
         alt: bool,
         win: bool|
         -> Result<ModifiedKey, Box<rhai::EvalAltResult>> {
            let vk =
                parse_virtual_key(key).map_err(|e| format!("Invalid key in with_mods(): {}", e))?;
            Ok(ModifiedKey {
                key: vk,
                shift,
                ctrl,
                alt,
                win,
            })
        },
    );
}
