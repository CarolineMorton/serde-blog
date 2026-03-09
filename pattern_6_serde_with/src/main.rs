use csv::ReaderBuilder;
use serde::Deserialize;
use serde_with::formats::SemicolonSeparator;
use serde_with::{serde_as, BoolFromInt, DisplayFromStr, NoneAsEmptyString, StringWithSeparator};
use std::fmt;
use std::str::FromStr;

/// ICD code newtype with FromStr validation and Display implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct IcdCode(String);

impl FromStr for IcdCode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_uppercase().replace('.', "");

        if trimmed.len() >= 3
            && trimmed.as_bytes()[0].is_ascii_alphabetic()
            && trimmed[1..].chars().all(|c| c.is_ascii_digit())
        {
            Ok(IcdCode(trimmed))
        } else {
            Err(format!("invalid ICD-10 format: '{s}'"))
        }
    }
}

impl fmt::Display for IcdCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// ReadingId newtype with FromStr validation and Display implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ReadingId(String);

impl FromStr for ReadingId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.starts_with("RD-") && trimmed.len() > 3 {
            Ok(ReadingId(trimmed.to_string()))
        } else {
            Err(format!("invalid reading ID: '{s}', expected format RD-xxx"))
        }
    }
}

impl fmt::Display for ReadingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Parses a diagnosis code list from a semicolon-separated string 
/// into separate strings. We use this to show how to parse the raw strings,
#[serde_as]
#[derive(Debug, Deserialize)]
struct PatientDiagnosesRaw {
    patient_id: String,

    #[serde_as(as = "StringWithSeparator::<SemicolonSeparator, String>")]
    diagnosis_codes: Vec<String>,
}

/// Parses a diagnosis code list from a semicolon-separated string
/// into a Vec<IcdCode>, running each code through IcdCode's FromStr validation.
/// We use this to show how to parse into a more strongly-typed struct with validation.
#[serde_as]
#[derive(Debug, Deserialize)]
struct PatientDiagnosesTyped {
    patient_id: String,

    /// Deserializes "E119;I10;J45" directly into a Vec<IcdCode>,
    /// running each code through IcdCode's FromStr validation.
    #[serde_as(as = "StringWithSeparator::<SemicolonSeparator, IcdCode>")]
    diagnosis_codes: Vec<IcdCode>,
}

/// Sensor reading struct to demonstrate NoneAsEmptyString, BoolFromInt, and DisplayFromStr.
#[serde_as]
#[derive(Debug, Deserialize, PartialEq)]
struct SensorReading {
    device_id: String,

    /// Source sends empty string instead of null for missing locations.
    #[serde_as(as = "NoneAsEmptyString")]
    location: Option<String>,

    /// Source encodes booleans as 0/1 integers.
    #[serde_as(as = "BoolFromInt")]
    is_calibrated: bool,

    /// Any type implementing FromStr can be deserialized from a string.
    #[serde_as(as = "DisplayFromStr")]
    reading_id: ReadingId,
}

// ============================================================
// Main
// ============================================================

fn main() {
    println!("=== Raw string parsing ===\n");

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/diagnoses.csv")
        .expect("could not open CSV file");

    for (i, result) in reader.deserialize::<PatientDiagnosesRaw>().enumerate() {
        let line = i + 2;
        match result {
            Ok(record) => {
                println!(
                    "  line {line}: {} -> {} codes: {:?}",
                    record.patient_id,
                    record.diagnosis_codes.len(),
                    record.diagnosis_codes,
                );
            }
            Err(e) => println!("  line {line}: [ERROR] {e}"),
        }
    }

    println!("\n=== Typed IcdCode parsing ===\n");

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/diagnoses.csv")
        .expect("could not open CSV file");

    for (i, result) in reader.deserialize::<PatientDiagnosesTyped>().enumerate() {
        let line = i + 2;
        match result {
            Ok(record) => {
                println!(
                    "  line {line}: {} -> {} codes: {:?}",
                    record.patient_id,
                    record.diagnosis_codes.len(),
                    record.diagnosis_codes,
                );
            }
            Err(e) => println!("  line {line}: [ERROR] {e}"),
        }
    }

    println!("\n=== SensorReading (NoneAsEmptyString, BoolFromInt, DisplayFromStr) ===\n");

    let csv_data = "device_id,location,is_calibrated,reading_id\n\
                    SENS-001,London,1,RD-001\n\
                    SENS-002,,0,RD-002\n\
                    SENS-003,Manchester,1,RD-003\n";

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_data.as_bytes());

    for (i, result) in reader.deserialize::<SensorReading>().enumerate() {
        let line = i + 2;
        match result {
            Ok(reading) => {
                let loc = reading
                    .location
                    .as_deref()
                    .unwrap_or("(none)");
                println!(
                    "  line {line}: {} location={} calibrated={} id={}",
                    reading.device_id, loc, reading.is_calibrated, reading.reading_id.0
                );
            }
            Err(e) => println!("  line {line}: [ERROR] {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- IcdCode unit tests --

    #[test]
    fn icd_code_parses_valid() {
        let code: IcdCode = "E119".parse().unwrap();
        assert_eq!(code.0, "E119");
    }

    #[test]
    fn icd_code_normalises_dots() {
        let code: IcdCode = "E11.9".parse().unwrap();
        assert_eq!(code.0, "E119");
    }

    #[test]
    fn icd_code_normalises_case() {
        let code: IcdCode = "e119".parse().unwrap();
        assert_eq!(code.0, "E119");
    }

    #[test]
    fn icd_code_rejects_numeric_only() {
        assert!("119".parse::<IcdCode>().is_err());
    }

    #[test]
    fn icd_code_rejects_too_short() {
        assert!("E1".parse::<IcdCode>().is_err());
    }

    // -- ReadingId unit tests --

    #[test]
    fn reading_id_parses_valid() {
        let id: ReadingId = "RD-001".parse().unwrap();
        assert_eq!(id.0, "RD-001");
    }

    #[test]
    fn reading_id_rejects_wrong_prefix() {
        assert!("XX-001".parse::<ReadingId>().is_err());
    }

    #[test]
    fn reading_id_rejects_bare_prefix() {
        assert!("RD-".parse::<ReadingId>().is_err());
    }

    // -- CSV integration tests: raw strings --

    fn parse_raw(csv_data: &str) -> Result<PatientDiagnosesRaw, csv::Error> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        reader.deserialize().next().unwrap()
    }

    #[test]
    fn raw_parses_multiple_codes() {
        let csv = "patient_id,diagnosis_codes\nP001,E119;I10;J45";
        let record = parse_raw(csv).unwrap();
        assert_eq!(record.diagnosis_codes, vec!["E119", "I10", "J45"]);
    }

    #[test]
    fn raw_parses_single_code() {
        let csv = "patient_id,diagnosis_codes\nP002,I10";
        let record = parse_raw(csv).unwrap();
        assert_eq!(record.diagnosis_codes, vec!["I10"]);
    }

    #[test]
    fn raw_parses_many_codes() {
        let csv = "patient_id,diagnosis_codes\nP003,E119;E149;I10;J45;M545";
        let record = parse_raw(csv).unwrap();
        assert_eq!(record.diagnosis_codes.len(), 5);
    }

    // -- CSV integration tests: typed IcdCode --

    fn parse_typed(csv_data: &str) -> Result<PatientDiagnosesTyped, csv::Error> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        reader.deserialize().next().unwrap()
    }

    #[test]
    fn typed_parses_valid_codes() {
        let csv = "patient_id,diagnosis_codes\nP001,E119;I10;J45";
        let record = parse_typed(csv).unwrap();
        assert_eq!(record.diagnosis_codes.len(), 3);
        assert_eq!(record.diagnosis_codes[0].0, "E119");
        assert_eq!(record.diagnosis_codes[1].0, "I10");
        assert_eq!(record.diagnosis_codes[2].0, "J45");
    }

    #[test]
    fn typed_rejects_invalid_code_in_list() {
        let csv = "patient_id,diagnosis_codes\nP001,E119;INVALID;J45";
        assert!(parse_typed(csv).is_err());
    }

    // -- CSV integration tests: SensorReading (DisplayFromStr, NoneAsEmptyString, BoolFromInt) --

    fn parse_sensor(csv_data: &str) -> Result<SensorReading, csv::Error> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        reader.deserialize().next().unwrap()
    }

    #[test]
    fn sensor_display_from_str_parses_reading_id() {
        let csv = "device_id,location,is_calibrated,reading_id\nS1,London,1,RD-001";
        let reading = parse_sensor(csv).unwrap();
        assert_eq!(reading.reading_id, ReadingId("RD-001".to_string()));
    }

    #[test]
    fn sensor_display_from_str_rejects_invalid_reading_id() {
        let csv = "device_id,location,is_calibrated,reading_id\nS1,London,1,INVALID";
        assert!(parse_sensor(csv).is_err());
    }

    #[test]
    fn sensor_none_as_empty_string_returns_none() {
        let csv = "device_id,location,is_calibrated,reading_id\nS1,,1,RD-001";
        let reading = parse_sensor(csv).unwrap();
        assert_eq!(reading.location, None);
    }

    #[test]
    fn sensor_none_as_empty_string_returns_some() {
        let csv = "device_id,location,is_calibrated,reading_id\nS1,London,1,RD-001";
        let reading = parse_sensor(csv).unwrap();
        assert_eq!(reading.location, Some("London".to_string()));
    }

    #[test]
    fn sensor_bool_from_int_parses_1_as_true() {
        let csv = "device_id,location,is_calibrated,reading_id\nS1,London,1,RD-001";
        let reading = parse_sensor(csv).unwrap();
        assert!(reading.is_calibrated);
    }

    #[test]
    fn sensor_bool_from_int_parses_0_as_false() {
        let csv = "device_id,location,is_calibrated,reading_id\nS1,London,0,RD-001";
        let reading = parse_sensor(csv).unwrap();
        assert!(!reading.is_calibrated);
    }
}