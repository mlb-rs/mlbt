use chrono::NaiveDate;
use serde::{Deserialize, Deserializer, Serializer};

/// Tolerant serde adapter for `Option<NaiveDate>` from API `YYYY-MM-DD` strings. Deserialization
/// returns `None` for null, missing, or unparseable values so a single bad date doesn't fail the
/// whole response.
pub mod optional_date {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = Option::<String>::deserialize(deserializer)?;
        Ok(s.as_deref()
            .and_then(|s| NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()))
    }

    pub fn serialize<S>(date: &Option<NaiveDate>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => serializer.serialize_str(&d.format("%Y-%m-%d").to_string()),
            None => serializer.serialize_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Holder {
        #[serde(default, with = "super::optional_date")]
        date: Option<NaiveDate>,
    }

    #[test]
    fn optional_date_handles_all_cases() {
        let valid = NaiveDate::from_ymd_opt(2026, 4, 13).unwrap();

        // Valid string
        let h: Holder = serde_json::from_str(r#"{"date":"2026-04-13"}"#).unwrap();
        assert_eq!(h.date, Some(valid));

        // Null, missing, and unparseable all yield None instead of failing
        for input in [
            r#"{"date":null}"#,
            r#"{}"#,
            r#"{"date":"not-a-date"}"#,
            r#"{"date":""}"#,
        ] {
            let h: Holder = serde_json::from_str(input).unwrap();
            assert_eq!(h.date, None, "input: {input}");
        }

        // Round-trip serializes back to the same string
        let h = Holder { date: Some(valid) };
        let json = serde_json::to_string(&h).unwrap();
        assert_eq!(json, r#"{"date":"2026-04-13"}"#);
    }
}
