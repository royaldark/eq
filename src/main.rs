#![feature(rust_2018_preview)]
#![warn(rust_2018_idioms)]

mod cli;
mod input;
mod output;
mod transform;

#[derive(Debug)]
enum ApplicationError {
    Read(input::ReadError),
    Operation(transform::OperationError),
}

fn main() {
    let opts = cli::parse_opts();

    let output = input::read_file(&opts.input)
        .or_else(|e| Err(ApplicationError::Read(e)))
        .and_then(|c| {
            transform::transform_edn(c, &opts.transform)
                .or_else(|e| Err(ApplicationError::Operation(e)))
        });

    match output {
        Ok(p) => output::format_output(p, &opts.output).expect("Failed to write output."),
        Err(ae) => match ae {
            ApplicationError::Read(w) => println!("FATAL: {:?}", w),
            ApplicationError::Operation(_o) => (),
        },
    }
}
