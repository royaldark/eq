use clap::{value_t, App, Arg};

use super::input;
use super::output;
use super::transform;

crate struct EqOptions {
    crate input: input::InputOptions,
    crate output: output::OutputOptions,
    crate transform: transform::TransformOptions,
}

crate fn parse_opts() -> EqOptions {
    let matches = App::new("eq (jq for edn)")
        .version("0.1")
        .author("Joe Einertson <joe@einertson>")
        .about("jq for EDN")
        .arg(
            Arg::with_name("expression")
                .help("eq expression which will transform the input")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("input_path")
                .help("Input path (JSON, EDN, etc.) to read from")
                .required(true)
                .index(2),
        )
        .arg(
            Arg::with_name("input_format")
                .help("Input data format")
                .short("i")
                .long("input-format")
                .takes_value(true)
                .case_insensitive(true)
                .possible_values(&input::InputFormat::variants()),
        )
        .arg(
            Arg::with_name("output_style")
                .help("Output data style")
                .long("output-style")
                .takes_value(true)
                .case_insensitive(true)
                .default_value("pretty")
                .possible_values(&output::OutputStyle::variants()),
        )
        .arg(
            Arg::with_name("output_format")
                .help("Output data format")
                .short("o")
                .long("output-format")
                .takes_value(true)
                .case_insensitive(true)
                .default_value("EDN")
                .possible_values(&output::OutputFormat::variants()),
        )
        .arg(
            Arg::with_name("color")
                .help("Colorize output?")
                .short("c")
                .long("color")
                .takes_value(true)
                .case_insensitive(true)
                .default_value("default")
                .possible_values(&["default", "always", "never"]),
        )
        .get_matches();

    match matches.value_of("color").unwrap() {
        "always" => colored::control::set_override(true),
        "never" => colored::control::set_override(false),
        _ => (),
    }

    EqOptions {
        input: input::InputOptions {
            format: value_t!(matches.value_of("input_format"), input::InputFormat).unwrap(),
            path: matches.value_of("input_path").unwrap().into(),
        },
        output: output::OutputOptions {
            format: value_t!(matches.value_of("output_format"), output::OutputFormat).unwrap(),
            style: value_t!(matches.value_of("output_style"), output::OutputStyle).unwrap(),
            destination: output::OutputDestination::Stdout,
        },
        transform: transform::TransformOptions {
            expression: matches.value_of("expression").unwrap().into(),
        },
    }
}
