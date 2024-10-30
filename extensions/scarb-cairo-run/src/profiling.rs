use std::{fs, iter::repeat, ops::AddAssign};

use cairo_lang_runner::profiling::ProfilingInfo;
use cairo_lang_sierra::program::Program;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use camino::Utf8PathBuf;
use itertools::Itertools;

/// Saves aggregated stack trace with weights
pub fn save_profiler_output(
    program: &Program,
    profiling_info: &ProfilingInfo,
    output_path: &Utf8PathBuf,
) -> anyhow::Result<()> {
    let mut stack_trace_agg: OrderedHashMap<String, usize> = Default::default();
    let mut stack = Vec::with_capacity(100);

    for (stack_trace, weight) in profiling_info.stack_trace_weights.iter() {
        let mut key_parts: Vec<String> = Vec::with_capacity(stack_trace.len());
        for idx in stack_trace {
            let func_name = program.funcs[*idx].id.to_string();
            if let Some(last_name) = key_parts.last() {
                if last_name == &func_name {
                    continue;
                }
            }
            key_parts.push(func_name);
        }

        let value = if stack_trace.len() > stack.len() {
            // Stack snapshot is done when function returns hence recursive calls will start
            // with a large trace (recursion depth or number of loop iterations).
            stack.extend(repeat(0).take(stack_trace.len() - stack.len() - 1));
            stack.push(*weight);
            *weight
        } else if stack_trace.len() < stack.len() {
            if stack_trace.len() == 1 {
                // Main function exits
                weight.checked_sub(stack.drain(..).sum()).unwrap()
            } else {
                let self_weight = weight.checked_sub(stack.pop().unwrap()).unwrap();
                stack.last_mut().unwrap().add_assign(weight);
                self_weight
            }
        } else {
            stack.last_mut().unwrap().add_assign(weight);
            *weight
        };

        let key = key_parts.join(";");
        if let Some(elt) = stack_trace_agg.get_mut(&key) {
            *elt += value;
        } else {
            stack_trace_agg.insert(key, value);
        }
    }

    let contents = stack_trace_agg
        .into_iter()
        .map(|(key, weight)| format!("{key} {weight}"))
        .join("\n");

    fs::write(output_path, contents)?;
    Ok(())
}
