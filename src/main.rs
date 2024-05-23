mod errors;
mod utils;
mod command;
mod commands;

use clap::AppSettings::ArgRequiredElseHelp;
use clap::{crate_authors, crate_version, App, Arg, SubCommand, value_t};
use env_logger::Env;

use crate::command::run_command;
use crate::errors::PQRSError;

fn main() -> Result<(), PQRSError> {

    let matches = App::new("pqrs")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Apache Parquet command-line utility")
        .setting(ArgRequiredElseHelp)
        .arg(
            Arg::with_name("debug")
                .short("dd")
                .long("debug")
                .takes_value(false)
                .global(true)
                .help("Show debug output"),
        )
        .subcommands(
            vec![
                commands::cat::CatCommand::command(),
                commands::schema::SchemaCommand::command(),
                commands::head::HeadCommand::command(),
                commands::rowcount::RowCountCommand::command(),
                commands::size::SizeCommand::command(),
                commands::sample::SampleCommand::command(),
                commands::merge::MergeCommand::command(),
            ]
        )
        .get_matches();


    let mut env = Env::default();
    if matches.is_present("debug") {
        env = env.default_filter_or("debug");
    } else {
        env = env.default_filter_or("info");
    }
    env_logger::Builder::from_env(env).init();
    run_command(matches)?;

    Ok(())
}
