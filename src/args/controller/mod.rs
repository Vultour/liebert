extern crate clap;

use clap::{App, Arg, ArgMatches};

use ::types;


fn program_args<'b>() -> ArgMatches<'b>{
    super::program_args(
        App::new("Liebert Controller")
    )

    .get_matches()
}

fn check_args(args: ArgMatches) -> super::ArgsResult{
    let mut output_args = types::ConfigurationMap::new();
    let mut global_args = super::check_args(args).unwrap();

    output_args.extend(global_args);
    Ok(output_args)
}

pub fn get_args() -> super::ArgsResult{
    check_args(program_args())
}
