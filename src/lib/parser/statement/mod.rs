mod case;
mod functions;
#[cfg(not(fuzzing))]
pub mod parse;
#[cfg(fuzzing)]
pub mod parse;
mod splitter;

pub(crate) use self::parse::parse;
pub use self::splitter::{StatementError, StatementSplitter, StatementVariant};
use crate::{builtins::BuiltinMap, shell::flow_control::Statement};

/// Parses a given statement string and return's the corresponding mapped
/// `Statement`
pub(crate) fn parse_and_validate<'b>(
    statement: Result<StatementVariant, StatementError>,
    builtins: &BuiltinMap<'b>,
) -> Statement<'b> {
    match statement {
        Ok(StatementVariant::And(statement)) => {
            Statement::And(Box::new(parse(statement, builtins)))
        }
        Ok(StatementVariant::Or(statement)) => Statement::Or(Box::new(parse(statement, builtins))),
        Ok(StatementVariant::Default(statement)) => parse(statement, builtins),
        Err(err) => {
            eprintln!("ion: {}", err);
            Statement::Error(-1)
        }
    }
}

/// Splits a string into two, based on a given pattern. We know that the first string will always
/// exist, but if the pattern is not found, or no string follows the pattern, then the second
/// string will not exist. Useful for splitting the function expression by the "--" pattern.
pub(crate) fn split_pattern<'a>(arg: &'a str, pattern: &str) -> (&'a str, Option<&'a str>) {
    match arg.find(pattern) {
        Some(pos) => {
            let args = &arg[..pos].trim();
            let comment = &arg[pos + pattern.len()..].trim();
            if comment.is_empty() {
                (args, None)
            } else {
                (args, Some(comment))
            }
        }
        None => (arg, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statement_pattern_splitting() {
        let (args, description) = split_pattern("a:int b:bool -- a comment", "--");
        assert_eq!(args, "a:int b:bool");
        assert_eq!(description, Some("a comment"));

        let (args, description) = split_pattern("a --", "--");
        assert_eq!(args, "a");
        assert_eq!(description, None);

        let (args, description) = split_pattern("a", "--");
        assert_eq!(args, "a");
        assert_eq!(description, None);
    }
}
