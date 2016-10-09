extern crate clap;

use clap::{App, Arg, ArgMatches};

use super::types;

mod defaults;
pub mod agent;
pub mod controller;


pub type ArgsResult = Result<types::ConfigurationMap, String>;


fn program_args<'a, 'b, 'c>(mut app: App<'a, 'b>) -> App<'b, 'c>{
    app
        .version(env!("CARGO_PKG_VERSION"))
        .author("(c) Martin Kukura")
        .arg(
            Arg::with_name("conf")
            .short("c")
            .long("conf")
            .takes_value(true)
            .help("Path to the configuration file")
        ).arg(
            Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .takes_value(false)
            .multiple(true)
            .help("Verbose output")
        )
}

fn check_args<'a>(args: ArgMatches) -> ArgsResult{
    let mut output_args = types::ConfigurationMap::new();

    output_args.insert(String::from(".args.config"),    String::from(args.value_of("conf").unwrap_or(defaults::PATH_CONFIG)));
    output_args.insert(String::from(".args.verbose"),   String::from(format!("{}", args.occurrences_of("verbose").to_string())));

    Ok(output_args)
}
