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

pub fn internal_complete_command() -> FrontendResult<FrontendCommandResult> {
    Ok(FrontendCommandResult::new(
        "_complete",
        "frontend internal completion hook is available",
    ))
}

#[cfg(test)]
mod tests {
    use super::{
        completion_command, generate_bash_completion_script, internal_complete_command,
        CompletionShell,
    };

    #[test]
    fn completion_command_shell_reports_requested_shell() {
        let result = completion_command(CompletionShell::Bash).unwrap();

        assert_eq!(result.command, "completion");
        assert!(result.summary.contains("bash"));
    }

    #[test]
    fn internal_complete_command_has_a_stable_placeholder_surface() {
        let result = internal_complete_command().unwrap();

        assert_eq!(result.command, "_complete");
    }

    #[test]
    fn bash_completion_script_contains_bash_completion_shape() {
        let script = generate_bash_completion_script().unwrap();

        assert!(script.contains("_fol()"));
        assert!(script.contains("complete -F"));
    }
}
