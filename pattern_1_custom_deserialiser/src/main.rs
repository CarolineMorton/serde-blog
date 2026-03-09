use csv::ReaderBuilder;
use serde::{Deserialize, Deserializer};

/// Custom deserialiser to handle flexible boolean representations in the CSV data.
fn flexible_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?.trim().to_lowercase();

    match value.as_str() {
        "y" | "yes" | "true" | "1" | "t" => Ok(true),
        "n" | "no" | "false" | "0" | "f" => Ok(false),
        other => Err(serde::de::Error::custom(format!(
            "unrecognised boolean value: '{other}'"
        ))),
    }
}

/// Represents a patient record with a flexible boolean field for smoking status.
#[derive(Debug, Deserialize)]
struct PatientRecord {
    patient_id: String,

    #[serde(deserialize_with = "flexible_bool")]
    is_smoker: bool,
}

fn main() {
    // Read data in 
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/patients.csv")
        .expect("could not open CSV file");

    // Deserialise each record and print the results, including any errors encountered.
    for (i, result) in reader.deserialize::<PatientRecord>().enumerate() {
        let line = i + 2;
        match result {
            Ok(record) => {
                println!("line {line}: {} -> is_smoker={}", record.patient_id, record.is_smoker);
            }
            Err(e) => {
                println!("line {line}: [ERROR] {e}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::flexible_bool;
    use serde::de::IntoDeserializer;

    fn parse(input: &str) -> Result<bool, serde::de::value::Error> {
        let deserializer: serde::de::value::StrDeserializer<serde::de::value::Error> =
            input.into_deserializer();
        flexible_bool(deserializer)
    }

    #[test]
    fn parses_true_variants() {
        for input in ["yes", "Yes", "YES", "y", "Y", "true", "True", "TRUE", "1", "t", "T"] {
            assert_eq!(parse(input).unwrap(), true, "expected true for '{input}'");
        }
    }

    #[test]
    fn parses_false_variants() {
        for input in ["no", "No", "NO", "n", "N", "false", "False", "FALSE", "0", "f", "F"] {
            assert_eq!(parse(input).unwrap(), false, "expected false for '{input}'");
        }
    }

    #[test]
    fn rejects_unknown_values() {
        for input in ["maybe", "unknown", "2", "yep", "nah", ""] {
            assert!(parse(input).is_err(), "expected error for '{input}'");
        }
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(parse(" yes ").unwrap(), true);
        assert_eq!(parse(" no ").unwrap(), false);
    }
}