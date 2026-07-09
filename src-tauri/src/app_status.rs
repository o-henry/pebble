use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub phase: &'static str,
    pub scaffold_ready: bool,
    pub capture_enabled: bool,
    pub ai_enabled: bool,
}

impl AppStatus {
    pub fn pre_alpha() -> Self {
        Self {
            phase: "pre-alpha",
            scaffold_ready: true,
            capture_enabled: false,
            ai_enabled: false,
        }
    }
}
