use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn render(&self, source: &str, file_name: &str) -> String {
        let line_num = self.span.line;
        let col_num = self.span.col;

        let line_content = source.lines().nth(line_num.saturating_sub(1)).unwrap_or("");
        let gutter_width = line_num.to_string().len().max(2);
        let gutter_pad = " ".repeat(gutter_width);

        let underline_len = if self.span.end > self.span.start {
            (self.span.end - self.span.start).max(1)
        } else {
            1
        };

        let start_col = col_num.saturating_sub(1);
        let max_underline = line_content.len().saturating_sub(start_col).max(1);
        let caret_len = underline_len.min(max_underline);

        let caret = "^".repeat(caret_len);
        let padding = " ".repeat(start_col);

        let hint_text = match &self.hint {
            Some(h) => format!(" {h}"),
            None => String::new(),
        };

        format!(
            "error: {}\n  --> {}:{}:{}\n {} |\n {:>width$} | {}\n {} | {}{}{}",
            self.message,
            file_name,
            line_num,
            col_num,
            gutter_pad,
            line_num,
            line_content,
            gutter_pad,
            padding,
            caret,
            hint_text,
            width = gutter_width
        )
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}:{}: {}", self.span.line, self.span.col, self.message)
    }
}

impl std::error::Error for Diagnostic {}
