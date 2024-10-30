use std::{collections::HashMap, fs};

use cairo_lang_runner::profiling::ProfilingInfo;
use cairo_lang_sierra::program::Program;
use camino::Utf8PathBuf;
use itertools::Itertools;

/// Saves post-processed stack trace with weights
pub fn save_profiler_output_old(program: &Program, profiling_info: &ProfilingInfo, output_path: &Utf8PathBuf) -> anyhow::Result<()> {
    let format_stack_item = |(idx_stack_trace, weight): (&Vec<usize>, &usize)| {
        let key = index_stack_trace_to_name_stack_trace(program, &idx_stack_trace).join(";");
        format!("{key} {weight}")
    };

    let contents = profiling_info
        .stack_trace_weights
        .iter()
        .map(format_stack_item)
        .collect::<Vec::<String>>()
        .join("\n");

    fs::write(output_path, contents)?;
    Ok(())
}

/// Saves aggregated stack trace with weights
pub fn save_profiler_output(program: &Program, profiling_info: &ProfilingInfo, output_path: &Utf8PathBuf) -> anyhow::Result<()> {
    let mut stack_trace_agg: HashMap<String, usize> = Default::default();
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

        let key = key_parts.join(";");
        if let Some(elt) = stack_trace_agg.get_mut(&key) {
            *elt += weight;
        } else {
            stack_trace_agg.insert(key, *weight);
        }
    }

    let contents = stack_trace_agg
        .into_iter()
        .sorted()
        .map(|(key, weight)| format!("{key} {weight}"))
        .join("\n");

    fs::write(output_path, contents)?;
    Ok(())
}

/// Converts a stack trace represented as a vector of indices of functions in the sierra program to
/// a stack trace represented as a vector of function names.
/// Assumes that the given `idx_stack_trace` is valid with respect to the given `sierra_program`.
/// That is, each index in the stack trace is within range of the sierra program.
fn index_stack_trace_to_name_stack_trace(
    sierra_program: &Program,
    idx_stack_trace: &[usize],
) -> Vec<String> {
    idx_stack_trace.iter().map(|idx| sierra_program.funcs[*idx].id.to_string()).collect()
}
