use fol_frontend::{ColorPolicy, FrontendCli, FrontendProfile, OutputMode, run_command_from_args_in_dir};

struct EnvGuard {
    key: &'static str,
    old: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let old = std::env::var_os(key);
        unsafe {
            std::env::set_var(key, value);
        }
        Self { key, old }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        match &self.old {
            Some(value) => unsafe {
                std::env::set_var(self.key, value);
            },
            None => unsafe {
                std::env::remove_var(self.key);
            },
        }
    }
}

#[test]
fn frontend_dispatch_uses_env_defaults_for_output_and_color() {
    let _output = EnvGuard::set("FOL_OUTPUT", "plain");
    let _color = EnvGuard::set("FOL_COLOR", "never");
    let _profile = EnvGuard::set("FOL_PROFILE", "release");

    let (output, _) =
        run_command_from_args_in_dir(["fol", "_complete"], std::env::temp_dir()).unwrap();

    assert_eq!(output.config().mode, OutputMode::Plain);
    assert_eq!(output.config().color, ColorPolicy::Never);

    let cli = FrontendCli::parse_from(["fol", "build"]);
    assert_eq!(cli.selected_profile(), FrontendProfile::Release);
}

#[test]
fn frontend_dispatch_flags_override_env_defaults_for_output_color_and_profile() {
    let _output = EnvGuard::set("FOL_OUTPUT", "plain");
    let _color = EnvGuard::set("FOL_COLOR", "never");
    let _profile = EnvGuard::set("FOL_PROFILE", "release");

    let (output, _) = run_command_from_args_in_dir(
        ["fol", "--output", "json", "--color", "always", "_complete"],
        std::env::temp_dir(),
    )
    .unwrap();

    assert_eq!(output.config().mode, OutputMode::Json);
    assert_eq!(output.config().color, ColorPolicy::Always);

    let cli = FrontendCli::parse_from(["fol", "--debug", "build"]);
    assert_eq!(cli.selected_profile(), FrontendProfile::Debug);
}
