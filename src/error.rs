use std::io::{self, Write};
use crate::ir::Type;
use crate::syntax::TokenType;
use unicode_segmentation::UnicodeSegmentation;


#[derive(Debug)]
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
    ArrayNoNewLine,
    ArrayNotPiped,
    TypeError {
        expected: Type,
        got: Type,
    },
}

#[derive(Debug)]
pub struct CompileError {
    first_char: usize,
    reason: ErrorReason,
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

    pub fn new_nesetd_macro(first_char: usize) -> Self {
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

    pub fn write_message(self, f: &mut impl Write, filename: &str, code: &str) -> io::Result<()> {
        match self.reason {
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
            ErrorReason::UndefinedVar { name } => {
                let msg = format!("Usage of undefined scope variable '{}'", name);
                error_message(f, filename, code, self.first_char, &msg)
            }
        }
    }
}

fn error_message(f: &mut impl Write, filename: &str, code: &str, first_char: usize, message: &str) -> io::Result<()> 
{
    write!(f, "{}:\n\n", filename)?;
    let mut iter = code.graphemes(true);
    let mut last_line = 0;
    let mut i = 0;

    while let Some(g) = iter.next() {
        if i == first_char {
            write!(f, "{}", g)?;

            while let Some(g) = iter.next() {
                write!(f, "{}", g)?;
                if g == "\n" || g == "\r\n" {
                    break;
                }
            }

            if first_char > 0 {
                for _ in last_line..first_char - 1 {
                    write!(f, " ")?;
                }
            }
            write!(f, "^")?;
            write!(f, "\n")?;

            if first_char > 0 {
                for _ in last_line..first_char - 1 {
                    write!(f, " ")?;
                }
            }

            write!(f, "{}", message)?;

            write!(f, "\n\n")?;

            i += 1;
            continue;
        }

        if g == "\n" || g == "\r\n" {
            last_line = i;
        }

        // add chars to buf
        write!(f, "{}", g)?;

        i += 1;
    }

    Ok(())
}

