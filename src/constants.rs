use std::path::PathBuf;

pub const PROGRAM_NAME: &str = "ssh-agent-sync";

pub const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Name of the SSH directory in the user's home folder.
pub const SSH_DIR_NAME: &str = ".ssh";

pub const SSH_BASE_CONFIG_FILE_NAME: &str = "config";

/// Name of the folder used to store ssh-agent-sync keys.
pub const SSH_CONFIG_KEY_FOLDER: &str = "ssh_agent_sync";

/// File name of the ssh-agent-sync config file.
pub const SSH_CONFIG_FILE_NAME: &str = "config.ssh_agent_sync";

/// Prefix used in the SSH config file to store the CRC of synced keys.
pub const SSH_AGENT_SYNC_CRC_PREFIX: &str = "### SSH_AGENT_SYNC_CRC=";

/// Returns the user's SSH directory path (e.g. `$HOME/.ssh`).
/// Returns `None` if the home directory can't be determined.
pub fn ssh_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|d| d.join(SSH_DIR_NAME))
}

pub fn ssh_base_config_file_path() -> Option<PathBuf> {
    ssh_dir().map(|d| d.join(SSH_BASE_CONFIG_FILE_NAME))
}

/// Returns the path to the key folder (e.g. `$HOME/.ssh/ssh_agent_sync`).
pub fn ssh_config_key_folder_path() -> Option<PathBuf> {
    ssh_dir().map(|d| d.join(SSH_CONFIG_KEY_FOLDER))
}

/// Returns the path to the ssh-agent-sync config file (e.g. `$HOME/.ssh/config.ssh_agent_sync`).
pub fn ssh_config_file_path() -> Option<PathBuf> {
    ssh_dir().map(|d| d.join(SSH_CONFIG_FILE_NAME))
}

pub fn ssh_base_include_line() -> Option<String> {
    let file_path = ssh_config_file_path()?;
    Some(format!("Include {}", file_path.display()))
}
