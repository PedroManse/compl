use compl::*;
use std::collections::HashMap;

fn main() {
    let tmpl = Context {
        infile_scripts: HashMap::new(),
        rule_book: vec![
            StaticRule {
                inputs: vec![Input::Any],
                output: Output::Word(vec![
                    "sys".to_string(),
                    "home".to_string(),
                    "progs".to_string(),
                ]),
            },
            StaticRule {
                inputs: vec![
                    Input::Word("progs".to_string()),
                    Input::Any,
                ],
                output: Output::Sh("list_nix_files".to_string()),
            },
            StaticRule {
                inputs: vec![Input::Word("progs".to_string()), Input::Any, Input::Any],
                output: Output::End,
            },
            StaticRule {
                inputs: vec![Input::Word("home".to_string())],
                output: Output::End,
            },
            StaticRule {
                inputs: vec![Input::Word("sys".to_string())],
                output: Output::End,
            },
        ],
    };

    let inputs = [
        vec!["pro".to_string()],
        vec!["ho".to_string()],
        vec!["progs".to_string(), "ba".to_string()],
    ];
    for input in inputs {
        let rule = tmpl.rule_book.iter().find_map(|r| r.try_rule(&input));
        println!("{input:?} -> {rule:?}");
    }

}
