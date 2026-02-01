use crate::types::SshKey;
use base64::{Engine as _, engine::general_purpose};

/// Extract text enclosed in `<` and `>` from the given string.
///
/// Returns `Some(&str)` if both chevrons are found and contain text,
/// otherwise returns `None`.
fn extract_chevron_text(s: &str) -> Option<&str> {
    let start = s.find('<')?;
    let end = s.find('>')?;
    if end <= start + 1 {
        return None; // empty inner part
    }
    Some(&s[start + 1..end])
}

/// Parse an SSH identity comment of the form `user@host`.
///
/// Returns `Some((user, host))` if the comment contains exactly one `@`,
/// contains no whitespace, and both `user` and `host` are non-empty.
/// Otherwise returns `None`.
fn process_ssh_comment(comment: &str) -> Option<(String, String, Option<String>)> {
    let mut user_host = comment.trim();
    let mut name: Option<String> = None;

    if comment.contains('<') || comment.contains('>') {
        // Must have exactly one '<' and one '>' and the '>' must come after '<'.
        if comment.matches('<').count() != 1 || comment.matches('>').count() != 1 {
            return None;
        }

        user_host = extract_chevron_text(&comment)?; // Return None as extraction of <> failed.
        name = comment.find('<').map(|idx| {
            comment[..idx].trim().replace( " ", "_")
        });
    }

    // exactly one @
    if user_host.matches('@').count() != 1 {
        return None;
    }
    // no whitespace anywhere
    if user_host.chars().any(|c| c.is_whitespace()) {
        return None;
    }
    // split and ensure non-empty parts
    let (user, host) = user_host.split_once('@')?;
    if user.is_empty() || host.is_empty() {
        return None;
    }
    Some((user.to_string(), host.to_string(), name))
}

/// Extract the SSH key type from an SSH public key blob.
///
/// The blob is expected to start with a big-endian `u32` length followed by
/// a UTF-8 string containing the key type (e.g. `"ssh-rsa"`). Returns
/// `Some(type)` on success, otherwise `None`.
fn get_ssh_key_type(blob: &[u8]) -> Option<String> {
    if blob.len() < 4 {
        return None;
    }
    let len = u32::from_be_bytes(blob[0..4].try_into().ok()?) as usize;
    if blob.len() < 4 + len {
        return None;
    }
    std::str::from_utf8(&blob[4..4 + len])
        .ok()
        .map(|s| s.to_string())
}

/// Query the local SSH agent and return identities as a vector of `SshKey`.
///
/// Connects to the local SSH agent, lists identities and converts them into
/// `SshKey` values. Identities whose comments can't be parsed or whose blob
/// doesn't contain a valid key type are skipped. Returns an `anyhow::Error` on failure.
///
/// # Examples
///
/// ```no_run
/// use ssh_agent_sync::agent::get_ssh_keys;
/// let _keys = get_ssh_keys().unwrap();
/// ```
pub fn get_ssh_keys() -> anyhow::Result<Vec<SshKey>> {
    let sess = ssh2::Session::new()?;
    let mut agent = sess.agent()?;
    agent.connect()?;
    agent.list_identities()?;
    let ids = agent.identities()?;
    let mut keys = Vec::new();
    for id in ids {
        let blob = id.blob();
        let comment = id.comment();

        let (user, host, name) = match process_ssh_comment(comment) {
            Some(t) => t,
            None => continue,
        };

        let key_type = match get_ssh_key_type(blob) {
            Some(k) => k,
            None => continue,
        };

        let b64 = general_purpose::STANDARD.encode(blob);

        keys.push(SshKey {
            name,
            user,
            host,
            key_type: key_type.clone(),
            key: b64,
            file_name: String::new(),
        });
    }
    Ok(keys)
}

/// Print SSH keys to stdout in the format: "<type> <base64> <user>@<host>".
///
/// Borrows the provided slice of `SshKey` and prints each key on its own line.
pub fn print_ssh_keys(keys: &[SshKey]) {
    println!("ssh agent keys:");
    for key in keys {
        println!("{} {} {}@{}", key.key_type, key.key, key.user, key.host);
    }
    println!("total keys: {}", keys.len());
}
