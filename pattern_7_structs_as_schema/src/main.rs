use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::{Deserialize, Deserializer};

/// Flexible boolean deserializer that accepts various common representations of true/false.
/// We saw this in pattern_2_dates
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

/// HospitalEpisode struct representing a single row of the CSV, but we 
/// only include the fields we care about. We compute length_of_stay ourselves
/// from the admission and discharge dates, rather than taking it from the CSV.
#[derive(Debug, Deserialize)]
struct HospitalEpisode {
    patient_id: String,
    episode_id: String,

    #[serde(deserialize_with = "parse_clinical_date")]
    admission_date: NaiveDate,

    #[serde(deserialize_with = "parse_clinical_date")]
    discharge_date: NaiveDate,

    primary_diagnosis: String,

    #[serde(deserialize_with = "flexible_bool")]
    is_emergency: bool,

    // length_of_stay is in the source CSV but we don't include it here.
    // It simply doesn't exist in our program. We compute it ourselves.
}

impl HospitalEpisode {
    fn length_of_stay(&self) -> i64 {
        (self.discharge_date - self.admission_date).num_days()
    }
}

fn main() {
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/episodes.csv")
        .expect("could not open CSV file");

    for (i, result) in reader.deserialize::<HospitalEpisode>().enumerate() {
        let line = i + 2;
        match result {
            Ok(ep) => {
                println!(
                    "line {line}: {} ({}) admitted={} discharged={} los={} days emergency={} diagnosis={}",
                    ep.patient_id,
                    ep.episode_id,
                    ep.admission_date,
                    ep.discharge_date,
                    ep.length_of_stay(),
                    ep.is_emergency,
                    ep.primary_diagnosis,
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

    fn parse(csv_data: &str) -> Result<HospitalEpisode, csv::Error> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        reader.deserialize().next().unwrap()
    }

    #[test]
    fn ignores_extra_columns() {
        let csv = "patient_id,episode_id,admission_date,discharge_date,primary_diagnosis,is_emergency,length_of_stay,junk_column\n\
                   P001,E001,2024-03-15,2024-03-20,E119,yes,5,anything";
        let ep = parse(csv).unwrap();
        assert_eq!(ep.patient_id, "P001");
        assert_eq!(ep.admission_date, NaiveDate::from_ymd_opt(2024, 3, 15).unwrap());
    }

    #[test]
    fn computes_length_of_stay() {
        let csv = "patient_id,episode_id,admission_date,discharge_date,primary_diagnosis,is_emergency\n\
                   P001,E001,2024-03-15,2024-03-20,E119,yes";
        let ep = parse(csv).unwrap();
        assert_eq!(ep.length_of_stay(), 5);
    }

    #[test]
    fn same_day_admission_is_zero_days() {
        let csv = "patient_id,episode_id,admission_date,discharge_date,primary_diagnosis,is_emergency\n\
                   P001,E001,2024-05-10,2024-05-10,K359,true";
        let ep = parse(csv).unwrap();
        assert_eq!(ep.length_of_stay(), 0);
    }

    #[test]
    fn source_length_of_stay_is_ignored() {
        // The source says length_of_stay is -1 (clearly wrong).
        // Our computed value from the dates is correct.
        let csv = "patient_id,episode_id,admission_date,discharge_date,primary_diagnosis,is_emergency,length_of_stay\n\
                   P005,E005,2024-01-10,2024-01-15,M545,y,-1";
        let ep = parse(csv).unwrap();
        assert_eq!(ep.length_of_stay(), 5);
    }

    #[test]
    fn works_with_mixed_date_formats() {
        let csv = "patient_id,episode_id,admission_date,discharge_date,primary_diagnosis,is_emergency\n\
                   P003,E003,15/03/2024,22/03/2024,J45,1";
        let ep = parse(csv).unwrap();
        assert_eq!(ep.length_of_stay(), 7);
    }

    #[test]
    fn works_with_mixed_boolean_formats() {
        let csv = "patient_id,episode_id,admission_date,discharge_date,primary_diagnosis,is_emergency\n\
                   P001,E001,2024-03-15,2024-03-20,E119,yes";
        let ep = parse(csv).unwrap();
        assert!(ep.is_emergency);

        let csv = "patient_id,episode_id,admission_date,discharge_date,primary_diagnosis,is_emergency\n\
                   P002,E002,2024-04-01,2024-04-03,I10,no";
        let ep = parse(csv).unwrap();
        assert!(!ep.is_emergency);
    }
}