use csv::ReaderBuilder;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum SmokingStatus {
    #[serde(alias = "non-smoker", alias = "nonsmoker", alias = "never smoked")]
    Never,

    #[serde(alias = "ex", alias = "ex-smoker", alias = "former smoker")]
    Former,

    #[serde(alias = "smoker", alias = "active", alias = "current smoker")]
    Current,

    #[serde(alias = "NA", alias = "", alias = "not recorded")]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ImdQuintile(u8);

impl<'de> Deserialize<'de> for ImdQuintile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = u8::deserialize(deserializer)?;
        if (1..=5).contains(&v) {
            Ok(ImdQuintile(v))
        } else {
            Err(serde::de::Error::custom(format!(
                "IMD quintile must be 1-5, got {v}"
            )))
        }
    }
}

#[derive(Debug, Deserialize)]
struct PatientRecord {
    patient_id: String,
    smoking_status: SmokingStatus,
    imd_quintile: ImdQuintile,
}

fn main() {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/patients.csv")
        .expect("could not open CSV file");

    for (i, result) in reader.deserialize::<PatientRecord>().enumerate() {
        let line = i + 2;
        match result {
            Ok(record) => {
                println!(
                    "line {line}: {} smoking={:?} imd={}",
                    record.patient_id, record.smoking_status, record.imd_quintile.0
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

    // Helper: deserialise a SmokingStatus from a CSV row
    fn parse_smoking(input: &str) -> Result<SmokingStatus, csv::Error> {
        let csv_data = format!("smoking_status\n{input}");
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        let record: SmokingStatus = reader.deserialize().next().unwrap()?;
        Ok(record)
    }

    // Helper: deserialise an ImdQuintile from a CSV row
    fn parse_imd(input: &str) -> Result<ImdQuintile, csv::Error> {
        let csv_data = format!("imd_quintile\n{input}");
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        let record: ImdQuintile = reader.deserialize().next().unwrap()?;
        Ok(record)
    }

    #[test]
    fn smoking_canonical_values() {
        assert_eq!(parse_smoking("never").unwrap(), SmokingStatus::Never);
        assert_eq!(parse_smoking("former").unwrap(), SmokingStatus::Former);
        assert_eq!(parse_smoking("current").unwrap(), SmokingStatus::Current);
        assert_eq!(parse_smoking("unknown").unwrap(), SmokingStatus::Unknown);
    }

    #[test]
    fn smoking_aliases_never() {
        for input in ["non-smoker", "nonsmoker", "never smoked"] {
            assert_eq!(
                parse_smoking(input).unwrap(),
                SmokingStatus::Never,
                "expected Never for '{input}'"
            );
        }
    }

    #[test]
    fn smoking_aliases_former() {
        for input in ["ex", "ex-smoker", "former smoker"] {
            assert_eq!(
                parse_smoking(input).unwrap(),
                SmokingStatus::Former,
                "expected Former for '{input}'"
            );
        }
    }

    #[test]
    fn smoking_aliases_current() {
        for input in ["smoker", "active", "current smoker"] {
            assert_eq!(
                parse_smoking(input).unwrap(),
                SmokingStatus::Current,
                "expected Current for '{input}'"
            );
        }
    }

    #[test]
    fn smoking_aliases_unknown() {
        for input in ["NA", "not recorded"] {
            assert_eq!(
                parse_smoking(input).unwrap(),
                SmokingStatus::Unknown,
                "expected Unknown for '{input}'"
            );
        }
    }

    #[test]
    fn smoking_rejects_invalid() {
        assert!(parse_smoking("occasionally").is_err());
        assert!(parse_smoking("social smoker").is_err());
    }

    #[test]
    fn imd_valid_range() {
        for v in 1..=5 {
            let result = parse_imd(&v.to_string()).unwrap();
            assert_eq!(result.0, v);
        }
    }

    #[test]
    fn imd_rejects_zero() {
        assert!(parse_imd("0").is_err());
    }

    #[test]
    fn imd_rejects_six() {
        assert!(parse_imd("6").is_err());
    }
}