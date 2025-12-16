use unicode_segmentation::{UnicodeSegmentation, Graphemes};
use std::iter::Enumerate;
use std::collections::HashMap;
use std::fmt;
use std::cmp;
use crate::error::CompileError;


#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    Int,
    Name,
    Literal,
    CodeBegin,
    CodeEnd,
    ExprBegin,
    ExprEnd,
    Quote,
    String,
    Space,
    NewLine,
    EscapeSymbol,
    FormatSymbol,
    Range,
    RangeBegin,
    RangeSep,
    RangeEnd,
    MacroDef,
    MacroExp,
    Pipe,
}

impl Into<TokenType> for &str {
    fn into(self) -> TokenType {
        match self {
            "\n" | "\r\n" => TokenType::NewLine,
            " " | "\t" => TokenType::Space,
            "?" => TokenType::MacroExp,
            "{" => TokenType::CodeBegin,
            "}" => TokenType::CodeEnd,
            "\\" => TokenType::EscapeSymbol,
            "$" => TokenType::FormatSymbol,
            "\"" => TokenType::Quote,
            "[" => TokenType::RangeBegin,
            ":" => TokenType::RangeSep,
            "]" => TokenType::RangeEnd,
            "@" => TokenType::MacroDef,
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => TokenType::Int,
            "(" => TokenType::ExprBegin,
            ")" => TokenType::ExprEnd,
            _ => TokenType::Literal,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::NewLine => "'\\n'",
            Self::Space => "' '",
            Self::MacroExp => "'?'",
            Self::CodeBegin => "'{{'",
            Self::CodeEnd => "'}}'",
            Self::EscapeSymbol => "'\\'",
            Self::FormatSymbol => "'$'",
            Self::Quote => "'\"'",
            Self::Range => "Range",
            Self::Pipe => "'|'",
            Self::RangeBegin => "'['",
            Self::RangeSep => "':'",
            Self::RangeEnd => "']'",
            Self::MacroDef => "'@'",
            Self::Int => "Int",
            Self::Literal => "Literal",
            Self::Name => "Name",
            Self::String => "String",
            Self::ExprBegin => "'('",
            Self::ExprEnd => "')'",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub first_char: usize,
    pub end_char: usize,
}

impl Token {
    fn new(first_char: usize, end_char: usize, token_type: TokenType) -> Self {
        Self {
            token_type,
            first_char,
            end_char,
        }
    }

    pub fn as_str<'a>(&self, code: &'a str) -> &'a str {
        &code[self.first_char..self.end_char]
    }

}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub first_char: usize,
    pub end_char: usize,
    pub inner: Box<InnerNode>,
}

impl Node {
    fn new(first_char: usize, end_char: usize, inner: InnerNode) -> Self {
        Node {
            first_char,
            end_char,
            inner: Box::new(inner),
        }
    }

    pub fn as_str<'a>(&self, code: &'a str) -> &'a str {
        &code[self.first_char..self.end_char]
    }

    pub fn as_escaped_string(&self, code: &str, escape_tokens: &[TokenType]) -> String {
        EscapeIter::new(
            &code[self.first_char..self.end_char], 0, escape_tokens
        ).map(|(is_escaping, _, t)| {
            match (is_escaping, t) {
                (true, "n") => "\n",
                (true, "t") => "\t",
                (_, &_) => t,
            }
        }).collect()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum InnerNode {
    String {
        children: Vec<Node>,
    },
    Int {
        value: usize,
    },
    Range {
        start: Option<usize>,
        end: Option<usize>,
    },
    Array {
        name: Box<str>,
        start: Option<usize>,
        end: Option<usize>,
    },
    MacroDef,
    MacroExp,
    Literal,
    Name,
    Pipe,
    NewLine,
}

struct EscapeIter<'a> {
    is_escaping: bool,
    offset: usize,
    iter: Enumerate<Graphemes<'a>>,
    tokens: Vec<TokenType>,
}

impl<'a> EscapeIter<'a> {
    fn new(s: &'a str, offset: usize, tokens: &[TokenType]) -> Self {
        let iter = UnicodeSegmentation::graphemes(s, true).enumerate();

        Self {
            iter,
            offset,
            is_escaping: false,
            tokens: tokens.to_vec(),
        }
    }
}

impl<'a> Iterator for EscapeIter<'a> {
    type Item = (bool, usize, &'a str);

    fn next(&mut self) -> Option<Self::Item> {

        while let Some((i, t)) = self.iter.next() {
            let i = i + self.offset;

            if t == "\\" && !self.is_escaping {
                self.is_escaping = true;
                continue;
            }

            let esc = self.is_escaping;
            let t_type: TokenType = t.into();
            
            for token_type in &self.tokens {
                if t_type == *token_type {
                    self.is_escaping = false;
                    return Some((esc, i, t));
                }
            }

            self.is_escaping = false;
            return Some((esc, i, t));

        }

        None
    }
}

fn lex(code: &str) -> Result<Vec<Token>, CompileError> {
    let mut tokens = vec![];
    let mut literal_begin = 0;
    let mut literal_end = literal_begin;
    let mut iter = EscapeIter::new(code, 0, &[TokenType::CodeBegin, TokenType::CodeEnd]);

    if code.len() == 0 {
        return Ok(tokens);
    }

    while let Some((is_escaping, i, t)) = iter.next() {
        
        match (is_escaping, t) {
            (false, "{") => {
                // push literal
                literal_end = i;
                tokens.push(Token::new(literal_begin, literal_end, TokenType::Literal));
                // process code block
                expect_symbol(&mut iter, &[TokenType::CodeBegin], false)?;
                let begin = i + 2;
                let mut end = 0;
                // find code end
                while let Some((is_escaping, ni, t)) = iter.next() {
                    match (is_escaping, t) {
                        (false, "{") => {
                            return Err(CompileError::new_syntax(ni, &[TokenType::CodeEnd]));
                        },
                        (false, "}") => {
                            end = ni;
                            break;
                        },
                        (_, &_) => {}
                    }
                }
                expect_symbol(&mut iter, &[TokenType::CodeEnd], false)?;
                // set new literal boundary
                literal_begin = end + 2; // len("}}") - 1 == 1
                literal_end = cmp::min(literal_begin, code.len() - 1);
                // push code block
                lex_code(begin, &code[begin..end], &mut tokens)?;
            },
            (false, "}") => {
                return Err(CompileError::new_syntax(i, &[TokenType::CodeBegin]));
            }
            (true, &_) => {
                if t != "{" && t != "}" {
                    return Err(CompileError::new_syntax(i, &[TokenType::CodeBegin, TokenType::CodeEnd]));
                }
                literal_end = i;
            }
            (_, &_) => {
                literal_end = i;
            }
        }
    }
    // push last literal token
    tokens.push(Token::new(literal_begin, literal_end + 1, TokenType::Literal));

    Ok(tokens)
}

fn lex_code(first_char: usize, code: &str, tokens: &mut Vec<Token>) -> Result<(), CompileError> {
    let mut iter = EscapeIter::new(code, first_char, &[TokenType::Quote, TokenType::FormatSymbol]);

    while let Some((is_escaping, i, t)) = iter.next() {
        match (is_escaping, t) {
            (false, "@") => {
                let end = find_boundary(i, &mut iter, TokenType::Literal, &[TokenType::Space])?;
                tokens.push(Token::new(i, end, TokenType::MacroDef));
            },
            (false, "?") => {
                let end = find_symbol(&mut iter, &[TokenType::Space, TokenType::NewLine])?;
                tokens.push(Token::new(i, end, TokenType::MacroExp));
            },
            (false, "\"") => {
                let mut end = i;
                let mut found_boundary = false;
                
                while let Some((is_escaping, ni, tn)) = iter.next() {
                    end = ni;

                    match (is_escaping, tn) {
                        (false, "\"") => {
                            found_boundary = true;
                            break;
                        },
                        (_, &_) => {}
                    }
                }

                if !found_boundary {
                    return Err(CompileError::new_syntax(end, &[TokenType::Quote]));
                }

                let symbol = expect_symbol(&mut iter, &[TokenType::Space, TokenType::NewLine], false)?;

                tokens.push(Token::new(i, end + 1, TokenType::String));

                if symbol == "\n" || symbol == "\r\n" {
                    tokens.push(Token::new(end, end + 1, TokenType::NewLine));
                }

            },
            (false, "#") => {
                // don't care about the result cause it's a comment
                if let Ok(i) = find_symbol(&mut iter, &[TokenType::NewLine]) {
                    tokens.push(Token::new(i, i + 1, TokenType::NewLine));
                }
            },
            (false, "|") => {
                tokens.push(Token::new(i, i + 1, TokenType::Pipe));
            },
            (false, "0") | (false, "1") | (false, "2") | (false, "3") | (false, "4") | 
                (false, "5") | (false, "6") | (false, "7") | (false, "8") | (false, "9") => {
                let end = find_boundary(i, &mut iter, TokenType::Int, &[TokenType::Space, TokenType::NewLine])?;
                tokens.push(Token::new(i, end, TokenType::Int));
            },
            (false, "\n") | (false, "\r\n") => {
                tokens.push(Token::new(i, i + 1, TokenType::NewLine));
            }
            (false, " ") | (false, "\t")  => {},
            (false, &_) => {
                let end = find_boundary(i, &mut iter, TokenType::Literal, &[TokenType::Space, TokenType::NewLine, TokenType::RangeBegin])?;
                tokens.push(Token::new(i, end, TokenType::Name));

                let token = code.get(i - first_char..end - first_char);

                match token {
                    Some(t) => {
                        if !t.is_ascii() {
                            return Err(CompileError::new_name(i));
                        }
                    },
                    None => return Err(CompileError::new_name(i))
                }

                if end - first_char + 1 < code.len() && &code[end - first_char..end - first_char + 1] == "[" {
                    let nend = find_symbol(&mut iter, &[TokenType::RangeEnd])?;
                    tokens.push(Token::new(end, nend + 1, TokenType::Range));
                }
            },
            (true, &_) => {
                // forbid escaping
                return Err(CompileError::new_syntax(i - 1, &[]));
            }
        }
    }

    Ok(())
}

fn expect_symbol<'a>(iter: &mut impl Iterator<Item = (bool, usize, &'a str)>, expected: &[TokenType], ignore_whitespace: bool) -> Result<&'a str, CompileError>
{
    while let Some((is_escaping, i, t)) = iter.next() {
        if ignore_whitespace && (t == " " || t == "\n" || t == "\r\n") {
            continue;
        }

        let t_type: TokenType = t.into();

        for token_type in expected {
            if t_type == *token_type && is_escaping {
                return Err(CompileError::new_syntax(i - 1, expected));
            }
            else if t_type == *token_type {
                return Ok(t)
            }
        }

        return Err(CompileError::new_syntax(i, expected));
    }

    return Err(CompileError::new_syntax(0, expected));
}

fn find_symbol<'a>(iter: &mut impl Iterator<Item = (bool, usize, &'a str)>, expected: &[TokenType]) -> Result<usize, CompileError> 
{
    let mut c = 0;

    while let Some((is_escaping, i, t)) = iter.next() {
        c = i;
        let t: TokenType = t.into();

        for exp in expected {
            if t == *exp && !is_escaping {
                return Ok(c);
            }
        }
    }

    dbg!(c);
    return Err(CompileError::new_syntax(c, expected));
}

fn find_boundary<'a>(first_char: usize, iter: &mut impl Iterator<Item = (bool, usize, &'a str)>, expected: TokenType, terminator: &[TokenType]) -> Result<usize, CompileError> 
{
    let mut c = first_char;

    while let Some((is_escaping, i, t)) = iter.next() {
        c = i;
        let t: TokenType = if is_escaping { TokenType::Literal } else { t.into() };

        if t != expected {

            for term in terminator {
                if t == *term {
                    return Ok(c);
                }
            }

            return Err(CompileError::new_syntax(i, terminator));
        }
    }

    Ok(c + 1)
}

fn parse_string(first_char: usize, end_char: usize, string: &str) -> Result<Node, CompileError> {
    let mut first_literal = first_char + 1;
    let mut end_literal = 0;
    let mut nodes = vec![];
    let mut iter = EscapeIter::new(string, first_char, &[TokenType::Quote, TokenType::FormatSymbol, TokenType::ExprBegin, TokenType::ExprEnd]);

    while let Some((is_escaping, i, t)) = iter.next() {
        match (is_escaping, t) {
            (false, "$") => {
                if i - first_literal != 0 {
                    nodes.push(Node::new(first_literal, i, InnerNode::Literal));
                } 
                expect_symbol(&mut iter, &[TokenType::ExprBegin], false)?;
                expect_symbol(&mut iter, &[TokenType::Literal], false)?;
                first_literal = find_boundary(i, &mut iter, TokenType::Literal, &[TokenType::ExprEnd])?;
                nodes.push(Node::new(i + 2, first_literal, InnerNode::Name));
                first_literal += 1;
                end_literal = first_literal;
            },
            (_, &_) => {
                end_literal = i;
            }
        }
    }

    if end_literal - first_literal != 0 {
        nodes.push(Node::new(first_literal, end_literal, InnerNode::Literal));
    } 

    let s = Node::new(first_char, end_char, InnerNode::String { children: nodes });

    Ok(s)
}

fn parse_range(first_char: usize, end_char: usize, range: &str) -> Result<Node, CompileError> {
    let mut iter = EscapeIter::new(range, first_char, &[]);
    let mut start = None;
    let mut separator = 0;
    let mut end = None;

    expect_symbol(&mut iter, &[TokenType::RangeBegin], false)?;

    match expect_symbol(&mut iter, &[TokenType::Int], false) {
        Ok(_) => {
            separator = find_boundary(0, &mut iter, TokenType::Int, &[TokenType::RangeSep])?;
            let token = &range[1..separator - first_char];
            start = Some(token.parse::<usize>().unwrap());
        },
        Err(_) => {
            separator += first_char + 1;
        }
    }

    if let Ok(_) = expect_symbol(&mut iter, &[TokenType::Int], false) {
        let boundary = find_boundary(0, &mut iter, TokenType::Int, &[TokenType::RangeEnd])?;
        let token = &range[separator - first_char + 1..boundary - first_char];
        end = Some(token.parse::<usize>().unwrap());
    }

    let n = Node::new(first_char, end_char, InnerNode::Range { start, end });

    Ok(n)
}

pub fn ast(code: &str) -> Result<Vec<Node>, CompileError> {
    let mut nodes = vec![];
    let tokens = lex(code)?;

    // parse strings and ranges
    for t in tokens {
        match t.token_type {
            TokenType::Literal => {
                nodes.push(Node::new(t.first_char, t.end_char, InnerNode::Literal));
            },
            TokenType::Int => {
                let value = t.as_str(code).parse::<usize>().unwrap();
                nodes.push(Node::new(t.first_char, t.end_char, InnerNode::Int { value }));
            },
            TokenType::Name => {
                nodes.push(Node::new(t.first_char, t.end_char, InnerNode::Name));
            },
            TokenType::Pipe => {
                nodes.push(Node::new(t.first_char, t.end_char, InnerNode::Pipe));
            },
            TokenType::String => {
                nodes.push(parse_string(t.first_char, t.end_char, t.as_str(code))?);
            },
            TokenType::Range => {
                nodes.push(parse_range(t.first_char, t.end_char, t.as_str(code))?)
            },
            TokenType::MacroDef => {
                nodes.push(Node::new(t.first_char, t.end_char, InnerNode::MacroDef));
            },
            TokenType::MacroExp => {
                nodes.push(Node::new(t.first_char, t.end_char, InnerNode::MacroExp));
            },
            TokenType::NewLine => {
                nodes.push(Node::new(t.first_char, t.end_char, InnerNode::NewLine));
            },
            _ => {}
        }
    }
    // expand macros and convert 'Name' + 'Range' to 'Array'
    let mut expanded = Vec::with_capacity(nodes.len());
    let mut iter = nodes.into_iter();
    let mut macro_table = HashMap::new();

    while let Some(n) = iter.next() {
        match *n.inner {
            InnerNode::MacroDef => {
                // check if defined
                let name = &n.as_str(code)[1..];

                if macro_table.contains_key(name) {
                    return Err(CompileError::new_macro_redefinition(n.first_char, name.to_string()));
                }
                // collect children
                let mut children = vec![];

                while let Some(n) = iter.next() {
                    match *n.inner {
                        InnerNode::NewLine => break,
                        InnerNode::MacroExp | InnerNode::MacroDef => {
                            return Err(CompileError::new_nesetd_macro(n.first_char));
                        },
                        _ => {
                            children.push(n);
                        },
                    }
                }

                if children.len() == 0 {
                    return Err(CompileError::new_empty_macro(n.first_char));
                }

                macro_table.insert(name, children);
            },
            InnerNode::MacroExp => {
                let name = &n.as_str(code)[1..];

                match macro_table.get(&name) {
                    Some(n) => {
                        expanded.extend(n.clone());
                    },
                    None => {
                        return Err(CompileError::new_undefined_macro(n.first_char, name.to_string()));
                    }
                }
            },
            InnerNode::Name => {
                if let Some(nn) = iter.next() {
                    match *nn.inner {
                        InnerNode::Range { start, end } => {
                            let inner = InnerNode::Array {
                                name: n.as_str(code).into(),
                                start,
                                end,
                            };
                            let n = Node::new(n.first_char, nn.end_char, inner);
                            expanded.push(n);
                        },
                        _ => {
                            expanded.push(n);
                            expanded.push(nn);
                        }
                    }
                }
            }
            InnerNode::NewLine => {},
            _ => expanded.push(n),
        }
    }

    Ok(expanded)
}

