use glob::glob;
use std::collections::HashMap;
use std::ops::Deref;
pub mod read;

#[derive(Clone, Debug)]
pub enum Input {
    Word(String), // text
    Any,          // .
    Maybe,        // ?
    Var(String),  // ${name}
    Rest,         // *
}

#[derive(Debug, PartialEq)]
pub enum RawOutput {
    Raw,
    NeedsFilter,
}

#[derive(Debug)]
pub enum Output {
    Sh(String),        // sh[ infile ]
    Glob(String),      // glob[ pattern ]
    Exec(String),      // exec[ file ]
    Word(Vec<String>), // word[ word list ]
    End,               // end
}

impl<'r> ContextfullRule<'r> {
    fn make(&self, ctx: &Context) -> Vec<String> {
        match &self.rule.output {
            Output::End => vec![],
            Output::Word(k) => k.clone(),
            Output::Sh(k) => {
                let out = std::process::Command::new("bash")
                    .envs(&self.variables)
                    .arg("-c")
                    .arg(ctx.shell_scripts.get(k).unwrap())
                    .output()
                    .unwrap()
                    .stdout;
                String::from_utf8(out)
                    .unwrap()
                    .split_whitespace()
                    .map(String::from)
                    .collect()
            }
            Output::Glob(pat) => glob(pat)
                .unwrap()
                .map(|p| p.unwrap().into_os_string().into_string().unwrap())
                .collect(),
            Output::Exec(_file) => {
                todo!()
            }
        }
    }
    pub fn words(self, ctx: &Context) -> Vec<String> {
        let last = std::env::args().skip(1).last().unwrap_or("".to_string());
        let words = self.make(ctx);
        if self.ignore_last || self.raw == RawOutput::Raw {
            words
        } else {
            words.into_iter().filter(|w| w.starts_with(&last)).collect()
        }
    }
}

#[derive(Debug)]
pub struct StaticRule {
    pub inputs: Vec<Input>,
    pub output: Output,
    pub raw: RawOutput,
}

impl StaticRule {
    pub fn try_rule(&self, user_inputs: &[String]) -> Option<ContextfullRule<'_>> {
        let mut input_rules = self.inputs.clone().into_iter();
        let mut rule = input_rules.next();
        let mut vars = HashMap::new();
        for ipt in user_inputs {
            rule = match rule {
                Some(Input::Maybe) => input_rules.next(),
                Some(Input::Word(txt)) if txt == *ipt => input_rules.next(),
                Some(Input::Any) => input_rules.next(),
                Some(Input::Var(var_name)) => {
                    vars.insert(var_name.to_string(), ipt.to_string());
                    input_rules.next()
                }
                Some(Input::Rest) => rule,
                None | Some(Input::Word(_)) => {
                    return None;
                }
            };
        }
        match rule {
            Some(Input::Maybe) => Some(ContextfullRule {
                rule: self,
                variables: vars,
                ignore_last: true,
            }),
            None => Some(ContextfullRule {
                rule: self,
                variables: vars,
                ignore_last: false,
            }),
            Some(_) => None,
        }
    }
}

#[derive(Debug)]
pub struct ContextfullRule<'r> {
    pub rule: &'r StaticRule,
    pub variables: HashMap<String, String>,
    pub ignore_last: bool,
}

#[derive(Debug)]
pub struct Context {
    pub rule_book: Vec<StaticRule>,
    pub shell_scripts: HashMap<String, String>,
}

impl<'r> Deref for ContextfullRule<'r> {
    type Target = StaticRule;
    fn deref(&self) -> &Self::Target {
        self.rule
    }
}
