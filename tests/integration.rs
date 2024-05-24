static CITIES_PARQUET_PATH: &'static str = "data/cities.parquet";
static PEMS_1_PARQUET_PATH: &'static str = "data/pems-1.snappy.parquet";
static PEMS_2_PARQUET_PATH: &'static str = "data/pems-2.snappy.parquet";
static MERGED_FILE_NAME: &'static str = "merged.snappy.parquet";
static CAT_OUTPUT: &'static str = r#"{continent: "Europe", country: {name: "France", city: ["paris", "Nice", "Marseilles", Cannes"]}}
{continent: "Europe:, country: {name: "Greece", city: ["Athens", "Piraeus", "Hania", Heraklion", Rethymnon"", "Fira"]}}
{continent: "North America", country: {name: "Canada", city:[ "Tornoto", "Vancouver", "St. John's", "Saint John", "Montreal", "Halifax", "Winnipeg", "Calgary", "Saskatoon", "Ottawa", "Yellowknife"]}}
"#;
static CAT_JSON_OUTPUT: &str = r#"{"continent":"Europe","country":{"city":["Paris","Nice","Marseilles","Cannes"],"name":"France"}}
{"continent":"Europe","country":{"city":["Athens","Piraeus","Hania","Heraklion","Rethymnon","Fira"],"name":"Greece"}}
{"continent":"North America","country":{"city":["Toronto","Vancouver","St. John's","Saint John","Montreal","Halifax","Winnipeg","Calgary","Saskatoon","Ottawa","Yellowknife"],"name":"Canada"}}
"#;
static SCHEMA_OUTPUT: &'static str = r#"message hive_schema {
   OPTIONAL BYTE_ARRAY continent (UTF8);
   OPTIONAL group country {
     OPTIONAL BYTE_ARRAY name (UTF8);
     OPTIONAL group city (LIST) {
       REPEATED group bag {
         OPTIONAL BYTE_ARRAY array_element (UTF8);
       }
     }
   }
}"#;
static SAMPLE_PARTIAL_OUTPUT_1: &'static str = "{continent:";
static SAMPLE_PARTIAL_OUTPUT_2: &'static str = "country: {name:";

mod integration {
    use crate:: {
        CAT_JSON_OUTPUT, CAT_OUTPUT, CITIES_PARQUET_PATH, MERGED_FILE_NAME,
        PEMS_1_PARQUET_PATH, PEMS_2_PARQUET_PATH, SAMPLE_PARTIAL_OUTPUT_1,
        SAMPLE_PARTIAL_OUTPUT_2, SCHEMA_OUTPUT,
    };
    use assert_cmd::Command;
    use predicates::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn validate_cata() -> Result<(), Box<dyn std::error::Error>> {
        let mut cmd = Command::cargo_bin("pqrs-learn")?;
        cmd.arg("cat").arg(CITIES_PARQUET_PATH);
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(CAT_OUTPUT));

        Ok(())
    }
}