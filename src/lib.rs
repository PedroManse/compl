use std::collections::HashMap;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum Input {
    Word(String), // text
    Any,          // ?
    Var(String),  // ${name}
    Rest,         // *
}

#[derive(Debug)]
pub enum Output {
    StckRaw(String),   // stck![ infile ]
    Sh(String),        // sh[ infile ]
    Glob(String),      // glob[ pattern ]
    ShRaw(String),     // sh![ infile ]
    Exec(String),      // exec[ file ]
    ExecRaw(String),   // exec![ file ]
    Word(Vec<String>), // word[ word list ]
    End,               // end
}

fn write_arr(vec: impl Iterator<Item = impl AsRef<str>>) {
    for txt in vec {
        println!("{}", txt.as_ref());
    }
}

impl<'r> ContextfullRule<'r> {
    pub fn make(&self, files: &HashMap<String, InfileScript>) -> Vec<String> {
        match &self.rule.output {
            Output::End => vec![],
            Output::Word(k) => k.clone(),
            Output::Sh(k) => {
                let out = std::process::Command::new("bash")
                    .envs(&self.variables)
                    .arg("-c")
                    .arg(match files.get(k){
                        Some(InfileScript::Sh(t))=>t,
                        _ => todo!()
                    })
                    .output()
                    .unwrap()
                    .stdout;
                String::from_utf8(out)
                    .unwrap()
                    .split_whitespace()
                    .map(String::from)
                    .collect()
            }
            _ => {
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub struct StaticRule {
    pub inputs: Vec<Input>,
    pub output: Output,
}

impl StaticRule {
    pub fn try_rule(&self, user_inputs: &[String]) -> Option<ContextfullRule<'_>> {
        let mut input_rules = self.inputs.clone().into_iter();
        let mut rule = input_rules.next();
        let mut vars = HashMap::new();
        for ipt in user_inputs {
            rule = match rule {
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
        match input_rules.next() {
            Some(_) => None,
            None => Some(ContextfullRule {
                rule: self,
                variables: vars,
            }),
        }
    }
}

#[derive(Debug)]
pub struct ContextfullRule<'r> {
    pub rule: &'r StaticRule,
    pub variables: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Context {
    pub rule_book: Vec<StaticRule>,
    pub infile_scripts: HashMap<String, InfileScript>,
}

#[derive(Debug)]
pub enum InfileScript {
    Sh(String),
    Stck(stck::internals::Code),
}

impl<'r> Deref for ContextfullRule<'r> {
    type Target = StaticRule;
    fn deref(&self) -> &Self::Target {
        self.rule
    }
}
