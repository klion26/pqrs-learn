use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use arrow::datatypes::Schema;
use clap::Parser;
use log::debug;
use parquet::arrow::ArrowWriter;
use crate::errors::PQRSError;
use crate::errors::PQRSError::{FileExists, FileNotFound};
use crate::utils::{check_path_present, get_row_batches, open_file};

#[derive(Parser, Debug)]
pub struct MergeCommandArgs {
    #[clap(short, long, value_delimiter = ' ', num_args = 1..)]
    input: Vec<PathBuf>,
    #[clap(short, long)]
    output: PathBuf,
}

pub(crate) fn execute(opts: MergeCommandArgs) -> Result<(), PQRSError> {
    debug!("The file names to read are:{:?}", &opts.input);
    debug!( "The file name to write to: {}", &opts.output.display());

    if check_path_present(&opts.output) {
        return Err(FileExists(opts.output.to_path_buf()));
    }

    for file_name in &opts.input {
        if !check_path_present(file_name) {
            return Err(FileNotFound(file_name.to_path_buf()));
        }
    }

    let mut writer = {
        let seed = open_file(&opts.input[0])?;
        let data = get_row_batches(seed)?;

        let file = File::create(&opts.output)?;
        let fields = data.schema.fields.to_vec();

        let schema_without_metadata = Schema::new(fields);

        let mut writer = ArrowWriter::try_new(file, Arc::new(schema_without_metadata), None)?;

        for record_batch in data.batches.iter() {
            writer.write(record_batch)?;
        }

        writer
    };

    for input in &opts.input[1..] {
        let current = open_file(input)?;
        let local = get_row_batches(current)?;

        for record_batch in local.batches.iter() {
            writer.write(record_batch)?;
        }
    }

    writer.close()?;

    Ok(())
}