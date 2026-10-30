#[derive(Debug, Clone)]
pub struct Station {
    pub ip: String,
    pub reported_label: Option<String>,
    pub label_override: Option<String>,
}

impl Station {
    pub fn label(&self) -> Option<&str> {
        self.label_override
            .as_deref()
            .or(self.reported_label.as_deref())
    }
}

pub fn normalize_label(label: &str) -> Option<String> {
    let label = label.trim();
    (!label.is_empty()).then(|| label.to_lowercase())
}
