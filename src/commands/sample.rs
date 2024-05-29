use std::path::PathBuf;

use clap::{Parser};
use log::debug;

use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, Formats, open_file, print_rows_random};

// prints a random sample of records from the parquet file
#[derive(Parser, Debug)]
pub struct SampleCommandArgs {
    file: PathBuf,

    #[arg(short = 'n', long)]
    records: usize,

    #[arg(short, long)]
    json: bool,
}

pub fn execute(opts: SampleCommandArgs) -> Result<(), PQRSError> {
    let format = if opts.json {
        Formats::Json
    } else {
        Formats::Default
    };

    debug!("The file name to read is :{}", opts.file.display());
    debug!("Number of records to print: {}", opts.records);
    debug!("Output format :{}", format);

    if !check_path_present(&opts.file) {
        return Err(FileNotFound(opts.file.to_path_buf()));
    }

    let file = open_file(&opts.file)?;
    print_rows_random(file, opts.records, format)?;

    Ok(())
}
