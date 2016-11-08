#[derive(Default, Deserialize, Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct Token {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub expires_in: u32,
    #[serde(default)]
    pub id_token: String,

    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub error_desc: String,
    #[serde(default)]
    pub error_uri: String,
}
