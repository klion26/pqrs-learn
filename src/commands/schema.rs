use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::hash::Hash;
use std::io::BufWriter;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Parser};
use log::debug;
use parquet::file::metadata::ParquetMetaData;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::schema::printer::{print_file_metadata, print_parquet_metadata, print_schema};
use serde::{Deserialize, Serialize};

use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, open_file};

#[derive(Parser, Debug)]
pub struct SchemaCommandArgs {
    files: Vec<PathBuf>,
    #[arg(short = 'D', long)]
    detailed: bool,
    #[arg(short, long, conflicts_with = "detailed")]
    json: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ParquetSchema {
    version: i32,
    num_rows: i64,
    created_by: Option<String>,
    metadata: Option<HashMap<String, Option<String>>>,
    columns: Vec<HashMap<String, String>>,
    message: String,
}

fn get_schema_metadata(metadata: &ParquetMetaData) -> Option<HashMap<String, Option<String>>> {
    if let Some(metadata) = metadata.file_metadata().key_value_metadata() {
        let mut fields: HashMap<String, Option<String>> = HashMap::new();
        for kv in metadata.iter() {
            fields.insert(kv.key.to_string(), kv.value.to_owned());
        }
        Some(fields)
    } else {
        None
    }
}

fn get_column_information(metadata: &ParquetMetaData) -> Vec<HashMap<String, String>> {
    let schema = metadata.file_metadata().schema_descr();
    let mut columns = Vec::new();
    for (_i, col) in schema.columns().iter().enumerate() {
        let mut column_info: HashMap<String, String> = HashMap::new();
        column_info.insert(String::from("name"), String::from(col.name()));
        column_info.insert(String::from("path"), col.path().string());
        column_info.insert(String::from("optional"), col.self_type().is_optional().to_string());
        column_info.insert(String::from("physical_type"), col.physical_type().to_string());
        column_info.insert(String::from("converted_type"), col.converted_type().to_string());
        columns.push(column_info)
    }
    columns
}

fn get_message(metadata: &ParquetMetaData) -> Result<String, PQRSError> {
    let mut buf = BufWriter::new(Vec::new());
    print_schema(&mut buf, metadata.file_metadata().schema());
    let byte = buf.into_inner()?;
    Ok(String::from_utf8(byte)?)
}
pub fn execute(opts: SchemaCommandArgs) -> Result<(), PQRSError> {
    debug!("The file names to read are: {:?}", opts.files);
    debug!("Print detailed output:{:?}", opts.detailed);

    for file_name in &opts.files {
        if !check_path_present(file_name) {
            return Err(FileNotFound(file_name.to_path_buf()));
        }
    }

    for file_name in &opts.files {
        let file = open_file(file_name)?;
        match SerializedFileReader::new(file) {
            Ok(parquet_reader) => {
                let metadata = parquet_reader.metadata();
                if opts.json {
                    let schema = ParquetSchema {
                        version: metadata.file_metadata().version(),
                        num_rows: metadata.file_metadata().num_rows(),
                        created_by: metadata
                            .file_metadata()
                            .created_by()
                            .map(|str| str.to_string()),
                        metadata: get_schema_metadata(metadata),
                        columns: get_column_information(metadata),
                        message: get_message(metadata)?,
                    };
                    let schema_json = serde_json::to_string(&schema)?;
                    println!("{}", schema_json);
                } else {
                    println!("Metadata for file: {}", file_name.display());
                    println!();
                    if opts.detailed {
                        print_parquet_metadata(&mut std::io::stdout(), metadata);
                    } else {
                        print_file_metadata(&mut std::io::stdout(), metadata.file_metadata())
                    }
                }
            }
            Err(e) => { return Err(PQRSError::ParquetError(e)); }
        }
    }

    Ok(())
}
