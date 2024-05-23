use std::fmt;
use std::fmt::Formatter;
use std::fs::File;
use std::sync::Arc;
use arrow::datatypes::Schema;
use clap::{App, Arg, ArgMatches, SubCommand};
use log::debug;
use parquet::arrow::ArrowWriter;
use crate::errors::PQRSError;
use crate::errors::PQRSError::{FileExists, FileNotFound};
use crate::utils::{check_path_present, get_row_batches, open_file};
use crate::command::PQRSCommand;

pub struct MergeCommand<'a> {
    inputs: Vec<&'a str>,
    output: &'a str,
}

impl<'a> MergeCommand<'a> {
    pub(crate) fn command() -> App<'static, 'static> {
        SubCommand::with_name("merge")
            .about("Merge file(s) into another parquet file")
            .arg(
                Arg::with_name("input")
                    .short("i")
                    .long("input")
                    .multiple(true)
                    .value_name("INPUT")
                    .value_delimiter(" ")
                    .required(true)
                    .help("Parquet files to read"),
            )
            .arg(
                Arg::with_name("output")
                    .short("o")
                    .long("output")
                    .value_name("OUTPUT")
                    .required(true)
                    .help("Parquet file to write"),
            )
    }

    pub(crate)  fn new(matchers: &'a ArgMatches<'a>) -> Self {
        Self {
            inputs: matchers.values_of("input").unwrap().collect(),
            output: matchers.value_of("output").unwrap(),
        }
    }

}

impl<'a> PQRSCommand for MergeCommand<'a> {
    fn execute(&self) -> Result<(), PQRSError> {
        debug!("{:#?}", self);

        if check_path_present(self.output) {
            return Err(FileExists(self.output.to_string()))
        }

        for file_name in &self.inputs {
            if !check_path_present(*file_name) {
                return Err(FileNotFound(String::from(*file_name)))
            }
        }

        let mut writer = {
            let seed = open_file(self.inputs[0])?;
            let data = get_row_batches(seed)?;

            let file = File::create(&self.output)?;
            let fields = data.schema.fields.to_vec();

            let schema_without_metadata = Schema::new(fields);

            let mut writer = ArrowWriter::try_new(file, Arc::new(schema_without_metadata), None)?;

            for record_batch in data.batches.iter() {
                writer.write(record_batch)?;
            }

            writer
        };

        for input in &self.inputs[1..] {
            let current = open_file(input)?;
            let local = get_row_batches(current)?;

            for record_batch in local.batches.iter() {
                writer.write(record_batch)?;
            }

        }

        writer.close()?;

        Ok(())

    }
}


impl<'a> fmt::Debug for MergeCommand<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "The file names to read are:{}", &self.inputs.join(", "));
        writeln!(f, "The file name to write to: {}", &self.output);

        Ok(())
    }
}