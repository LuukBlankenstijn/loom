use chrono::TimeZone;
use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, Offset};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, Default)]
pub struct UiConfig {
    #[serde(default = "default_chain")]
    pub(crate) chain: String,

    pub(crate) background_source: Option<String>,

    #[serde(default, deserialize_with = "deserialize_end_time")]
    pub(crate) countdown_end_time: Option<DateTime<FixedOffset>>,

    #[serde(default = "default_count_from")]
    pub(crate) countdown_from: Option<u64>,

    #[serde(default = "default_count_end_login")]
    pub(crate) countdown_end_login: bool,

    #[serde(default = "default_countdown_label_color")]
    pub(crate) countdown_label_color: String,
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
