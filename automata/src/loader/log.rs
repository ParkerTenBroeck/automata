use std::fmt::Display;

use crate::loader::Span;

pub struct Logs {
    logs: Vec<LogEntry>,
    has_error: bool,
}

impl Logs {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            has_error: false,
        }
    }

    pub fn contains_errors(&self) -> bool {
        self.has_error
    }

    pub fn emit(&mut self, entry: LogEntry) {
        self.has_error |= matches!(entry.level, LogLevel::Error);
        self.logs.push(entry);
    }

    pub fn emit_error_locless(&mut self, msg: impl Into<String>) {
        self.emit(LogEntry {
            message: msg.into(),
            span: None,
            level: LogLevel::Error,
        });
    }

    pub fn emit_error(&mut self, msg: impl Into<String>, span: Span) {
        self.emit(LogEntry {
            message: msg.into(),
            span: Some(span),
            level: LogLevel::Error,
        });
    }

    pub fn emit_warning(&mut self, msg: impl Into<String>, span: Span) {
        self.emit(LogEntry {
            message: msg.into(),
            span: Some(span),
            level: LogLevel::Warning,
        });
    }

    pub fn emit_warning_locless(&mut self, msg: impl Into<String>) {
        self.emit(LogEntry {
            message: msg.into(),
            span: None,
            level: LogLevel::Warning,
        });
    }

    pub fn emit_info(&mut self, msg: impl Into<String>, span: Span) {
        self.emit(LogEntry {
            message: msg.into(),
            span: Some(span),
            level: LogLevel::Info,
        });
    }

    pub fn displayable_with<'a>(
        &'a self,
        src: &'a str,
    ) -> impl Iterator<Item = LogEntryDisplay<'a>> {
        self.logs.iter().map(|entry| LogEntryDisplay { src, entry })
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.logs
    }

    pub fn into_entries(self) -> impl Iterator<Item = LogEntry> {
        self.logs.into_iter()
    }
}

impl Default for Logs {
    fn default() -> Self {
        Self::new()
    }
}

pub enum LogLevel {
    Info,
    Warning,
    Error,
}

pub struct LogEntry {
    pub message: String,
    pub span: Option<Span>,
    pub level: LogLevel,
}

pub struct LogEntryDisplay<'a> {
    src: &'a str,
    entry: &'a LogEntry,
}

impl<'a> Display for LogEntryDisplay<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        pub const RESET: &str = "\x1b[0;22m";
        pub const BOLD: &str = "\x1b[1m";
        // pub const UNDERLINE: &str = "\x1b[4m";
        pub const RED: &str = "\x1b[31m";
        // pub const GREEN: &str = "\x1b[32m";
        pub const YELLOW: &str = "\x1b[33m";
        // pub const BLUE: &str = "\x1b[34m";
        pub const CYAN: &str = "\x1b[36m";

        match self.entry.level {
            LogLevel::Info => write!(f, "{BOLD}{CYAN}info{RESET}{BOLD}: ")?,
            LogLevel::Warning => write!(f, "{BOLD}{YELLOW}warning{RESET}{BOLD}: ")?,
            LogLevel::Error => write!(f, "{BOLD}{RED}error{RESET}{BOLD}: ")?,
        }
        writeln!(f, "{}{RESET}", self.entry.message)?;

        if let Some(span) = self.entry.span {
            let line_start = self.src.get(..=span.0).unwrap_or("").lines().count();
            let line_end = self.src.get(..span.1).unwrap_or("").lines().count();

            let padding = if line_end == 0 {1} else {line_end.ilog10() as usize};

            let start = self
                .src
                .get(..span.0)
                .and_then(|s| s.rfind('\n'))
                .map(|v| v + 1)
                .unwrap_or(0);

            let end = if self.src.get(..span.1).unwrap_or("").ends_with("\n") {
                span.1
            } else {
                self.src
                    .get(span.1..)
                    .and_then(|s| s.find('\n'))
                    .map(|v| v + span.1)
                    .unwrap_or(self.src.len())
            };

            let mut index = start;
            for (i, line) in self
                .src
                .get(start..end)
                .unwrap_or("")
                .split_inclusive("\n")
                .enumerate()
            {
                write!(f, "{BOLD}{CYAN}{:>padding$}: {RESET}", i + line_start)?;
                for char in line.chars() {
                    if char == '\t' {
                        write!(f, " ")?
                    } else {
                        write!(f, "{char}")?
                    }
                }
                if !line.ends_with("\n") {
                    writeln!(f)?;
                }
                write!(f, "{BOLD}{CYAN}")?;
                for _ in 0..padding + 3 {
                    write!(f, " ")?;
                }
                for char in line.chars() {
                    if (span.0..span.1).contains(&index) {
                        write!(f, "~")?;
                    } else {
                        write!(f, " ")?;
                    }
                    index += char.len_utf8();
                }
                write!(f, "{RESET}")?;
                index += '\n'.len_utf8();
                writeln!(f)?;
            }
        }

        Ok(())
    }
}
