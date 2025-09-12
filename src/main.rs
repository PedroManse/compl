use compl::*;
use std::collections::HashMap;

fn main() {
    let tmpl = Context {
        infile_scripts: HashMap::from([(
            "list_nix_files".to_string(),
            InfileScript::Sh(
                " for f in /home/manse/dots/nix/programs/*.nix ; do basename ${f%.nix} ; done "
                    .to_lowercase(),
            ),
        )]),
        rule_book: vec![
            StaticRule {
                inputs: vec![
                    Input::Word("progs".to_string()),
                    Input::Var("prog".to_string()),
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
            StaticRule {
                inputs: vec![Input::Any],
                output: Output::Word(vec![
                    "sys".to_string(),
                    "home".to_string(),
                    "progs".to_string(),
                ]),
            },
        ],
    };

    let input: Vec<_> = std::env::args().skip(1).collect();
    let rule = tmpl.rule_book.iter().find_map(|r| r.try_rule(&input));
    let last = std::env::args().last().unwrap_or("".to_string());
    print!("{input:?} -> {rule:?}");
    if let Some(ctx) = rule {
        print!(" -> ");
        let words = ctx.make(&tmpl.infile_scripts);
        println!("{last}");
        println!("{:?}", words.into_iter().filter(|w|w.starts_with(&last)).collect::<Vec<_>>());
    } else {
        print!("\n");
    }
}
