use std::env;
use std::process;

fn main() {
    // get all the cli arguments
    let config = expression_evaluation::Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    // evaluate the received expression
    if let Err(e) = expression_evaluation::run(config) {
        eprintln!("Application error: {e}");
        process::exit(1);
    }
}
