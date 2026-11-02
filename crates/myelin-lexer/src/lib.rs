//! Single-pass indentation-aware lexer for Myelin (`.myl`).

use myelin_diag::{Diagnostic, Span};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Top-level `#` directives
    HashMod,
    HashUse,
    HashRec,
    HashSum,
    HashExt,
    HashFn,
    HashTest,

    // Control `@` tags
    AtIf,
    AtElif,
    AtEl,
    AtFor,
    AtLoop,
    AtMatch,
    AtNext,
    AtBrk,

    // Plain keywords
    KwReq,
    KwEns,
    KwTrue,
    KwFalse,
    KwNone,

    // Identifiers & Literals
    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    StrLit(String),

    // Delimiters
    LParen,
    RParen,
    LBracket,
    RBracket,

    // Punctuation & Operators
    Colon,
    Comma,
    Dot,
    Arrow,            // ->
    FatArrow,         // =>
    Eq,               // =
    EqEq,             // ==
    Bang,             // !
    BangEq,           // !=
    Question,         // ?
    QuestionQuestion, // ??
    Caret,            // ^
    Pipe,             // |
    PipePipe,         // ||
    AmpAmp,           // &&
    Plus,             // +
    Minus,            // -
    Star,             // *
    Slash,            // /
    Percent,          // %
    Lt,               // <
    Le,               // <=
    Gt,               // >
    Ge,               // >=

    // Layout
    Newline,
    Indent,
    Dedent,
    Eof,
}

impl TokenKind {
    pub fn describe(&self) -> &'static str {
        match self {
            TokenKind::HashMod => "#mod",
            TokenKind::HashUse => "#use",
            TokenKind::HashRec => "#rec",
            TokenKind::HashSum => "#sum",
            TokenKind::HashExt => "#ext",
            TokenKind::HashFn => "#fn",
            TokenKind::HashTest => "#test",
            TokenKind::AtIf => "@if",
            TokenKind::AtElif => "@elif",
            TokenKind::AtEl => "@el",
            TokenKind::AtFor => "@for",
            TokenKind::AtLoop => "@loop",
            TokenKind::AtMatch => "@match",
            TokenKind::AtNext => "@next",
            TokenKind::AtBrk => "@brk",
            TokenKind::KwReq => "req",
            TokenKind::KwEns => "ens",
            TokenKind::KwTrue => "true",
            TokenKind::KwFalse => "false",
            TokenKind::KwNone => "none",
            TokenKind::Ident(_) => "identifier",
            TokenKind::IntLit(_) => "integer",
            TokenKind::FloatLit(_) => "float",
            TokenKind::StrLit(_) => "string",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBracket => "[",
            TokenKind::RBracket => "]",
            TokenKind::Colon => ":",
            TokenKind::Comma => ",",
            TokenKind::Dot => ".",
            TokenKind::Arrow => "->",
            TokenKind::FatArrow => "=>",
            TokenKind::Eq => "=",
            TokenKind::EqEq => "==",
            TokenKind::Bang => "!",
            TokenKind::BangEq => "!=",
            TokenKind::Question => "?",
            TokenKind::QuestionQuestion => "??",
            TokenKind::Caret => "^",
            TokenKind::Pipe => "|",
            TokenKind::PipePipe => "||",
            TokenKind::AmpAmp => "&&",
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::Lt => "<",
            TokenKind::Le => "<=",
            TokenKind::Gt => ">",
            TokenKind::Ge => ">=",
            TokenKind::Newline => "newline",
            TokenKind::Indent => "indent",
            TokenKind::Dedent => "dedent",
            TokenKind::Eof => "eof",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

pub struct LexOutput {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Tokenizes a UTF-8 Myelin source string into canonical layout + lexical tokens.
pub fn tokenize(src: &str) -> LexOutput {
    let mut lexer = Lexer::new(src);
    lexer.run();
    LexOutput {
        tokens: lexer.tokens,
        diagnostics: lexer.diagnostics,
    }
}

struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    bracket_depth: usize,
    indent_stack: Vec<usize>,
    at_line_start: bool,
    emitted_on_line: bool,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
            bracket_depth: 0,
            indent_stack: vec![0],
            at_line_start: true,
            emitted_on_line: false,
            tokens: Vec::with_capacity(src.len() / 3),
            diagnostics: Vec::new(),
        }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    #[inline]
    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    fn emit(&mut self, kind: TokenKind, start: usize, end: usize) {
        let is_layout = matches!(
            kind,
            TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof
        );
        if !is_layout {
            self.emitted_on_line = true;
        }
        self.tokens
            .push(Token::new(kind, Span::new(start as u32, end as u32)));
    }

    fn skip_comment_to_eol(&mut self) {
        while let Some(b) = self.peek() {
            if b == b'\n' {
                break;
            }
            self.pos += 1;
        }
    }

    fn handle_line_start(&mut self) {
        let line_start = self.pos;
        let mut spaces = 0usize;
        let mut saw_tab = false;

        while let Some(b) = self.peek() {
            match b {
                b' ' => {
                    spaces += 1;
                    self.pos += 1;
                }
                b'\t' => {
                    saw_tab = true;
                    spaces += 2;
                    self.pos += 1;
                }
                _ => break,
            }
        }

        // Ignore blank lines and pure `--` comment lines completely.
        match (self.peek(), self.peek_at(1)) {
            (None, _) | (Some(b'\r'), _) | (Some(b'\n'), _) => {
                if self.peek() == Some(b'\r') {
                    self.pos += 1;
                }
                if self.peek() == Some(b'\n') {
                    self.pos += 1;
                }
                self.at_line_start = true;
                return;
            }
            (Some(b'-'), Some(b'-')) => {
                self.skip_comment_to_eol();
                if self.peek() == Some(b'\n') {
                    self.pos += 1;
                }
                self.at_line_start = true;
                return;
            }
            _ => {}
        }

        self.at_line_start = false;
        let indent_span = Span::new(line_start as u32, self.pos as u32);

        if saw_tab {
            self.diagnostics.push(Diagnostic::error(
                "E001",
                indent_span,
                "tab_indent",
                "replace tab characters with 2 spaces per block level",
            ));
        } else if !spaces.is_multiple_of(2) {
            self.diagnostics.push(Diagnostic::error(
                "E002",
                indent_span,
                format!("odd_indent spaces={spaces}"),
                "align statement to a multiple of 2 spaces",
            ));
        }

        let current = *self.indent_stack.last().unwrap_or(&0);
        if spaces > current {
            let mut level = current;
            while level + 2 <= spaces {
                level += 2;
                self.indent_stack.push(level);
                self.emit(TokenKind::Indent, line_start, self.pos);
            }
            if level < spaces {
                // Recovery for odd indentation
                self.indent_stack.push(spaces);
                self.emit(TokenKind::Indent, line_start, self.pos);
            }
        } else if spaces < current {
            while let Some(&top) = self.indent_stack.last() {
                if top <= spaces {
                    break;
                }
                self.indent_stack.pop();
                self.emit(TokenKind::Dedent, line_start, self.pos);
            }
            let landed = *self.indent_stack.last().unwrap_or(&0);
            if landed != spaces {
                self.diagnostics.push(Diagnostic::error(
                    "E003",
                    indent_span,
                    format!("mismatched_dedent spaces={spaces}"),
                    format!("dedent to matching outer block level ({landed} spaces)"),
                ));
            }
        }
    }

    fn run(&mut self) {
        while self.pos < self.bytes.len() {
            if self.at_line_start && self.bracket_depth == 0 {
                self.handle_line_start();
                continue;
            }

            let start = self.pos;
            let b = self.bytes[self.pos];

            match b {
                b' ' | b'\r' => {
                    self.pos += 1;
                }
                b'\t' => {
                    self.pos += 1;
                }
                b'\n' => {
                    self.pos += 1;
                    if self.bracket_depth == 0 {
                        if self.emitted_on_line {
                            self.emit(TokenKind::Newline, start, self.pos);
                            self.emitted_on_line = false;
                        }
                        self.at_line_start = true;
                    }
                }
                b'-' if self.peek_at(1) == Some(b'-') => {
                    self.skip_comment_to_eol();
                }
                b'-' if self.peek_at(1) == Some(b'>') => {
                    self.pos += 2;
                    self.emit(TokenKind::Arrow, start, self.pos);
                }
                b'-' => {
                    self.pos += 1;
                    self.emit(TokenKind::Minus, start, self.pos);
                }
                b'#' => self.lex_directive(start),
                b'@' => self.lex_control(start),
                b'"' => self.lex_string(start),
                b'0'..=b'9' => self.lex_number(start),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.lex_ident_or_kw(start),
                b'(' => {
                    self.pos += 1;
                    self.bracket_depth += 1;
                    self.emit(TokenKind::LParen, start, self.pos);
                }
                b')' => {
                    self.pos += 1;
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                    self.emit(TokenKind::RParen, start, self.pos);
                }
                b'[' => {
                    self.pos += 1;
                    self.bracket_depth += 1;
                    self.emit(TokenKind::LBracket, start, self.pos);
                }
                b']' => {
                    self.pos += 1;
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                    self.emit(TokenKind::RBracket, start, self.pos);
                }
                b':' => {
                    self.pos += 1;
                    self.emit(TokenKind::Colon, start, self.pos);
                }
                b',' => {
                    self.pos += 1;
                    self.emit(TokenKind::Comma, start, self.pos);
                }
                b'.' => {
                    self.pos += 1;
                    self.emit(TokenKind::Dot, start, self.pos);
                }
                b'=' if self.peek_at(1) == Some(b'>') => {
                    self.pos += 2;
                    self.emit(TokenKind::FatArrow, start, self.pos);
                }
                b'=' if self.peek_at(1) == Some(b'=') => {
                    self.pos += 2;
                    self.emit(TokenKind::EqEq, start, self.pos);
                }
                b'=' => {
                    self.pos += 1;
                    self.emit(TokenKind::Eq, start, self.pos);
                }
                b'!' if self.peek_at(1) == Some(b'=') => {
                    self.pos += 2;
                    self.emit(TokenKind::BangEq, start, self.pos);
                }
                b'!' => {
                    self.pos += 1;
                    self.emit(TokenKind::Bang, start, self.pos);
                }
                b'?' if self.peek_at(1) == Some(b'?') => {
                    self.pos += 2;
                    self.emit(TokenKind::QuestionQuestion, start, self.pos);
                }
                b'?' => {
                    self.pos += 1;
                    self.emit(TokenKind::Question, start, self.pos);
                }
                b'^' => {
                    self.pos += 1;
                    self.emit(TokenKind::Caret, start, self.pos);
                }
                b'|' if self.peek_at(1) == Some(b'|') => {
                    self.pos += 2;
                    self.emit(TokenKind::PipePipe, start, self.pos);
                }
                b'|' => {
                    self.pos += 1;
                    self.emit(TokenKind::Pipe, start, self.pos);
                }
                b'&' if self.peek_at(1) == Some(b'&') => {
                    self.pos += 2;
                    self.emit(TokenKind::AmpAmp, start, self.pos);
                }
                b'+' => {
                    self.pos += 1;
                    self.emit(TokenKind::Plus, start, self.pos);
                }
                b'*' => {
                    self.pos += 1;
                    self.emit(TokenKind::Star, start, self.pos);
                }
                b'/' => {
                    self.pos += 1;
                    self.emit(TokenKind::Slash, start, self.pos);
                }
                b'%' => {
                    self.pos += 1;
                    self.emit(TokenKind::Percent, start, self.pos);
                }
                b'<' if self.peek_at(1) == Some(b'=') => {
                    self.pos += 2;
                    self.emit(TokenKind::Le, start, self.pos);
                }
                b'<' => {
                    self.pos += 1;
                    self.emit(TokenKind::Lt, start, self.pos);
                }
                b'>' if self.peek_at(1) == Some(b'=') => {
                    self.pos += 2;
                    self.emit(TokenKind::Ge, start, self.pos);
                }
                b'>' => {
                    self.pos += 1;
                    self.emit(TokenKind::Gt, start, self.pos);
                }
                b'{' | b'}' => {
                    self.pos += 1;
                    self.diagnostics.push(Diagnostic::error(
                        "E006",
                        Span::new(start as u32, self.pos as u32),
                        "curly_brace_forbidden",
                        "remove '{' / '}'; Myelin uses 2-space indentation for blocks and [...] for signatures",
                    ));
                }
                b';' => {
                    self.pos += 1;
                    self.diagnostics.push(Diagnostic::error(
                        "E007",
                        Span::new(start as u32, self.pos as u32),
                        "semicolon_forbidden",
                        "remove ';'; place each atomic statement on its own line",
                    ));
                }
                _ => {
                    // Advance one full UTF-8 char safely.
                    let ch_len = self.src[self.pos..]
                        .chars()
                        .next()
                        .map(char::len_utf8)
                        .unwrap_or(1);
                    self.pos += ch_len;
                    let bad = &self.src[start..self.pos];
                    self.diagnostics.push(Diagnostic::error(
                        "E008",
                        Span::new(start as u32, self.pos as u32),
                        format!("unexpected_char '{bad}'"),
                        "remove unrecognized symbol or check SKILL.md operator table",
                    ));
                }
            }
        }

        let end = self.bytes.len();
        if self.emitted_on_line {
            self.emit(TokenKind::Newline, end, end);
        }
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            self.emit(TokenKind::Dedent, end, end);
        }
        self.emit(TokenKind::Eof, end, end);
    }

    fn scan_word(&mut self) -> &'a str {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_alphanumeric() || b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }
        &self.src[start..self.pos]
    }

    fn lex_directive(&mut self, start: usize) {
        self.pos += 1; // '#'
        let name = self.scan_word();
        let kind = match name {
            "mod" => Some(TokenKind::HashMod),
            "use" => Some(TokenKind::HashUse),
            "rec" => Some(TokenKind::HashRec),
            "sum" => Some(TokenKind::HashSum),
            "ext" => Some(TokenKind::HashExt),
            "fn" => Some(TokenKind::HashFn),
            "test" => Some(TokenKind::HashTest),
            _ => None,
        };
        match kind {
            Some(k) => self.emit(k, start, self.pos),
            None => {
                let hint = match name {
                    "struct" | "class" | "type" => "use #rec Name [field:type ...]",
                    "enum" => "use #sum Name [Variant1 Variant2(type)]",
                    "import" => "use #use module.path",
                    "def" | "func" => "use #fn name [params -> ret] !caps",
                    _ => "valid directives are #mod, #use, #rec, #sum, #ext, #fn, #test",
                };
                self.diagnostics.push(Diagnostic::error(
                    "E004",
                    Span::new(start as u32, self.pos as u32),
                    format!("unknown_directive #{name}"),
                    hint,
                ));
            }
        }
    }

    fn lex_control(&mut self, start: usize) {
        self.pos += 1; // '@'
        let name = self.scan_word();
        let kind = match name {
            "if" => Some(TokenKind::AtIf),
            "elif" => Some(TokenKind::AtElif),
            "el" => Some(TokenKind::AtEl),
            "for" => Some(TokenKind::AtFor),
            "loop" => Some(TokenKind::AtLoop),
            "match" => Some(TokenKind::AtMatch),
            "next" => Some(TokenKind::AtNext),
            "brk" => Some(TokenKind::AtBrk),
            _ => None,
        };
        match kind {
            Some(k) => self.emit(k, start, self.pos),
            None => {
                let hint = match name {
                    "else" => "use @el instead of @else",
                    "elseif" | "elsif" => "use @elif <cond>",
                    "while" => "use @loop with '? <cond> | @brk'",
                    "continue" => "use @next instead of @continue",
                    "break" => "use @brk instead of @break",
                    _ => "valid control tags are @if, @elif, @el, @for, @loop, @match, @next, @brk",
                };
                self.diagnostics.push(Diagnostic::error(
                    "E005",
                    Span::new(start as u32, self.pos as u32),
                    format!("unknown_control @{name}"),
                    hint,
                ));
            }
        }
    }

    fn lex_ident_or_kw(&mut self, start: usize) {
        let word = self.scan_word();
        let kind = match word {
            "req" => TokenKind::KwReq,
            "ens" => TokenKind::KwEns,
            "true" => TokenKind::KwTrue,
            "false" => TokenKind::KwFalse,
            "none" => TokenKind::KwNone,
            _ => TokenKind::Ident(word.to_string()),
        };
        self.emit(kind, start, self.pos);
    }

    fn lex_string(&mut self, start: usize) {
        self.pos += 1; // opening '"'
        let mut buf = String::new();
        let mut closed = false;

        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'"' {
                self.pos += 1;
                closed = true;
                break;
            }
            if b == b'\n' {
                break;
            }
            if b == b'\\' {
                self.pos += 1;
                if let Some(esc) = self.peek() {
                    self.pos += 1;
                    match esc {
                        b'n' => buf.push('\n'),
                        b'r' => buf.push('\r'),
                        b't' => buf.push('\t'),
                        b'\\' => buf.push('\\'),
                        b'"' => buf.push('"'),
                        other => buf.push(other as char),
                    }
                }
                continue;
            }
            let ch = self.src[self.pos..].chars().next().unwrap();
            self.pos += ch.len_utf8();
            buf.push(ch);
        }

        if !closed {
            self.diagnostics.push(Diagnostic::error(
                "E009",
                Span::new(start as u32, self.pos as u32),
                "unterminated_string",
                "add closing '\"' before end of line",
            ));
        }
        self.emit(TokenKind::StrLit(buf), start, self.pos);
    }

    fn lex_number(&mut self, start: usize) {
        if self.peek() == Some(b'0') && matches!(self.peek_at(1), Some(b'x' | b'X')) {
            self.pos += 2;
            let hex_start = self.pos;
            while let Some(b) = self.peek() {
                if b.is_ascii_hexdigit() || b == b'_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
            let clean: String = self.src[hex_start..self.pos]
                .chars()
                .filter(|&c| c != '_')
                .collect();
            match i64::from_str_radix(&clean, 16) {
                Ok(v) => self.emit(TokenKind::IntLit(v), start, self.pos),
                Err(_) => self.diagnostics.push(Diagnostic::error(
                    "E010",
                    Span::new(start as u32, self.pos as u32),
                    "invalid_hex_literal",
                    "provide valid 64-bit hex digits after 0x",
                )),
            }
            return;
        }

        while let Some(b) = self.peek() {
            if b.is_ascii_digit() || b == b'_' {
                self.pos += 1;
            } else {
                break;
            }
        }

        let mut is_float = false;
        if self.peek() == Some(b'.') && self.peek_at(1).map(|c| c.is_ascii_digit()).unwrap_or(false)
        {
            is_float = true;
            self.pos += 1;
            while let Some(b) = self.peek() {
                if b.is_ascii_digit() || b == b'_' {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }

        let clean: String = self.src[start..self.pos]
            .chars()
            .filter(|&c| c != '_')
            .collect();

        if is_float {
            match clean.parse::<f64>() {
                Ok(v) => self.emit(TokenKind::FloatLit(v), start, self.pos),
                Err(_) => self.diagnostics.push(Diagnostic::error(
                    "E010",
                    Span::new(start as u32, self.pos as u32),
                    "invalid_float_literal",
                    "write a valid 64-bit floating-point number",
                )),
            }
        } else {
            match clean.parse::<i64>() {
                Ok(v) => self.emit(TokenKind::IntLit(v), start, self.pos),
                Err(_) => self.diagnostics.push(Diagnostic::error(
                    "E010",
                    Span::new(start as u32, self.pos as u32),
                    "invalid_int_literal",
                    "integer exceeds i64 bounds",
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthesizes_indent_and_dedent_tokens() {
        let src = "#fn f [x:i64 -> i64]\n  @if x > 0\n    = x\n  = 0\n";
        let out = tokenize(src);
        assert!(out.diagnostics.is_empty(), "{:?}", out.diagnostics);

        let kinds: Vec<_> = out.tokens.into_iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::HashFn,
                TokenKind::Ident("f".into()),
                TokenKind::LBracket,
                TokenKind::Ident("x".into()),
                TokenKind::Colon,
                TokenKind::Ident("i64".into()),
                TokenKind::Arrow,
                TokenKind::Ident("i64".into()),
                TokenKind::RBracket,
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::AtIf,
                TokenKind::Ident("x".into()),
                TokenKind::Gt,
                TokenKind::IntLit(0),
                TokenKind::Newline,
                TokenKind::Indent,
                TokenKind::Eq,
                TokenKind::Ident("x".into()),
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Eq,
                TokenKind::IntLit(0),
                TokenKind::Newline,
                TokenKind::Dedent,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn catches_curly_braces_and_odd_indent_with_llm_hints() {
        let out = tokenize("#fn bad [] {\n   x = 1\n}\n");
        assert!(out.diagnostics.iter().any(|d| d.code == "E006"));
        assert!(out.diagnostics.iter().any(|d| d.code == "E002"));
    }
}
