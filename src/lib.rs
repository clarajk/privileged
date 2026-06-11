use nix::unistd::Uid;
use thiserror::Error;

#[cfg(not(target_os = "linux"))]
compile_error!(
    "This crate only supports Linux. For cross-platform privilege elevation, consider the runas crate."
);

#[derive(Debug, Error)]
pub enum PrivilegeError {
    #[error("Privilege elevation command `{0}` is not available.")]
    NotAvailable(&'static str),

    #[error("Privilege elevation command `{0}` is not supported.")]
    NotSupported(String),

    #[error("Unable to detect a privilege elevation command (tried: run0, sudo, doas).")]
    NotDetected,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrivilegeMethod {
    Auto,
    Run0,
    Sudo,
    Doas,
    #[cfg(feature = "pkexec")]
    Pkexec,
}

impl PrivilegeMethod {
    pub fn from_env() -> Result<Self, PrivilegeError> {
        Self::from_env_var("PRIVILEGED_METHOD")
    }

    pub fn from_env_var(var_name: impl AsRef<str>) -> Result<Self, PrivilegeError> {
        match std::env::var(var_name.as_ref()).as_deref() {
            Ok("auto") => Ok(Self::Auto),
            Ok("run0") => Ok(Self::Run0),
            Ok("sudo") => Ok(Self::Sudo),
            Ok("doas") => Ok(Self::Doas),
            #[cfg(feature = "pkexec")]
            Ok("pkexec") => Ok(Self::Pkexec),
            Ok(x) => Err(PrivilegeError::NotSupported(x.to_string())),
            _ => Ok(Self::Auto),
        }
    }

    pub fn is_available(&self) -> bool {
        let Ok(detected) = detect_privilege_method(*self) else {
            return false;
        };

        which::which(detected).is_ok()
    }
}

pub fn command(program: impl AsRef<str>) -> Result<std::process::Command, PrivilegeError> {
    command_with(program, PrivilegeMethod::from_env()?)
}

pub fn command_with(
    program: impl AsRef<str>,
    method: PrivilegeMethod,
) -> Result<std::process::Command, PrivilegeError> {
    let privilege_exe = detect_privilege_method(method)?;
    if which::which(privilege_exe).is_err() {
        return Err(PrivilegeError::NotAvailable(privilege_exe));
    }

    if is_privileged() {
        Ok(std::process::Command::new(program.as_ref()))
    } else {
        let mut cmd = std::process::Command::new(privilege_exe);
        cmd.arg(program.as_ref());
        Ok(cmd)
    }
}

pub fn is_privileged() -> bool {
    Uid::effective().is_root()
}

fn detect_privilege_method(method: PrivilegeMethod) -> Result<&'static str, PrivilegeError> {
    const PRIVILEGE_METHODS: &[&str] = &["run0", "sudo", "doas"];

    match method {
        PrivilegeMethod::Auto => {
            for method in PRIVILEGE_METHODS {
                if which::which(method).is_ok() {
                    return Ok(method);
                }
            }

            Err(PrivilegeError::NotDetected)
        }
        PrivilegeMethod::Run0 => Ok("run0"),
        PrivilegeMethod::Sudo => Ok("sudo"),
        PrivilegeMethod::Doas => Ok("doas"),
        #[cfg(feature = "pkexec")]
        PrivilegeMethod::Pkexec => Ok("pkexec"),
    }
}
