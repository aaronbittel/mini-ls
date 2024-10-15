use mini_ls::Config;
use std::env;

fn main() {
    let config = Config::build(env::args()).unwrap_or_else(|err| {
        eprintln!("could not parse arguments: {err:?}");
        std::process::exit(1);
    });

    mini_ls::run(config).unwrap_or_else(|err| {
        eprintln!("could not run command: {err:?}");
    });
}
