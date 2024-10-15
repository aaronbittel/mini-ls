use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::path;

#[allow(non_snake_case)]
#[derive(Debug, PartialEq)]
pub struct Config {
    path: path::PathBuf,
    l: bool,
    a: bool,
    F: bool,
    h: bool,
    S: bool,
    R: bool,
    r: bool,
    t: bool,
    d: bool,
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut entries: Vec<fs::DirEntry> = vec![];
    for entry in fs::read_dir(&config.path)? {
        let entry = entry?;
        if let Some(e) = entry.file_name().to_str() {
            if !e.starts_with(".") {
                entries.push(entry);
            }
        }
    }

    let sep = if config.l { "\n" } else { "  " };

    entries.sort_by_key(|e| e.file_name());

    const blue: &str = "\x1b[1;38;5;12m";
    const reset: &str = "\x1b[0m";

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for entry in entries {
        if let Some(e) = entry.file_name().to_str() {
            if entry.file_type()?.is_dir() {
                write!(handle, "{blue}{e}{reset}{sep}");
            } else {
                write!(handle, "{e}{sep}");
            }
        }
    }
    writeln!(handle);

    Ok(())
}

impl Config {
    fn new() -> Self {
        Self {
            path: path::PathBuf::from("."),
            l: false,
            a: false,
            F: false,
            h: false,
            S: false,
            R: false,
            r: false,
            t: false,
            d: false,
        }
    }

    pub fn build(mut args: impl Iterator<Item = String>) -> Result<Config, &'static str> {
        args.next();
        let mut config = Config::new();

        while let Some(arg) = args.next() {
            if arg.starts_with("-") {
                for a in arg.chars().skip(1) {
                    match a {
                        'l' => config.with_l(),
                        'a' => config.with_a(),
                        'F' => config.with_F(),
                        'h' => config.with_h(),
                        'S' => config.with_S(),
                        'R' => config.with_R(),
                        'r' => config.with_r(),
                        't' => config.with_t(),
                        'd' => config.with_d(),
                        _ => {}
                    }
                }
            } else {
                config.set_path(arg);
            }
        }

        Ok(config)
    }

    fn set_path(&mut self, path: String) {
        self.path = path::PathBuf::from(path);
    }

    fn with_l(&mut self) {
        self.l = true;
    }

    fn with_a(&mut self) {
        self.a = true;
    }

    #[allow(non_snake_case)]
    fn with_F(&mut self) {
        self.F = true;
    }

    fn with_h(&mut self) {
        self.h = true;
    }

    #[allow(non_snake_case)]
    fn with_S(&mut self) {
        self.S = true;
    }

    #[allow(non_snake_case)]
    fn with_R(&mut self) {
        self.R = true;
    }

    fn with_r(&mut self) {
        self.r = true;
    }

    fn with_t(&mut self) {
        self.t = true;
    }

    fn with_d(&mut self) {
        self.d = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_arguments() {
        let args = [String::from("mini-ls")];
        let config = crate::Config::build(args.into_iter());
        assert_eq!(config, Ok(crate::Config::new()));
    }

    #[test]
    fn parse_l() {
        let args = to_string_vec(vec!["mini-ls", "-l"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.with_l();
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_la() {
        let args = to_string_vec(vec!["mini-ls", "-la"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.with_l();
        expected.with_a();
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_a_l() {
        let args = to_string_vec(vec!["mini-ls", "-a", "-l"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.with_l();
        expected.with_a();
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_al() {
        let args = to_string_vec(vec!["mini-ls", "-al"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.with_l();
        expected.with_a();
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_path_al() {
        let args = to_string_vec(vec!["mini-ls", "projects/", "-al"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.set_path(String::from("projects/"));
        expected.with_l();
        expected.with_a();
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_a_path_l() {
        let args = to_string_vec(vec!["mini-ls", "-a", "projects/", "-l"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.set_path(String::from("projects/"));
        expected.with_l();
        expected.with_a();
        assert_eq!(config, Ok(expected));
    }

    fn to_string_vec(str_slice: Vec<&str>) -> Vec<String> {
        str_slice
            .into_iter()
            .map(|elem| elem.to_string())
            .collect::<Vec<String>>()
    }
}
