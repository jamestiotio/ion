use super::functions::{collect_arguments, parse_function};
use super::super::{ArgumentSplitter, pipelines};
use super::super::pipelines::Pipeline;
use shell::flow_control::{Case, ElseIf, Statement};
use std::char;

fn collect<F>(arguments: &str, statement: F) -> Statement
    where F: Fn(Pipeline) -> Statement
{
    match pipelines::Collector::run(arguments) {
        Ok(pipeline) => statement(pipeline),
        Err(err) => {
            eprintln!("ion: syntax error: {}", err);
            return Statement::Default;
        }
    }
}

fn is_valid_name(name: &str) -> bool { !name.chars().any(|c| !(c.is_alphanumeric() || c == '_')) }

pub fn parse(code: &str) -> Statement {
    let cmd = code.trim();
    match cmd {
        "end" => return Statement::End,
        "break" => return Statement::Break,
        "continue" => return Statement::Continue,
        "for" | "match" | "case" => {
            eprintln!("ion: syntax error: incomplete control flow statement");
            return Statement::Default;
        }
        _ if cmd.starts_with("let ") => return Statement::Let { expression: cmd[4..].trim_left().into() },
        _ if cmd.starts_with("export ") => return Statement::Export(cmd[7..].trim_left().into()),
        _ if cmd.starts_with("if ") => {
            return collect(cmd[3..].trim_left(), |pipeline| {
                Statement::If {
                    expression: pipeline,
                    success: Vec::new(),
                    else_if: Vec::new(),
                    failure: Vec::new(),
                }
            });
        }
        "else" => return Statement::Else,
        _ if cmd.starts_with("else") => {
            let cmd = cmd[4..].trim_left();
            if cmd.len() == 0 {
                return Statement::Else;
            } else if cmd.starts_with("if ") {
                return collect(cmd[3..].trim_left(), |pipeline| {
                    Statement::ElseIf(ElseIf {
                        expression: pipeline,
                        success: Vec::new(),
                    })
                });
            }
        }
        _ if cmd.starts_with("while ") => {
            return collect(cmd[6..].trim_left(), |pipeline| {
                Statement::While {
                    expression: pipeline,
                    statements: Vec::new(),
                }
            });
        }
        _ if cmd.starts_with("for ") => {
            let mut cmd = cmd[4..].trim_left();
            let pos = match cmd.find(char::is_whitespace) {
                Some(pos) => pos,
                None => {
                    eprintln!("ion: syntax error: incorrect for loop syntax");
                    return Statement::Default;
                }
            };

            let variable = &cmd[..pos];
            cmd = &cmd[pos..].trim_left();

            if !cmd.starts_with("in ") {
                eprintln!("ion: syntax error: incorrect for loop syntax");
                return Statement::Default;
            }

            return Statement::For {
                variable: variable.into(),
                values: ArgumentSplitter::new(cmd[3..].trim_left())
                    .map(String::from)
                    .collect(),
                statements: Vec::new(),
            };
        }
        _ if cmd.starts_with("case ") => {
            let value = match cmd[5..].trim_left() {
                "_" => None,
                value @ _ => Some(value.into()),
            };
            return Statement::Case(Case {
                value: value,
                statements: Vec::new(),
            });
        }
        _ if cmd.starts_with("match ") => {
            return Statement::Match {
                expression: cmd[6..].trim_left().into(),
                cases: Vec::new(),
            };
        }
        _ if cmd.starts_with("fn ") => {
            let cmd = cmd[3..].trim_left();
            let pos = cmd.find(char::is_whitespace).unwrap_or(cmd.len());
            let name = &cmd[..pos];
            if !is_valid_name(name) {
                eprintln!(
                    "ion: syntax error: '{}' is not a valid function name\n     \
                    Function names may only contain alphanumeric characters",
                    name
                );
                return Statement::Default;
            }

            let (args, description) = parse_function(&cmd[pos..]);
            match collect_arguments(args) {
                Ok(args) => {
                    return Statement::Function {
                        description: description.map(String::from),
                        name: name.into(),
                        args,
                        statements: Vec::new(),
                    }
                }
                Err(why) => {
                    eprintln!("ion: function argument error: {}", why);
                    return Statement::Default;
                }
            }
        }
        _ => (),
    }


    if cmd.is_empty() || cmd.starts_with('#') {
        Statement::Default
    } else {
        collect(cmd, Statement::Pipeline)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::types::parse::{Primitive, TypeArgBuf};
    use shell::{Job, JobKind};
    use shell::flow_control::Statement;

    #[test]
    fn parsing_ifs() {
        // Default case where spaced normally
        let parsed_if = parse("if test 1 -eq 2");
        let correct_parse = Statement::If {
            expression: Pipeline::new(
                vec![
                    Job::new(
                        vec![
                            "test".to_owned(),
                            "1".to_owned(),
                            "-eq".to_owned(),
                            "2".to_owned(),
                        ].into_iter()
                            .collect(),
                        JobKind::Last
                    ),
                ],
                None,
                None,
            ),
            success: vec![],
            else_if: vec![],
            failure: vec![],
        };
        assert_eq!(correct_parse, parsed_if);

        // Trailing spaces after final value
        let parsed_if = parse("if test 1 -eq 2         ");
        assert_eq!(correct_parse, parsed_if);
    }

    #[test]
    fn parsing_elses() {
        // Default case where spaced normally
        let mut parsed_if = parse("else");
        let correct_parse = Statement::Else;
        assert_eq!(correct_parse, parsed_if);

        // Trailing spaces after final value
        parsed_if = parse("else         ");
        assert_eq!(correct_parse, parsed_if);

        // Leading spaces after final value
        parsed_if = parse("         else");
        assert_eq!(correct_parse, parsed_if);
    }

    #[test]
    fn parsing_ends() {
        // Default case where spaced normally
        let parsed_if = parse("end");
        let correct_parse = Statement::End;
        assert_eq!(correct_parse, parsed_if);

        // Trailing spaces after final value
        let parsed_if = parse("end         ");
        let correct_parse = Statement::End;
        assert_eq!(correct_parse, parsed_if);

        // Leading spaces after final value
        let parsed_if = parse("         end");
        let correct_parse = Statement::End;
        assert_eq!(correct_parse, parsed_if);
    }

    #[test]
    fn parsing_functions() {
        // Default case where spaced normally
        let parsed_if = parse("fn bob");
        let correct_parse = Statement::Function {
            description: None,
            name: "bob".into(),
            args: Default::default(),
            statements: Default::default(),
        };
        assert_eq!(correct_parse, parsed_if);

        // Trailing spaces after final value
        let parsed_if = parse("fn bob        ");
        assert_eq!(correct_parse, parsed_if);

        // Leading spaces after final value
        let parsed_if = parse("         fn bob");

        // Default case where spaced normally
        let parsed_if = parse("fn bob a b");
        let correct_parse = Statement::Function {
            description: None,
            name: "bob".into(),
            args: vec![
                TypeArgBuf {
                    name: "a".into(),
                    kind: Primitive::Any,
                },
                TypeArgBuf {
                    name: "b".into(),
                    kind: Primitive::Any,
                },
            ],
            statements: Default::default(),
        };
        assert_eq!(correct_parse, parsed_if);

        // Trailing spaces after final value
        let parsed_if = parse("fn bob a b       ");
        assert_eq!(correct_parse, parsed_if);

        let parsed_if = parse("fn bob a b --bob is a nice function");
        let correct_parse = Statement::Function {
            description: Some("bob is a nice function".to_string()),
            name: "bob".into(),
            args: vec![
                TypeArgBuf {
                    name: "a".into(),
                    kind: Primitive::Any,
                },
                TypeArgBuf {
                    name: "b".into(),
                    kind: Primitive::Any,
                },
            ],
            statements: vec![],
        };
        assert_eq!(correct_parse, parsed_if);
        let parsed_if = parse("fn bob a b --          bob is a nice function");
        assert_eq!(correct_parse, parsed_if);
        let parsed_if = parse("fn bob a b      --bob is a nice function");
        assert_eq!(correct_parse, parsed_if);
    }
}
