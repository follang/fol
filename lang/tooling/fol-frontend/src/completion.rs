use crate::{FrontendCli, FrontendCommandResult, FrontendError, FrontendErrorKind, FrontendResult};
use clap_complete::{generate, Shell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionShell {
    Bash,
    Zsh,
    Fish,
}

impl CompletionShell {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bash => "bash",
            Self::Zsh => "zsh",
            Self::Fish => "fish",
        }
    }

    fn clap_shell(self) -> Shell {
        match self {
            Self::Bash => Shell::Bash,
            Self::Zsh => Shell::Zsh,
            Self::Fish => Shell::Fish,
        }
    }
}

pub fn generate_completion_script(shell: CompletionShell) -> FrontendResult<String> {
    let mut command = FrontendCli::command();
    let mut out = Vec::new();
    generate(shell.clap_shell(), &mut command, "fol", &mut out);
    String::from_utf8(out).map_err(|error| {
        FrontendError::new(
            FrontendErrorKind::Internal,
            format!("generated completion output was not valid UTF-8: {error}"),
        )
    })
}

pub fn completion_command(shell: CompletionShell) -> FrontendResult<FrontendCommandResult> {
    let _ = generate_completion_script(shell)?;
    Ok(FrontendCommandResult::new(
        "completion",
        format!("generated {} completion script", shell.as_str()),
    ))
}

pub fn generate_bash_completion_script() -> FrontendResult<String> {
    generate_completion_script(CompletionShell::Bash)
}

pub fn generate_zsh_completion_script() -> FrontendResult<String> {
    generate_completion_script(CompletionShell::Zsh)
}

pub fn generate_fish_completion_script() -> FrontendResult<String> {
    generate_completion_script(CompletionShell::Fish)
}

pub fn internal_complete_command() -> FrontendResult<FrontendCommandResult> {
    internal_complete_command_with_tokens(&[])
}

pub fn internal_complete_command_with_tokens(
    tokens: &[String],
) -> FrontendResult<FrontendCommandResult> {
    let matches = internal_complete_matches(tokens);
    Ok(FrontendCommandResult::new("_complete", matches.join("\n")))
}

pub fn internal_complete_matches(tokens: &[String]) -> Vec<String> {
    let (path, prefix) = match tokens.split_last() {
        Some((last, rest)) => (rest, last.as_str()),
        None => (&[][..], ""),
    };
    let command = FrontendCli::command();
    let mut matches = Vec::new();
    collect_matches_for_command(&command, path, prefix, &mut matches);

    matches.sort();
    matches.dedup();
    matches
}

fn collect_matches_for_command(
    command: &clap::Command,
    path: &[String],
    prefix: &str,
    matches: &mut Vec<String>,
) {
    if let Some((head, tail)) = path.split_first() {
        for subcommand in command.get_subcommands() {
            if subcommand.is_hide_set() {
                continue;
            }
            let name_match = subcommand.get_name() == head;
            let alias_match = subcommand
                .get_visible_aliases()
                .any(|alias| alias == head.as_str());
            if name_match || alias_match {
                collect_matches_for_command(subcommand, tail, prefix, matches);
                return;
            }
        }
        return;
    }

    for subcommand in command.get_subcommands() {
        if subcommand.is_hide_set() {
            continue;
        }
        let name = subcommand.get_name().to_string();
        if name.starts_with(prefix) {
            matches.push(name);
        }
        for alias in subcommand.get_visible_aliases() {
            if alias.starts_with(prefix) {
                matches.push(alias.to_string());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        completion_command, generate_bash_completion_script, generate_fish_completion_script,
        generate_zsh_completion_script, internal_complete_command_with_tokens,
        internal_complete_matches, CompletionShell,
    };

    #[test]
    fn completion_command_shell_reports_requested_shell() {
        let result = completion_command(CompletionShell::Bash).unwrap();

        assert_eq!(result.command, "completion");
        assert!(result.summary.contains("bash"));
    }

    #[test]
    fn internal_complete_command_has_a_stable_placeholder_surface() {
        let result = internal_complete_command_with_tokens(&["co".to_string()]).unwrap();

        assert_eq!(result.command, "_complete");
        assert!(result.summary.contains("code"));
    }

    #[test]
    fn bash_completion_script_contains_bash_completion_shape() {
        let script = generate_bash_completion_script().unwrap();

        assert!(script.contains("_fol()"));
        assert!(script.contains("complete -F"));
    }

    #[test]
    fn zsh_completion_script_contains_zsh_completion_shape() {
        let script = generate_zsh_completion_script().unwrap();

        assert!(script.contains("#compdef fol"));
        assert!(script.contains("_arguments"));
    }

    #[test]
    fn fish_completion_script_contains_fish_completion_shape() {
        let script = generate_fish_completion_script().unwrap();

        assert!(script.contains("complete -c fol"));
        assert!(script.contains("__fish_use_subcommand"));
    }

    #[test]
    fn internal_complete_matches_filter_visible_commands_and_aliases() {
        let matches = internal_complete_matches(&["c".to_string()]);

        assert!(matches.contains(&"code".to_string()));
        assert!(matches.contains(&"c".to_string()));
    }

    #[test]
    fn internal_complete_matches_follow_subcommand_context() {
        let code_emit =
            internal_complete_matches(&["code".to_string(), "emit".to_string(), "r".to_string()]);
        let work = internal_complete_matches(&["work".to_string(), "i".to_string()]);

        assert!(code_emit.contains(&"rust".to_string()));
        assert!(work.contains(&"info".to_string()));
    }
}
