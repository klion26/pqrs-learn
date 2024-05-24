use std::fmt::Formatter;
use crate::errors::PQRSError::{CouldNotOpenFile, UnsupportedOperation};
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::record::Row;
use parquet::arrow::{ArrowWriter};
use arrow::{datatypes::Schema, record_batch::RecordBatch};
use std::fs::File;
use std::io::ErrorKind::Unsupported;
use std::path::Path;
use log::debug;
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::ops::Add;
use std::sync::Arc;
use parquet::arrow::arrow_reader::ArrowReaderBuilder;
use arrow::csv;
use tempfile::NamedTempFile;
use std::io::Read;
use walkdir::DirEntry;
use crate::errors::PQRSError;

// can this be implement by enum, then implement format function for enum?
static ONE_KI_B: i64 = 1024;
static ONE_MI_B: i64 = ONE_KI_B * 1024;
static ONE_GI_B: i64 = ONE_MI_B * 1024;
static ONE_TI_B: i64 = ONE_GI_B * 1024;
static ONE_PI_B: i64 = ONE_TI_B * 1024;

// output formats supported. Only cat command support CSV format.
#[derive(Debug)]
pub enum Formats {
    Default,
    Csv,
    Json,
}

impl std::fmt::Display for Formats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?}", self)
    }
}


/// check if a particular path is present on the filesystem
pub fn check_path_present(file_path: &str) -> bool {
    Path::new(file_path).exists()
}

pub fn open_file(file_name: &str) -> Result<File, PQRSError> {
    let path = Path::new(&file_name);
    let file = match File::open(&path) {
        Err(_) => return Err(CouldNotOpenFile(file_name.to_string())),
        Ok(f) => f
    };

    Ok(file)
}

pub fn print_rows(
    file: File,
    num_records: Option<i64>,
    format: &Formats) -> Result<(), PQRSError> {
    match format {
        Formats::Default | Formats::Json => {
            let parquet_reader = SerializedFileReader::new(file)?;
            let mut iter = parquet_reader.get_row_iter(None)?;

            let mut start: i64 = 0;
            let end: i64 = num_records.unwrap_or(0);
            let all_records = num_records.is_none();

            while all_records || start < end {
                match iter.next() {
                    Some(row) => print_row(&row, format),
                    None => break,
                }

                start += 1;
            }
        }
        Formats::Csv => {
            if num_records.is_some() {
                return Err(UnsupportedOperation())
            } else {
                let output = print_csv(file);
                if output.is_err() {
                    println!("{:?}", output);
                }
            }
        }
    }

    Ok(())
}


pub fn print_csv(
    file: File
) -> Result<(), PQRSError> {
    let data = get_row_batches(file)?;
    let output = NamedTempFile::new()?;

    let mut writer = csv::Writer::new(&output);
    for batch in &data.batches {
        writer.write(batch)?;
    }

    let mut buf = String::new();
    let mut resutl = output.reopen()?;
    resutl.read_to_string(&mut buf)?;

    if buf.len() == 0 {
        println!("Empty.");
    } else {
        println!("{}", buf);
    }

    Ok(())
}
// check if the given entry in the walking tree is a hidden file
pub fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn print_row(
    row: &Row,
    format: &Formats) {
    match format {
        Formats::Json => println!("{}", row.to_json_value()),
        Formats::Default => println!("{}", row.to_string()),
        Formats::Csv => println!("Unsupported! {}", row.to_string()),
    }
}

pub fn get_row_count(file: File) -> Result<i64, PQRSError> {
    let parquet_reader = SerializedFileReader::new(file)?;
    let row_group_metadata = parquet_reader.metadata().row_groups();
    let total_num_rows = row_group_metadata.iter().map(|rg| rg.num_rows()).sum();

    Ok(total_num_rows)
}

pub fn get_size(file: File) -> Result<(i64, i64), PQRSError> {
    let parquet_reader = SerializedFileReader::new(file)?;
    let row_group_metadata = parquet_reader.metadata().row_groups();

    let uncompressed_size = row_group_metadata
        .iter()
        .map(|rg| rg.total_byte_size())
        .sum();
    let compressed_size = row_group_metadata
        .iter()
        .map(|rg| rg.compressed_size())
        .sum();

    Ok((uncompressed_size, compressed_size))
}

pub fn get_pretty_size(bytes: i64) -> String {
    if bytes / ONE_KI_B < 1 {
        return format!("{} Bytes", bytes)
    }

    if bytes / ONE_MI_B < 1 {
        return format!("{:.3} KiB", bytes / ONE_KI_B);
    }

    if bytes / ONE_GI_B < 1 {
        return format!("{:.3} MiB", bytes / ONE_MI_B);
    }

    if bytes / ONE_TI_B < 1 {
        return format!("{:.3} GiB", bytes / ONE_GI_B);
    }

    if bytes / ONE_PI_B < 1 {
        return format!("{:.3} TiB", bytes / ONE_TI_B);
    }

    return format!("{:.3} PiB", bytes / ONE_PI_B);
}

pub fn print_rows_random(
    file: File,
    sample_size: i64,
    format: &Formats
) -> Result<(), PQRSError> {
    let parquet_reader = SerializedFileReader::new(file.try_clone()?)?;
    let mut iter = parquet_reader.get_row_iter(None)?;

    let total_records_in_file: i64 = get_row_count(file)?;
    let mut indexes = (0..total_records_in_file).collect::<Vec<_>>();

    let mut rng = thread_rng();
    indexes.shuffle(&mut rng);

    indexes = indexes
        .into_iter()
        .take(sample_size as usize)
        .collect::<Vec<_>>();

    debug!("Sampled indexes: {:#?}", indexes);

    let mut start: i64 = 0;
    while let Some(row) = iter.next() {
        if indexes.contains(&start) {
            print_row(&row, format)
        }
        start += 1;
    }

    Ok(())

}

#[derive(Debug)]
pub struct ParquetData {
    pub schema: Schema,
    pub batches: Vec<RecordBatch>,
    pub rows: usize,
}

impl Add for ParquetData {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        let mut combined_data = Vec::new();
        combined_data.append(&mut self.batches);
        combined_data.append(&mut rhs.batches);

        Self {
            schema: self.schema,
            batches: combined_data,
            rows: self.rows + rhs.rows
        }
    }
}

pub fn get_row_batches(input: File) -> Result<ParquetData, PQRSError> {
    let arrow_reader = ArrowReaderBuilder::try_new(input)?;

    let schema = Schema::clone(arrow_reader.schema());
    let record_batch_reader = arrow_reader.with_batch_size(1024).build()?;
    let mut batches: Vec<RecordBatch> = Vec::new();

    let mut rows = 0;
    for maybe_batch in record_batch_reader {
        let record_batch = maybe_batch?;
        rows += record_batch.num_rows();

        batches.push(record_batch);
    }

    Ok(ParquetData {
        schema,
        batches,
        rows
    })
}

pub fn write_parquet(data: ParquetData, output: &str) -> Result<(), PQRSError> {
    let file = File::create(output)?;
    let fields = data.schema.fields().to_vec();
    let schema_without_metadata = Schema::new(fields);

    let mut writer = ArrowWriter::try_new(file, Arc::new(schema_without_metadata), None)?;

    for record_batch in data.batches.iter() {
        writer.write(&record_batch)?;
    }

    writer.close()?;
    Ok(())
}