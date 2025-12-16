use chrono::TimeZone;
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Offset};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct UiConfig {
    /// Key sequence to toggle the login UI.
    #[serde(default = "default_chain")]
    pub(crate) chain: String,

    /// File path or URL for the background image.
    pub(crate) background_source: Option<String>,

    /// Contest start time. Accepts RFC3339 or `YYYY-MM-DD hh:mm:ss` (interpreted as local time).
    #[serde(default, deserialize_with = "deserialize_end_time")]
    pub(crate) countdown_end_time: Option<DateTime<FixedOffset>>,

    /// Start showing the countdown when this many seconds remain.
    #[serde(default = "default_count_from")]
    pub(crate) countdown_from: Option<u64>,

    /// Trigger login automatically when the countdown reaches zero.
    #[serde(default = "default_count_end_login")]
    pub(crate) countdown_end_login: bool,

    /// Color for the countdown label (CSS color value).
    #[serde(default = "default_countdown_label_color")]
    pub(crate) countdown_label_color: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            chain: default_chain(),
            background_source: None,
            countdown_end_time: None,
            countdown_from: default_count_from(),
            countdown_end_login: default_count_end_login(),
            countdown_label_color: default_countdown_label_color(),
        }
    }
}

fn default_chain() -> String {
    "chain".into()
}

fn default_count_end_login() -> bool {
    true
}

fn default_count_from() -> Option<u64> {
    Some(10)
}

fn default_countdown_label_color() -> String {
    "white".into()
}

fn deserialize_end_time<'de, D>(deserializer: D) -> Result<Option<DateTime<FixedOffset>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<toml::value::Datetime>::deserialize(deserializer)?;

    let Some(dt) = opt else {
        return Ok(None);
    };

    let repr = dt.to_string();

    if let Ok(with_offset) = DateTime::parse_from_rfc3339(&repr) {
        return Ok(Some(with_offset));
    }

    let naive = NaiveDateTime::parse_from_str(&repr, "%Y-%m-%d %H:%M:%S")
        .map_err(serde::de::Error::custom)?;

    match Local.from_local_datetime(&naive) {
        chrono::LocalResult::Single(local_dt) => {
            Ok(Some(local_dt.with_timezone(&local_dt.offset().fix())))
        }
        chrono::LocalResult::Ambiguous(_, _) => {
            Err(serde::de::Error::custom("ambiguous local time"))
        }
        chrono::LocalResult::None => Err(serde::de::Error::custom("invalid local time")),
    }
}
