use std::borrow::Cow;

use once_cell::sync::Lazy;
use regex::Regex;
use rustpython_ast::Location;
use rustpython_parser::lexer::{LexResult, Tok};

use crate::ast::types::Range;
use crate::source_code_locator::SourceCodeLocator;

fn build_line(tokens: &[(&Location, &Tok, &Location)], locator: &SourceCodeLocator) -> String {
    let mut logical = String::new();
    let mut mapping = Vec::new();
    let mut prev: Option<&Location> = None;
    for (start, tok, end) in tokens {
        if matches!(tok, Tok::Newline | Tok::Indent | Tok::Dedent | Tok::Comment) {
            continue;
        }
        if mapping.is_empty() {
            mapping.push((0, start));
        }

        // TODO(charlie): "Mute" strings.
        let text = if let Tok::String { .. } = tok {
            Cow::from("\"\"")
        } else {
            locator.slice_source_code_range(&Range {
                location: **start,
                end_location: **end,
            })
        };

        if let Some(prev) = prev {
            if prev.row() != start.row() {
                let prev_text = locator.slice_source_code_range(&Range {
                    location: *prev,
                    end_location: Location::new(prev.row() + 1, 0),
                });
                if prev_text == ","
                    || ((prev_text != "{" && prev_text != "[" && prev_text != "(")
                        && (text != "}" || text != "]" || text != ")"))
                {
                    logical.push(' ');
                }
            } else if prev.column() != start.column() {
                let prev_text = locator.slice_source_code_range(&Range {
                    location: *prev,
                    end_location: **start,
                });
                logical.push_str(&prev_text);
            }
        }
        logical.push_str(&text);
        mapping.push((text.len(), end));
        prev = Some(end);
    }
    logical
}

pub fn logical_lines(tokens: &[LexResult], locator: &SourceCodeLocator) -> Vec<String> {
    let mut parens = 0;
    let mut accumulator = vec![];
    let mut lines = vec![];
    for (start, tok, end) in tokens.iter().flatten() {
        accumulator.push((start, tok, end));
        if matches!(tok, Tok::Lbrace | Tok::Lpar | Tok::Lsqb) {
            parens += 1;
        } else if matches!(tok, Tok::Rbrace | Tok::Rpar | Tok::Rsqb) {
            parens -= 1;
        } else if parens == 0 {
            if matches!(tok, Tok::Newline) {
                lines.push(build_line(&accumulator, locator));
                accumulator.drain(..);
            }
        }
    }
    lines
}

static OPERATOR_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[^,\s](\s*)(?:[-+*/|!<=>%&^]+|:=)(\s*)").unwrap());

pub fn check_logical_lines(tokens: &[LexResult], locator: &SourceCodeLocator) {
    for line in logical_lines(tokens, locator) {
        for line_match in OPERATOR_REGEX.captures_iter(&line) {
            let before = line_match.get(1).unwrap().as_str();
            let after = line_match.get(2).unwrap().as_str();

            if before.contains('\t') {
                println!("E223 tab before operator: {line:?}");
            } else if before.len() > 1 {
                println!("E221 multiple spaces before operator: {line:?}");
            }

            if after.contains('\t') {
                println!("E224 tab after operator: {line:?}");
            } else if after.len() > 1 {
                println!("E224 multiple spaces after operator: {line:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rustpython_parser::lexer;
    use rustpython_parser::lexer::LexResult;

    use crate::check_logical_lines::{check_logical_lines, logical_lines};
    use crate::SourceCodeLocator;

    #[test]
    fn test_logical_lines() {
        let contents = "a = 12 + 3";
        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        check_logical_lines(&lxr, &locator);

        let contents = "a = 4 +  5";
        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        check_logical_lines(&lxr, &locator);

        let contents = "a = 4  + 5";
        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        check_logical_lines(&lxr, &locator);

        let contents = "a = 4\t + 5";
        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        check_logical_lines(&lxr, &locator);

        let contents = "a = 4 + \t5";
        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        check_logical_lines(&lxr, &locator);
    }

    #[test]
    fn split_logical_lines() {
        let contents = "x = 1
y = 2
z = x + 1";
        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        println!("{:?}", logical_lines(&lxr, &locator));

        let contents = "x = [
  1,
  2,
  3,
]
y = 2
z = x + 1";
        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        println!("{:?}", logical_lines(&lxr, &locator));

        let contents = "x = 'abc'";
        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        println!("{:?}", logical_lines(&lxr, &locator));

        let contents = "def f():
  x = 1

f()";

        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        println!("{:?}", logical_lines(&lxr, &locator));

        let contents = r#"def f():
  """Docstring goes here."""
  # Comment goes here.
  x = 1

f()"#;

        let lxr: Vec<LexResult> = lexer::make_tokenizer(contents).collect();
        let locator = SourceCodeLocator::new(contents);
        println!("{:?}", logical_lines(&lxr, &locator));
    }
}
