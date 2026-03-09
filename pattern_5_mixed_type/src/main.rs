use chrono::NaiveDate;
use csv::ReaderBuilder;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::sync::LazyLock;

/// Parses a clinical date from a string, trying multiple common formats. 
/// We saw this in `pattern_2_dates`
fn parse_clinical_date<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    let s = raw.trim();

    let formats = ["%Y-%m-%d", "%Y%m%d", "%d-%b-%Y", "%d/%m/%Y"];

    for fmt in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(s, fmt) {
            return Ok(date);
        }
    }

    Err(serde::de::Error::custom(format!(
        "no known date format matched: '{s}'"
    )))
}

/// CRP enum to represent the mixed-type CRP values we see in the CSV.
#[derive(Debug, PartialEq)]
enum CrpValue {
    Numeric(f64),
    NonNumeric(String),
}

/// CSV sends all fields as strings, so we deserialise as a string
/// and try to parse it as a number. If it parses, we get Numeric.
/// If not, we keep the original string as NonNumeric.
impl<'de> Deserialize<'de> for CrpValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.trim().parse::<f64>() {
            Ok(n) => Ok(CrpValue::Numeric(n)),
            Err(_) => Ok(CrpValue::NonNumeric(s)),
        }
    }
}

/// Matches below-detection-limit values like "<1", "<0.5", "< 1"
static BELOW_LIMIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^<\s*\d+(\.\d+)?$").unwrap()
});

impl CrpValue {
    /// Converts a CRP value to a numeric result.
    ///
    /// - Numeric values pass through directly.
    /// - Below-detection-limit strings like "<1" are recoded to 0.
    /// - Anything else (e.g. "sample haemolysed") is treated as missing.
    fn to_numeric(&self) -> Option<f64> {
        match self {
            CrpValue::Numeric(n) => Some(*n),
            CrpValue::NonNumeric(s) if BELOW_LIMIT.is_match(s.trim()) => Some(0.0),
            CrpValue::NonNumeric(_) => None,
        }
    }
}

impl std::fmt::Display for CrpValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrpValue::Numeric(n) => write!(f, "{n}"),
            CrpValue::NonNumeric(s) => write!(f, "\"{s}\""),
        }
    }
}

/// LabResult struct representing a single row of the CSV,
//  with custom deserialization for the date and CRP value.
#[derive(Debug, Deserialize)]
struct LabResult {
    patient_id: String,

    #[serde(deserialize_with = "parse_clinical_date")]
    sample_date: NaiveDate,

    value: CrpValue,
}

fn main() {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/lab_results.csv")
        .expect("could not open CSV file");

    let mut numeric_count = 0u32;
    let mut below_limit_count = 0u32;
    let mut unusable_count = 0u32;

    for (i, result) in reader.deserialize::<LabResult>().enumerate() {
        let line = i + 2;
        match result {
            Ok(lab) => {
                let numeric = lab.value.to_numeric();
                let status = match (&lab.value, numeric) {
                    (CrpValue::Numeric(_), Some(n)) => {
                        numeric_count += 1;
                        format!("numeric -> {n}")
                    }
                    (CrpValue::NonNumeric(_), Some(n)) => {
                        below_limit_count += 1;
                        format!("below limit -> recoded to {n}")
                    }
                    (_, None) => {
                        unusable_count += 1;
                        "unusable -> excluded".to_string()
                    }
                };
                println!(
                    "line {line}: {} date={} raw={} {}",
                    lab.patient_id, lab.sample_date, lab.value, status
                );
            }
            Err(e) => {
                println!("line {line}: [ERROR] {e}");
            }
        }
    }

    println!(
        "\nSummary: {} numeric, {} below limit (recoded to 0), {} unusable (excluded)",
        numeric_count, below_limit_count, unusable_count
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_crp(input: &str) -> CrpValue {
        let csv_data = format!("value\n{input}");
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        reader.deserialize::<CrpValue>().next().unwrap().unwrap()
    }

    #[test]
    fn parses_integer_as_numeric() {
        let v = parse_crp("45");
        assert_eq!(v.to_numeric(), Some(45.0));
    }

    #[test]
    fn parses_zero_as_numeric() {
        let v = parse_crp("0");
        assert_eq!(v.to_numeric(), Some(0.0));
    }

    #[test]
    fn parses_float_as_numeric() {
        let v = parse_crp("12.5");
        assert_eq!(v.to_numeric(), Some(12.5));
    }

    #[test]
    fn below_limit_recodes_to_zero() {
        for input in ["<1", "<0.5", "< 1", "<100"] {
            let v = parse_crp(input);
            assert_eq!(
                v.to_numeric(),
                Some(0.0),
                "expected 0.0 for '{input}'"
            );
        }
    }

    #[test]
    fn unusable_strings_return_none() {
        for input in ["sample haemolysed", "insufficient sample", "equipment error"] {
            let v = parse_crp(input);
            assert_eq!(
                v.to_numeric(),
                None,
                "expected None for '{input}'"
            );
        }
    }

    #[test]
    fn preserves_original_string() {
        let v = parse_crp("sample haemolysed");
        match v {
            CrpValue::NonNumeric(s) => assert_eq!(s, "sample haemolysed"),
            _ => panic!("expected NonNumeric"),
        }
    }
}