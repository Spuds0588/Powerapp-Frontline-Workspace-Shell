use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub title: String,
    pub kiosk_mode: bool,
    pub fullscreen: bool,
    pub window_width: f64,
    pub window_height: f64,
    pub disable_context_menu: bool,
    pub allowed_domains: Vec<String>,
    pub layout: LayoutConfig,
    pub panes: PanesConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConfig {
    pub mode: String,
    pub sidebar_position: String,
    pub sidebar_width: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanesConfig {
    pub primary: PaneTarget,
    pub secondary: Option<PaneTarget>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PaneTarget {
    pub url: String,
}
