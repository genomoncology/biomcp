#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NextCommand {
    program: String,
    args: Vec<String>,
}

impl NextCommand {
    pub(crate) fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub(crate) fn biomcp() -> Self {
        Self::new("biomcp")
    }

    pub(crate) fn arg(mut self, value: impl Into<String>) -> Self {
        self.args.push(value.into());
        self
    }

    pub(crate) fn args<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(values.into_iter().map(Into::into));
        self
    }

    pub(crate) fn render_shell(&self) -> String {
        std::iter::once(shell_quote_arg(&self.program))
            .chain(self.args.iter().map(|arg| shell_quote_arg(arg)))
            .filter(|arg| !arg.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub(crate) fn shell_quote_arg(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let needs_quotes = trimmed.chars().any(|ch| {
        ch.is_whitespace()
            || matches!(
                ch,
                '"' | '\''
                    | '\\'
                    | '$'
                    | '`'
                    | '|'
                    | '&'
                    | ';'
                    | '<'
                    | '>'
                    | '('
                    | ')'
                    | '['
                    | ']'
                    | '{'
                    | '}'
                    | '*'
                    | '?'
                    | '!'
                    | '#'
            )
    });
    if needs_quotes {
        let escaped = trimmed.chars().fold(String::new(), |mut out, ch| {
            if matches!(ch, '\\' | '"' | '$' | '`') {
                out.push('\\');
            }
            out.push(ch);
            out
        });
        return format!("\"{escaped}\"");
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_each_argument_as_its_own_shell_word() {
        let command = NextCommand::biomcp()
            .args(["search", "article", "-k"])
            .arg("NM_000248.3:c.1799T>A")
            .args(["--type", "review", "--limit", "5"]);

        assert_eq!(
            command.render_shell(),
            "biomcp search article -k \"NM_000248.3:c.1799T>A\" --type review --limit 5"
        );
    }
}
