use glob::glob;
use std::collections::HashMap;
use std::os::unix::ffi::OsStringExt;
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

#[derive(thiserror::Error, Debug)]
pub enum CompError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    GlobPat(#[from] glob::PatternError),
    #[error(transparent)]
    GlobRt(#[from] glob::GlobError),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("Inlined script {0} not found")]
    LocalScriptNotFound(String),
    #[error("Expected either graph item ([] -> []) or inline script definition")]
    ParseLineMissingArrow(String),
    #[error("Failed to find output item of line {0}")]
    ParseLineMissingOutputItem(String),
    #[error("First argument must be the .compl file to use as the completion graph")]
    MissingComplFile,
    #[error("Need a path to execute for sh[] or exec[]")]
    MissingExecutablePath,
}

impl ContextfullRule<'_> {
    /// # Errors out:
    /// * If a local script is not found with [`CompError::LocalScriptNotFound`] and the script name
    /// * If the local script fails to execute
    /// * If the result of the local script is not UTF-8
    /// * If a directory can't be searched with a glob pattern
    /// * If a glob pattern led to a file that directory entry can't be read
    /// * If the glob pattern match results to a non UTF-8 path
    /// * If an executable can't be found/executed
    /// * If an executable fails to execute
    /// * If the output of an executable is non UTF-8
    fn make(&self, ctx: &Context) -> Result<Option<Vec<String>>, CompError> {
        match &self.rule.output {
            Output::End => Ok(None),
            Output::Word(k) => Ok(Some(k.clone())),
            Output::Sh(k) => {
                let out = std::process::Command::new("bash")
                    .envs(&self.variables)
                    .arg("-c")
                    .arg(
                        ctx.shell_scripts
                            .get(k)
                            .ok_or(CompError::LocalScriptNotFound(k.clone()))?,
                    )
                    .output()?
                    .stdout;
                let words = String::from_utf8(out)?
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                Ok(Some(words))
            }
            Output::Glob(pat) => glob(pat)?
                .map(|p| {
                    let bytes = p?.into_os_string().into_vec();
                    let string = String::from_utf8(bytes)?;
                    Ok(string)
                })
                .collect::<Result<_, _>>()
                .map(Option::Some),
            Output::Exec(cmd) => {
                let out = std::process::Command::new(cmd)
                    .envs(&self.variables)
                    .args(std::env::args())
                    .output()?
                    .stdout;
                let words = String::from_utf8(out)?
                    .split_whitespace()
                    .map(String::from)
                    .collect();
                Ok(Some(words))
            }
        }
    }
    /// # Errors
    /// Errors out if the output can't be created
    pub fn words(self, ctx: &Context) -> Result<Option<Vec<String>>, CompError> {
        let Some(words) = self.make(ctx)? else {
            return Ok(None);
        };
        if let Some(last) = std::env::args().skip(1).last()
            && self.rule.raw == RawOutput::NeedsFilter
            && !self.ignore_last
        {
            Ok(Some(
                words.into_iter().filter(|w| w.starts_with(&last)).collect(),
            ))
        } else {
            Ok(Some(words))
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
    #[must_use]
    pub fn try_rule(&self, user_inputs: &[String]) -> Option<ContextfullRule<'_>> {
        let mut input_rules = self.inputs.iter();
        let mut rule = input_rules.next();
        let mut vars = HashMap::new();
        for ipt in user_inputs {
            rule = match rule {
                Some(Input::Maybe | Input::Any) => input_rules.next(),
                Some(Input::Word(txt)) if txt == ipt => input_rules.next(),
                Some(Input::Var(var_name)) => {
                    vars.insert(var_name.clone(), ipt.clone());
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
