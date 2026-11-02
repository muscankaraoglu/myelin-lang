//! Source span tracking and LLM-first structured diagnostics for Myelin.

use std::fmt;

/// Half-open byte offset range `[start, end)` within a [`SourceFile`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    #[inline]
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    #[inline]
    pub fn merge(self, other: Self) -> Self {
        Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

/// Loaded `.myl` source buffer with precomputed line-start byte offsets for `O(log n)` lookup.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub name: String,
    pub src: String,
    line_starts: Vec<u32>,
}

impl SourceFile {
    pub fn new(name: impl Into<String>, src: impl Into<String>) -> Self {
        let name = name.into();
        let src = src.into();
        let mut line_starts = vec![0];
        for (idx, b) in src.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push((idx + 1) as u32);
            }
        }
        Self {
            name,
            src,
            line_starts,
        }
    }

    /// Returns 1-based `(line, column)` for a byte offset.
    pub fn line_col(&self, offset: u32) -> (usize, usize) {
        let clamped = offset.min(self.src.len() as u32);
        let line_idx = match self.line_starts.binary_search(&clamped) {
            Ok(exact) => exact,
            Err(insert_pos) => insert_pos.saturating_sub(1),
        };
        let line_start = self.line_starts[line_idx] as usize;
        let col_chars = self.src[line_start..clamped as usize].chars().count();
        (line_idx + 1, col_chars + 1)
    }

    /// Returns the text of a 1-based line number without trailing CR/LF.
    pub fn line_str(&self, line: usize) -> &str {
        if line == 0 || line > self.line_starts.len() {
            return "";
        }
        let start = self.line_starts[line - 1] as usize;
        let end = self
            .line_starts
            .get(line)
            .map(|&v| v as usize)
            .unwrap_or(self.src.len());
        self.src[start..end].trim_end_matches(['\r', '\n'])
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "ERR"),
            Severity::Warning => write!(f, "WRN"),
        }
    }
}

/// Machine-actionable compiler diagnostic designed for single-pass LLM self-repair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub span: Span,
    pub kind: String,
    pub fix: String,
}

impl Diagnostic {
    pub fn error(
        code: &'static str,
        span: Span,
        kind: impl Into<String>,
        fix: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code,
            span,
            kind: kind.into(),
            fix: fix.into(),
        }
    }

    /// Formats the diagnostic in the ultra-compact `--llm` single-line wire format:
    /// `ERR E002 ledger.myl:14:3 | odd_indent spaces=3 | fix: align to a multiple of 2 spaces`
    pub fn format_llm(&self, file: &SourceFile) -> String {
        let (line, col) = file.line_col(self.span.start);
        format!(
            "{} {} {}:{}:{} | {} | fix: {}",
            self.severity, self.code, file.name, line, col, self.kind, self.fix
        )
    }

    /// Formats a human-readable view with source snippet for compiler developers.
    pub fn format_human(&self, file: &SourceFile) -> String {
        let (line, col) = file.line_col(self.span.start);
        let snippet = file.line_str(line);
        let width = ((self.span.end.saturating_sub(self.span.start)) as usize).max(1);
        let caret_pad = " ".repeat(col.saturating_sub(1));
        let carets = "^".repeat(width.min(40));
        format!(
            "[{}] {} at {}:{}:{}: {}\n  |\n{:>2}| {}\n  | {}{}\n  = fix: {}",
            self.severity,
            self.code,
            file.name,
            line,
            col,
            self.kind,
            line,
            snippet,
            caret_pad,
            carets,
            self.fix
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn computes_line_col_and_llm_wire_format() {
        let sf = SourceFile::new("ledger.myl", "#mod bank\n#fn test []\n   x = 1\n");
        let diag = Diagnostic::error(
            "E002",
            Span::new(22, 25),
            "odd_indent spaces=3",
            "align statement to 2 spaces",
        );
        assert_eq!(
            diag.format_llm(&sf),
            "ERR E002 ledger.myl:3:1 | odd_indent spaces=3 | fix: align statement to 2 spaces"
        );
    }
}
