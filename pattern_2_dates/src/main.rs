use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::{Deserialize, Deserializer};

/// Parses a clinical date from a string, trying multiple common formats.
fn parse_clinical_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    let s = raw.trim();

    let formats = [
        "%Y-%m-%d", // 2024-03-15
        "%Y%m%d",   // 20240315
        "%d-%b-%Y", // 15-Mar-2024
        "%d/%m/%Y", // 15/03/2024
    ];

    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
            return Ok(date);
        }
    }

    Err(serde::de::Error::custom(format!(
        "no known date format matched: '{s}'"
    )))
}

/// Parses an optional clinical date from a string, returning `None` for empty or special values.
fn parse_optional_clinical_date<'de, D>(
    deserializer: D,
) -> Result<Option<NaiveDate>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    let s = raw.trim();

    if s.is_empty() || s == "NULL" || s == "NA" || s == "." {
        return Ok(None);
    }

    let formats = ["%Y-%m-%d", "%Y%m%d", "%d-%b-%Y", "%d/%m/%Y"];

    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
            return Ok(Some(date));
        }
    }

    Err(serde::de::Error::custom(format!(
        "no known date format matched: '{s}'"
    )))
}

/// Represents a hospital episode with patient ID, admission date, and optional discharge date.
/// This is a row in the CSV file, with custom deserialization for the date fields.
#[derive(Debug, Deserialize)]
struct HospitalEpisode {
    patient_id: String,

    #[serde(deserialize_with = "parse_clinical_date")]
    admission_date: NaiveDate,

    #[serde(deserialize_with = "parse_optional_clinical_date")]
    discharge_date: Option<NaiveDate>,
}

fn main() {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/episodes.csv")
        .expect("could not open CSV file");

    for (i, result) in reader.deserialize::<HospitalEpisode>().enumerate() {
        let line = i + 2;
        match result {
            Ok(record) => {
                let discharge = match record.discharge_date {
                    Some(d) => d.to_string(),
                    None => "still admitted".to_string(),
                };
                println!(
                    "line {line}: {} admitted={} discharged={}",
                    record.patient_id, record.admission_date, discharge
                );
            }
            Err(e) => {
                println!("line {line}: [ERROR] {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::IntoDeserializer;

    fn parse_date(input: &str) -> Result<NaiveDate, serde::de::value::Error> {
        let deserializer: serde::de::value::StrDeserializer<serde::de::value::Error> =
            input.into_deserializer();
        parse_clinical_date(deserializer)
    }

    fn parse_opt_date(input: &str) -> Result<Option<NaiveDate>, serde::de::value::Error> {
        let deserializer: serde::de::value::StrDeserializer<serde::de::value::Error> =
            input.into_deserializer();
        parse_optional_clinical_date(deserializer)
    }

    #[test]
    fn parses_iso_format() {
        let date = parse_date("2024-03-15").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn parses_compact_format() {
        let date = parse_date("20240315").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn parses_clinical_format() {
        let date = parse_date("15-Mar-2024").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn parses_uk_format() {
        let date = parse_date("15/03/2024").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn rejects_invalid_date() {
        assert!(parse_date("not-a-date").is_err());
    }

    #[test]
    fn trims_whitespace() {
        let date = parse_date(" 2024-03-15 ").unwrap();
        assert_eq!(date, NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn optional_returns_none_for_na() {
        assert_eq!(parse_opt_date("NA").unwrap(), None);
    }

    #[test]
    fn optional_returns_none_for_null() {
        assert_eq!(parse_opt_date("NULL").unwrap(), None);
    }

    #[test]
    fn optional_returns_none_for_dot() {
        assert_eq!(parse_opt_date(".").unwrap(), None);
    }

    #[test]
    fn optional_returns_none_for_empty() {
        assert_eq!(parse_opt_date("").unwrap(), None);
    }

    #[test]
    fn optional_parses_valid_date() {
        let date = parse_opt_date("2024-03-15").unwrap();
        assert_eq!(date, Some(NaiveDate::from_ymd_opt(2024, 3, 15).unwrap()));
    }

    #[test]
    fn optional_rejects_invalid_non_empty_date() {
        assert!(parse_opt_date("not-a-date").is_err());
    }
}