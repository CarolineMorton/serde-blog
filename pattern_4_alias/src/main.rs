use csv::ReaderBuilder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct HospitalEpisode {
    patient_id: String,

    /// Source column is just "date", but we know this is the admission date.
    #[serde(rename = "date")]
    admission_date: String,

    #[serde(rename = "disch_date")]
    discharge_date: String,

    #[serde(rename = "diag_1")]
    primary_diagnosis: String,
}

#[derive(Debug, Deserialize)]
struct GpConsultation {
    patient_id: String,

    /// Also "date" in the source, but here it means the consultation date.
    #[serde(rename = "date")]
    consultation_date: String,

    #[serde(rename = "code")]
    clinical_code: String,
}

fn main() {
    println!("=== Hospital Episodes ===\n");

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/hospital.csv")
        .expect("could not open hospital CSV");

    for (i, result) in reader.deserialize::<HospitalEpisode>().enumerate() {
        let line = i + 2;
        match result {
            Ok(ep) => {
                println!(
                    "  line {line}: {} admission_date={} discharge_date={} diagnosis={}",
                    ep.patient_id, ep.admission_date, ep.discharge_date, ep.primary_diagnosis
                );
            }
            Err(e) => println!("  line {line}: [ERROR] {e}"),
        }
    }

    println!("\n=== GP Consultations ===\n");

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path("data/gp.csv")
        .expect("could not open GP CSV");

    for (i, result) in reader.deserialize::<GpConsultation>().enumerate() {
        let line = i + 2;
        match result {
            Ok(gp) => {
                println!(
                    "  line {line}: {} consultation_date={} code={}",
                    gp.patient_id, gp.consultation_date, gp.clinical_code
                );
            }
            Err(e) => println!("  line {line}: [ERROR] {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_hospital(csv_data: &str) -> Result<HospitalEpisode, csv::Error> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        reader.deserialize().next().unwrap()
    }

    fn parse_gp(csv_data: &str) -> Result<GpConsultation, csv::Error> {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(csv_data.as_bytes());
        reader.deserialize().next().unwrap()
    }

    #[test]
    fn hospital_renames_date_to_admission_date() {
        let csv = "patient_id,date,disch_date,diag_1\nP001,2024-03-15,2024-03-20,E119";
        let ep = parse_hospital(csv).unwrap();
        assert_eq!(ep.admission_date, "2024-03-15");
        assert_eq!(ep.discharge_date, "2024-03-20");
        assert_eq!(ep.primary_diagnosis, "E119");
    }

    #[test]
    fn gp_renames_date_to_consultation_date() {
        let csv = "patient_id,date,code\nP001,2024-03-18,XE0of";
        let gp = parse_gp(csv).unwrap();
        assert_eq!(gp.consultation_date, "2024-03-18");
        assert_eq!(gp.clinical_code, "XE0of");
    }

    #[test]
    fn hospital_fails_if_column_missing() {
        let csv = "patient_id,admission_date,disch_date,diag_1\nP001,2024-03-15,2024-03-20,E119";
        // "admission_date" is the struct field name, but the CSV should have "date"
        assert!(parse_hospital(csv).is_err());
    }

    #[test]
    fn gp_fails_if_column_missing() {
        let csv = "patient_id,consultation_date,code\nP001,2024-03-18,XE0of";
        // "consultation_date" is the struct field name, but the CSV should have "date"
        assert!(parse_gp(csv).is_err());
    }
}