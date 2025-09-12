use std::collections::HashMap;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub enum Input {
    Word(String), // text
    Any,          // *
    Var(String),  // ${name}
    Rest,         // $@
}

#[derive(Debug)]
pub enum Output {
    StckRaw(String),
    Sh(String),
    ShRaw(String),
    Exec(String),
    ExecRaw(String),
    Word(Vec<String>),
    End,
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
