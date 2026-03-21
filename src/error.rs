use std::io::{self, Write};
use crate::ir::Type;
use crate::syntax::{TokenType, EscapeIter};
use unicode_segmentation::UnicodeSegmentation;


#[derive(Debug, PartialEq)]
pub enum ErrorReason {
    SyntaxError {
        expected: Vec<TokenType>,
    },
    NameError,
    MacroRedefinition {
        name: String
    },
    UndefinedMacro {
        name: String
    },
    UndefinedVar {
        name: String
    },
    NestedMacro,
    EmptyMacro,
    PipeNoParent,
    PipeNoChildren, 
    ArrayNoNewLine,
    ArrayNotPiped,
    TypeError {
        expected: Type,
        got: Type,
    },
}

#[derive(Debug, PartialEq)]
pub struct CompileError {
    pub first_char: usize,
    pub reason: ErrorReason,
}

impl CompileError {
    pub fn new_syntax(first_char: usize, expected: &[TokenType]) -> Self {
        Self {
            first_char,
            reason: ErrorReason::SyntaxError {
                expected: expected.to_vec(),
            },
        }
    }

    pub fn new_name(first_char: usize) -> Self {
        Self {
            first_char,
            reason: ErrorReason::NameError,
        }
    }

    pub fn new_undefined_var(first_char: usize, name: String) -> Self {
        Self {
            first_char,
            reason: ErrorReason::UndefinedVar { name },
        }
    }

    pub fn new_array_pipe(first_char: usize) -> Self {
        Self {
            first_char,
            reason: ErrorReason::ArrayNotPiped,
        }
    }

    pub fn new_pipe_no_parent(first_char: usize) -> Self {
        Self {
            first_char,
            reason: ErrorReason::PipeNoParent,
        }
    }

    pub fn new_pipe_no_children(first_char: usize) -> Self {
        Self {
            first_char,
            reason: ErrorReason::PipeNoChildren,
        }
    }

    pub fn new_invalid_array(first_char: usize) -> Self {
        Self {
            first_char,
            reason: ErrorReason::ArrayNoNewLine,
        }
    }

    pub fn new_macro_redefinition(first_char: usize, name: String) -> Self {
        Self {
            first_char,
            reason: ErrorReason::MacroRedefinition {
                name,
            },
        }
    }

    pub fn new_undefined_macro(first_char: usize, name: String) -> Self {
        Self {
            first_char,
            reason: ErrorReason::UndefinedMacro {
                name,
            },
        }
    }

    pub fn new_nested_macro(first_char: usize) -> Self {
        Self {
            first_char,
            reason: ErrorReason::NestedMacro,
        }
    }

    pub fn new_empty_macro(first_char: usize) -> Self {
        Self {
            first_char,
            reason: ErrorReason::EmptyMacro,
        }
    }

    pub fn new_type_error(first_char: usize, expected: Type, got: Type) -> Self {
        Self {
            first_char,
            reason: ErrorReason::TypeError {
                expected,
                got,
            }
        }
    }

    pub fn write_message(&self, f: &mut impl Write, filename: &str, code: &str) -> io::Result<()> {
        match &self.reason {
            ErrorReason::SyntaxError { expected } => {
                if expected.len() > 0 {
                    let expected = expected.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(", ");
                    let msg = format!("Expected: {}", expected);
                    error_message(f, filename, code, self.first_char, &msg)
                } else {
                    error_message(f, filename, code, self.first_char, "Unexpected token")
                }
            },
            ErrorReason::NameError => {
                error_message(f, filename, code, self.first_char, "Only alphabetic ascii-chars can be used for names")
            },
            ErrorReason::MacroRedefinition { name } => {
                let msg = format!("Redefinition of '{}'. Macros cannot be redefined", name);
                error_message(f, filename, code, self.first_char, &msg)
            },
            ErrorReason::UndefinedMacro { name } => {
                let msg = format!("Usage of undefined macro '{}'", name);
                error_message(f, filename, code, self.first_char, &msg)
            },
            ErrorReason::NestedMacro => {
                error_message(f, filename, code, self.first_char, "Macros cannot be nested")
            },
            ErrorReason::EmptyMacro => {
                error_message(f, filename, code, self.first_char, "Macros cannot be empty")
            },
            ErrorReason::TypeError { expected, got } => {
                let msg = format!("Expected type {} but got {}", expected, got);
                error_message(f, filename, code, self.first_char, &msg)
            },
            ErrorReason::ArrayNotPiped => {
                error_message(f, filename, code, self.first_char, "Arrays must be piped")
            },
            ErrorReason::ArrayNoNewLine => {
                error_message(f, filename, code, self.first_char, "Array definitions must start with a newline")
            },
            ErrorReason::PipeNoParent => {
                error_message(f, filename, code, self.first_char, "Pipe has no parent")
            },
            ErrorReason::PipeNoChildren => {
                error_message(f, filename, code, self.first_char, "Pipe has no children")
            },
            ErrorReason::UndefinedVar { name } => {
                let msg = format!("Usage of undefined scope variable '{}'", name);
                error_message(f, filename, code, self.first_char, &msg)
            }
        }
    }
}

fn error_message(f: &mut impl Write, filename: &str, code: &str, first_char: usize, message: &str) -> io::Result<()> 
{
    let mut line = 1;
    let mut line_start = 0;
    let mut output = String::new();
    let mut iter = EscapeIter::new(code, 0, &[TokenType::CodeBegin, TokenType::CodeEnd]);

    while let Some((_, i, g)) = iter.next() {
        output.push_str(g);

        if i == first_char {
            // go to the end of the line
            if g != "\n" && "g" != "\r\n" {
                while let Some((_, _, g)) = iter.next() {

                    if g == "\n" || g == "\r\n" {
                        break;
                    }

                    output.push_str(g);
                }
            }

            // write header
            write!(f, "{}:{}:{}\n", filename, line, i - std::cmp::min(i, line_start))?;

            // write the line
            for g in UnicodeSegmentation::graphemes(output.as_str(), true) {
                match g {
                    "\n" => write!(f, " ")?,
                    _ => write!(f, "{}", g)?
                }
            }

            write!(f, "\n")?;
            
            // write error message aligned to the line
            let iter = EscapeIter::new(&output, line_start + 1, &[TokenType::CodeBegin, TokenType::CodeEnd]);
            let mut offset_str = String::new();
            
            // find the offset string
            for (_, i, g) in iter {
                match g {
                    " " | "\t" => offset_str.push_str(g),
                    _ => offset_str.push_str(" ")
                }

                if i == first_char {
                    break;
                }
            }

            // write message
            write!(f, "{}", offset_str)?;
            write!(f, "^")?;


            write!(f, "\n{}{}\n", offset_str, message)?;
        }

        if g == "\n" || g == "\r\n" {
            line_start = i + 1;
            line += 1;
            output.clear();
        }

    }

    Ok(())
}

