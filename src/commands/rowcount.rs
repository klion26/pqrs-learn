use std::fmt;
use std::fmt::Formatter;
use clap::{App, Arg, ArgMatches, SubCommand};
use log::debug;
use crate::errors::PQRSError;
use crate::errors::PQRSError::FileNotFound;
use crate::utils::{check_path_present, get_row_count, open_file};

use crate::command::PQRSCommand;


pub struct RowCountCommand<'a> {
    file_names: Vec<&'a str>,
}

impl <'a> RowCountCommand<'a> {
    pub(crate) fn command() -> App<'static, 'static> {
        SubCommand::with_name("rowcount")
            .about("Prints the count of rows in Parquet file(s)")
            .arg(
                Arg::with_name("files")
                    .index(1)
                    .multiple(true)
                    .value_name("FILES")
                    .value_delimiter(" ")
                    .required(true)
                    .help("Parquet files to read"),
            )
    }

    pub(crate) fn new(matchers: &'a ArgMatches<'a>) -> Self {
        Self {
            file_names: matchers.values_of("files").unwrap().collect(),
        }
    }
}

impl<'a> PQRSCommand for RowCountCommand<'a> {
    fn execute(&self) -> Result<(), PQRSError> {
        debug!("{:#?}", self);

        for file_name in &self.file_names {
            if !check_path_present(*file_name) {
                return Err(FileNotFound(String::from(*file_name)))
            }
        }

        for file_name in &self.file_names {
            let file = open_file(file_name)?;
            let row_count = get_row_count(file)?;

            println!("File Name:{file_name}, {row_count} rows");
        }

        Ok(())
    }
}

impl<'a> fmt::Debug for RowCountCommand<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "The file names to read are: {}",
            self.file_names.join(", ")
        )
    }
}