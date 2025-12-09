use crate::{CompError, Input, Output, RawOutput, StaticRule};
use std::collections::HashMap;

/// # Errors out if [`read_rule`] fails on a line
pub fn parse_doc(content: &str) -> Result<crate::Context, CompError> {
    let mut ctx = crate::Context {
        rule_book: vec![],
        shell_scripts: HashMap::new(),
    };

    let mut on_file = None;
    for line in content.lines() {
        let line = line.trim();
        match on_file {
            Some((name, file)) if line == "# end" => {
                ctx.shell_scripts.insert(name, file);
                on_file = None;
            }
            Some((_, ref mut file)) => {
                file.push_str(line);
                file.push('\n');
            }
            None => {
                if let Some(name) = line.strip_prefix("# sh ") {
                    on_file = Some((String::from(name), String::new()));
                } else if !line.is_empty() {
                    let rule = read_rule(line)?;
                    ctx.rule_book.push(rule);
                }
            }
        }
    }
    Ok(ctx)
}

fn read_rule(txt: &str) -> Result<StaticRule, CompError> {
    let (input, output) = txt
        .split_once("->")
        .ok_or(CompError::ParseLineMissingArrow(txt.to_string()))?;
    let mut parsed_inputs = vec![];
    let inputs = input
        .trim()
        .strip_prefix('[')
        .ok_or(CompError::ParseLineMissingArrow(txt.to_string()))?
        .strip_suffix(']')
        .ok_or(CompError::ParseLineMissingArrow(txt.to_string()))?
        .split_whitespace();
    for input in inputs {
        let parsed = match input.trim() {
            "." => Input::Any,
            "?" => Input::Maybe,
            "*" => Input::Rest,
            otherwise => {
                if let Some(var) = otherwise.strip_prefix('$') {
                    Input::Var(var.to_string())
                } else {
                    Input::Word(otherwise.to_string())
                }
            }
        };
        parsed_inputs.push(parsed);
    }
    let (output, raw) = if output.trim() == "end" {
        (Output::End, RawOutput::Raw)
    } else {
        let (cmd, args) = output
            .trim()
            .split_once('[')
            .ok_or(CompError::ParseLineMissingOutputItem(txt.to_string()))?;
        let mut args = args
            .strip_suffix(']')
            .ok_or(CompError::ParseLineMissingOutputItem(txt.to_string()))?
            .split_whitespace();
        let (cmd, raw) = if let Some(cmd) = cmd.strip_suffix('!') {
            (cmd, RawOutput::Raw)
        } else {
            (cmd, RawOutput::NeedsFilter)
        };
        let out = match cmd {
            "sh" | "exec" => Output::Sh(String::from(args.next().unwrap())),
            "word" | "words" => Output::Word(args.map(String::from).collect()),
            "glob" => Output::Glob(String::from(args.next().unwrap())),
            _ => todo!(),
        };
        (out, raw)
    };
    Ok(StaticRule {
        inputs: parsed_inputs,
        output,
        raw,
    })
}
