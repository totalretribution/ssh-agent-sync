use crate::types::SshKey;
use base64::engine::Engine;
use sha2::{Digest, Sha256};
extern crate sanitize_filename;

fn get_current_ssh_keys_crc() -> Option<String> {
    use std::fs;

    let config_path = crate::constants::ssh_config_file_path()?;
    let content = fs::read_to_string(&config_path).ok()?;

    for line in content.lines() {
        if line.starts_with(crate::constants::SSH_AGENT_SYNC_CRC_PREFIX) {
            let crc = line
                .trim_start_matches(crate::constants::SSH_AGENT_SYNC_CRC_PREFIX)
                .trim();
            return Some(crc.to_string());
        }
    }
    None
}

fn generate_ssh_keys_crc(keys: &Vec<SshKey>) -> String {
    let mut hasher = Sha256::new();
    for key in keys {
        hasher.update(key.user.as_bytes());
        hasher.update(key.host.as_bytes());
        hasher.update(key.key.as_bytes());
    }
    let result = hasher.finalize();
    base64::engine::general_purpose::STANDARD.encode(result)
}

/// Create a file for the given SSH key in the specified path.
///
/// The file name is derived from the key's name or host, sanitized for filesystem use.
///
/// The file will contain the SSH public key in the format: "<type> <base64> <user>@<host>".
///
/// Returns `Ok(())` on success, or an error message as `Err(String)` on failure.
fn create_key_file(key: &mut SshKey, path: &std::path::Path) -> Result<(), String> {
    key.file_name = match &key.name {
        Some(name) => name.clone(),
        None => key.host.clone(),
    };
    key.file_name = sanitize_filename::sanitize(&key.file_name.replace(".", "_").replace(" ", "_"));
    key.file_name.push_str(".pub");
    let file_path: std::path::PathBuf = path.join(&key.file_name);

    use std::fs::OpenOptions;
    use std::io::Write;

    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options
        .open(&file_path)
        .map_err(|e| format!("Failed to open key file {}: {}", file_path.display(), e))?;

    let key_file_content = format!("{} {} {}@{}", key.key_type, key.key, key.user, key.host);

    if cfg!(debug_assertions) {
        println!(
            "Creating key file at {} with content: {}",
            file_path.display(),
            key_file_content
        );
    }

    file.write_all(key_file_content.as_bytes())
        .map_err(|e| format!("Failed to write to key file {}: {}", file_path.display(), e))?;

    Ok(())
}

fn create_config_entry(key: &SshKey, key_folder: &std::path::Path) -> String {
    let key_path = key_folder.join(&key.file_name);

    let mut config = String::new();

    if let Some(ref name) = key.name {
        config.push_str(&format!("Host {}\n", name));
        config.push_str(&format!("    HostName {}\n", key.host));
        config.push_str(&format!("    User {}\n", key.user));
        config.push_str(&format!("    IdentityFile {}\n", key_path.display()));
        config.push_str("    IdentitiesOnly yes\n\n");
    }

    config.push_str(&format!("Host {}\n", key.host));
    config.push_str(&format!("    User {}\n", key.user));
    config.push_str(&format!("    IdentityFile {}\n", key_path.display()));
    config.push_str("    IdentitiesOnly yes\n\n");

    config
}

fn write_config_file(config: &str, config_file: &std::path::Path) -> Result<(), String> {
    use std::fs::OpenOptions;
    use std::io::Write;

    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);

    #[cfg(unix)]
    options.mode(0o600);

    let mut file = options.open(&config_file).map_err(|e| {
        format!(
            "Failed to open config file {}: {}",
            config_file.display(),
            e
        )
    })?;

    file.write_all(config.as_bytes()).map_err(|e| {
        format!(
            "Failed to write to config file {}: {}",
            config_file.display(),
            e
        )
    })?;
    Ok(())
}

fn check_base_config_needs_editing() -> bool {
    use std::io::{BufRead, BufReader};

    let Some(base_config_path) = crate::constants::ssh_base_config_file_path() else {
        return true;
    };

    let Some(include_line) = crate::constants::ssh_base_include_line() else {
        return true;
    };

    let Ok(file) = std::fs::File::open(&base_config_path) else {
        return true; // File doesn't exist, needs editing
    };

    let reader = BufReader::new(file);
    let include_trimmed = include_line.trim();

    for line in reader.lines().take(20) {
        // Only check first 20 lines
        if let Ok(line) = line {
            if line.contains(include_trimmed) {
                return false; // Include line found, no editing needed
            }
        }
    }

    true // Include line not found in first 20 lines
}

fn edit_base_config() -> Result<(), String> {
    let base_config_path = crate::constants::ssh_base_config_file_path()
        .ok_or_else(|| "Failed to determine SSH base config file path".to_string())?;

    let include_line = crate::constants::ssh_base_include_line()
        .ok_or_else(|| "Failed to determine SSH base config include line".to_string())?;

    let mut content = include_line.to_string();
    content.push_str("\n\n");

    // Append existing content if file exists
    if base_config_path.exists() {
        let existing_content = std::fs::read_to_string(&base_config_path).map_err(|e| {
            format!(
                "Failed to read SSH base config file {}: {}",
                base_config_path.display(),
                e
            )
        })?;
        content.push_str(&existing_content);
    }

    write_config_file(&content, &base_config_path)
}

pub fn add_keys_to_config(keys: &mut Vec<SshKey>, force: bool) -> Result<(), String> {
    println!("Getting stored ssh keys CRC");
    let new_crc = generate_ssh_keys_crc(keys);
    let current_crc = get_current_ssh_keys_crc();

    if !force && current_crc.is_some() {
        let crc = current_crc.unwrap();
        if crc == new_crc {
            println!("Skipping: CRCs match");
            return Ok(());
        }
    }

    // Resolve the SSH config key folder path or return an error if it cannot be determined.
    let key_folder = crate::constants::ssh_config_key_folder_path()
        .ok_or_else(|| "Failed to determine SSH config key folder path".to_string())?;

    // If the path exists but is not a directory, return an error.
    if key_folder.exists() && !key_folder.is_dir() {
        return Err(format!(
            "SSH config key folder exists but is not a directory: {}",
            key_folder.display()
        ));
    }

    // Create the folder if it does not exist.
    if !key_folder.exists() {
        std::fs::create_dir_all(&key_folder).map_err(|e| {
            format!(
                "Failed to create key folder {}: {}",
                key_folder.display(),
                e
            )
        })?;
    }

    let mut ssh_config = format!(
        "{}{}\n\n",
        crate::constants::SSH_AGENT_SYNC_CRC_PREFIX,
        new_crc
    );
    ssh_config.push_str("Host *\n    IdentitiesOnly yes\n\n");
    println!("Creating {} key files in ssh config key folder", keys.len());
    for key in keys.iter_mut() {
        // Fail fast if we cannot create a key file.
        if let Err(e) = create_key_file(key, &key_folder) {
            return Err(e);
        }
        let config_entry = create_config_entry(key, &key_folder);
        println!("Config entry for host {}:\n{}", key.host, config_entry);
        ssh_config.push_str(&config_entry);
    }

    let config_file = crate::constants::ssh_config_file_path()
        .ok_or_else(|| "Failed to determine SSH config file path".to_string())?;
    write_config_file(&ssh_config, &config_file)?;

    let edit_base = check_base_config_needs_editing();

    if edit_base {
        edit_base_config()?;
    }

    Ok(())
}
