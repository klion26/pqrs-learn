use std::fmt;
use std::fmt::Formatter;
use std::path::PathBuf;

use clap::{Arg, ArgMatches, Parser};
use log::debug;

use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, get_row_count, open_file};

#[derive(Parser, Debug)]
pub struct RowCountCommandArgs {
    /// parquet files to read
    files: Vec<PathBuf>,
}

pub fn execute(opts: RowCountCommandArgs) -> Result<(), PQRSError> {
    debug!("The files to read are {:#?}", opts.files);

    for file_name in &opts.files {
        if !check_path_present(file_name) {
            return Err(FileNotFound(file_name.to_path_buf()));
        }
    }

    for file_name in &opts.files {
        let file = open_file(file_name)?;
        let row_count = get_row_count(file)?;

        println!("File Name:{}, {} rows", file_name.display(), row_count);
    }

    Ok(())
}
