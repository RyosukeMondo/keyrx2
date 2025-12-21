use keyrx_core::config::{Condition, ConditionItem, KeyMapping};
use rhai::{Array, Engine, EvalAltResult};
use std::sync::{Arc, Mutex};

use crate::parser::core::ParserState;
use crate::parser::validators::parse_condition_string;

pub fn register_when_functions(engine: &mut Engine, state: Arc<Mutex<ParserState>>) {
    let state_clone_single = Arc::clone(&state);
    engine.register_fn(
        "when",
        move |cond: &str, _mappings: Array| -> Result<(), Box<EvalAltResult>> {
            let condition =
                parse_condition_string(cond).map_err(|e| format!("Invalid condition: {}", e))?;
            add_conditional_mapping(&state_clone_single, condition)
        },
    );

    let state_clone_multi = Arc::clone(&state);
    engine.register_fn(
        "when",
        move |conds: Array, _mappings: Array| -> Result<(), Box<EvalAltResult>> {
            let mut condition_items = Vec::new();
            for cond_dyn in conds {
                let cond_str = cond_dyn
                    .into_string()
                    .map_err(|_| "Condition must be a string")?;
                let cond = parse_condition_string(&cond_str)
                    .map_err(|e| format!("Invalid condition: {}", e))?;
                match cond {
                    Condition::ModifierActive(id) => {
                        condition_items.push(ConditionItem::ModifierActive(id))
                    }
                    Condition::LockActive(id) => {
                        condition_items.push(ConditionItem::LockActive(id))
                    }
                    _ => return Err("Only single modifiers/locks allowed in array".into()),
                }
            }
            add_conditional_mapping(&state_clone_multi, Condition::AllActive(condition_items))
        },
    );

    let state_clone_not = Arc::clone(&state);
    engine.register_fn(
        "when_not",
        move |cond: &str, _mappings: Array| -> Result<(), Box<EvalAltResult>> {
            let condition =
                parse_condition_string(cond).map_err(|e| format!("Invalid condition: {}", e))?;
            let item = match condition {
                Condition::ModifierActive(id) => ConditionItem::ModifierActive(id),
                Condition::LockActive(id) => ConditionItem::LockActive(id),
                _ => return Err("Only single modifiers/locks allowed in when_not".into()),
            };
            add_conditional_mapping(&state_clone_not, Condition::NotActive(vec![item]))
        },
    );
}

fn add_conditional_mapping(
    state: &Arc<Mutex<ParserState>>,
    condition: Condition,
) -> Result<(), Box<EvalAltResult>> {
    let mut state = state.lock().unwrap();
    if let Some(ref mut device) = state.current_device {
        device.mappings.push(KeyMapping::Conditional {
            condition,
            mappings: Vec::new(),
        });
        Ok(())
    } else {
        Err("Conditional blocks must be called inside a device() block".into())
    }
}
