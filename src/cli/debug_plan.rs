use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct DebugPlan {
    pub surface: &'static str,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor: Option<&'static str>,
    pub legs: Vec<DebugPlanLeg>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct DebugPlanLeg {
    pub leg: String,
    pub entity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filters: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routing: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_sources: Vec<String>,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
