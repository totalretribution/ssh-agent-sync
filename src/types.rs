#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshKey {
    pub name: Option<String>,
    pub user: String,
    pub host: String,
    pub key_type: String,
    pub key: String,
    pub file_name: String,
}