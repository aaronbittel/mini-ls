use chrono::{DateTime, Utc};
use std::error::Error;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path;
use std::time::SystemTime;
use users::{get_group_by_gid, get_user_by_uid};

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

#[derive(Clone)]
pub struct FileInfo {
    permissions: String,
    link_count: u64,
    owner: String,
    group: String,
    size: u64,
    modified: String,
    name: String,
    file_type: FileType,
}

impl FileInfo {
    fn new(
        permissions: String,
        link_count: u64,
        owner: String,
        group: String,
        size: u64,
        modified: String,
        name: String,
        file_type: FileType,
    ) -> Self {
        Self {
            permissions,
            link_count,
            owner,
            group,
            size,
            modified,
            name,
            file_type,
        }
    }

    fn long(&self) -> String {
        const COLOR_BLUE: &str = "\x1b[1;38;5;12m";
        const ANSI_RESET: &str = "\x1b[0m";
        const COLOR_CYAN: &str = "\x1b[1;38;5;14m";

        let color = if self.file_type == FileType::Dir {
            COLOR_BLUE
        } else {
            ""
        };

        let name = match &self.file_type {
            FileType::Symlink(target) => match target.to_str() {
                Some(target) => format!("{COLOR_CYAN}{}{ANSI_RESET} -> {target}", self.name),
                None => "".to_string(),
            },
            _ => self.name.clone(),
        };

        format!(
            "{} {:<2} {:>5} {:>5} {:>4} {} {color}{name}{ANSI_RESET}",
            self.permissions, self.link_count, self.owner, self.group, self.size, self.modified,
        )
    }

    fn short(&self) -> String {
        const COLOR_BLUE: &str = "\x1b[1;38;5;12m";
        const ANSI_RESET: &str = "\x1b[0m";
        const COLOR_CYAN: &str = "\x1b[1;38;5;14m";

        let color = if self.file_type == FileType::Dir {
            COLOR_BLUE
        } else if let FileType::Symlink(_) = self.file_type {
            COLOR_CYAN
        } else {
            ""
        };

        format!("{color}{}{ANSI_RESET}", self.name)
    }

    fn file_name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone, PartialEq)]
enum FileType {
    File,
    Dir,
    Symlink(path::PathBuf),
}

struct LSDisplay {
    config: Config,
    entries: Vec<FileInfo>,
}

impl LSDisplay {
    fn new(config: Config, mut entries: Vec<FileInfo>) -> Self {
        entries.sort_by_key(|e| e.file_name().to_lowercase());
        if config.r {
            entries.reverse();
        }

        Self { config, entries }
    }

    fn display(&self) {
        let entries = if self.config.a {
            &self.entries
        } else {
            &self
                .entries
                .iter()
                .filter(|entry| !entry.name.starts_with('.'))
                .cloned()
                .collect::<Vec<_>>()
        };

        for entry in entries {
            if self.config.l {
                println!("{}", entry.long());
            } else {
                print!("{}  ", entry.short());
            }
        }
    }
}

// TODO: Handle "." and ".."
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut entries: Vec<FileInfo> = vec![];
    for entry in fs::read_dir(&config.path)? {
        let entry = entry?;
        let fileinfo = create_fileinfo(entry)?;
        entries.push(fileinfo);
    }

    let printer = LSDisplay::new(config, entries);
    printer.display();

    Ok(())
}

fn format_system_time(time: SystemTime) -> String {
    // Convert SystemTime to DateTime<Utc>
    let datetime: DateTime<Utc> = time.into(); // SystemTime::into converts to DateTime<Utc>

    // Format the date and time
    datetime.format("%b %d %H:%M").to_string() // Format as "Oct 18 14:37"
}

fn get_owner_and_group_names(owner_id: u32, group_id: u32) -> Result<(String, String), String> {
    // Get owner name
    let owner_name = match get_user_by_uid(owner_id) {
        Some(user) => match user.name().to_str() {
            Some(u) => u.to_string(),
            None => "Unknown user".to_string(),
        },
        None => "Unknown user".to_string(),
    };

    // Get group name
    let group_name = match get_group_by_gid(group_id) {
        Some(group) => match group.name().to_str() {
            Some(g) => g.to_string(),
            None => "Unknown group".to_string(),
        },
        None => "Unknown group".to_string(),
    };

    Ok((owner_name, group_name))
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
                for flag in arg.chars().skip(1) {
                    config.set_flag(flag);
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

    fn set_flag(&mut self, flag: char) {
        match flag {
            'l' => self.l = true,
            'a' => self.a = true,
            'F' => self.F = true,
            'h' => self.h = true,
            'S' => self.S = true,
            'R' => self.R = true,
            'r' => self.r = true,
            't' => self.t = true,
            'd' => self.d = true,
            _ => {}
        }
    }
}

// TODO: Handle Error
fn get_perm_str(perm: u32) -> String {
    match perm {
        0 => "---".to_string(),
        1 => "--x".to_string(),
        2 => "-w-".to_string(),
        4 => "r--".to_string(),
        5 => "r-x".to_string(),
        6 => "rw-".to_string(),
        7 => "rwx".to_string(),
        _ => "".to_string(),
    }
}

fn get_mode(file_type: &FileType, mode: u32) -> String {
    let owner_perm = (mode / 64) % 8;
    let group_perm = (mode / 8) % 8;
    let other_perm = mode % 8;

    let mut mode = String::with_capacity(10);
    match file_type {
        FileType::Dir => mode.push_str("d"),
        FileType::File => mode.push_str("-"),
        FileType::Symlink(_) => mode.push_str("l"),
    }

    mode.push_str(&get_perm_str(owner_perm));
    mode.push_str(&get_perm_str(group_perm));
    mode.push_str(&get_perm_str(other_perm));

    mode
}

fn create_fileinfo(entry: fs::DirEntry) -> Result<FileInfo, Box<dyn Error>> {
    let path = entry.path();
    let metadata = fs::metadata(&path)?;
    let mode = metadata.mode();
    let file = entry.file_type()?;
    let filetype: FileType;
    if file.is_dir() {
        filetype = FileType::Dir;
    } else if file.is_file() {
        filetype = FileType::File;
    } else {
        match fs::read_link(path) {
            Ok(target) => {
                filetype = FileType::Symlink(target);
            }
            Err(_) => return Err("could not get symlink target".into()),
        }
    }
    let perm = get_mode(&filetype, mode);
    let nlinks = metadata.nlink();
    let (owner, group) = get_owner_and_group_names(metadata.uid(), metadata.gid())?;
    let size = metadata.len();
    let modified_time = metadata.modified()?;
    let modified_time = format_system_time(modified_time);

    if let Some(filename) = entry.file_name().to_str() {
        return Ok(FileInfo::new(
            perm,
            nlinks,
            owner,
            group,
            size,
            modified_time,
            filename.to_string(),
            filetype,
        ));
    }

    Err("Error occured".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_mode_777() {
        let mode: u32 = 33279;
        let mode = get_mode(&FileType::File, mode);
        assert_eq!("-rwxrwxrwx".to_string(), mode)
    }

    #[test]
    fn file_mode_465() {
        let mode: u32 = 33077;
        let mode = get_mode(&FileType::File, mode);
        assert_eq!("-r--rw-r-x".to_string(), mode)
    }

    #[test]
    fn file_mode_000() {
        let mode: u32 = 32768;
        let mode = get_mode(&FileType::File, mode);
        assert_eq!("----------".to_string(), mode)
    }

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
        expected.set_flag('l');
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_la() {
        let args = to_string_vec(vec!["mini-ls", "-la"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.set_flag('l');
        expected.set_flag('a');
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_a_l() {
        let args = to_string_vec(vec!["mini-ls", "-a", "-l"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.set_flag('l');
        expected.set_flag('a');
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_al() {
        let args = to_string_vec(vec!["mini-ls", "-al"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.set_flag('l');
        expected.set_flag('a');
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_path_al() {
        let args = to_string_vec(vec!["mini-ls", "projects/", "-al"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.set_path(String::from("projects/"));
        expected.set_flag('l');
        expected.set_flag('a');
        assert_eq!(config, Ok(expected));
    }

    #[test]
    fn parse_a_path_l() {
        let args = to_string_vec(vec!["mini-ls", "-a", "projects/", "-l"]);
        let config = crate::Config::build(args.into_iter());
        let mut expected = Config::new();
        expected.set_path(String::from("projects/"));
        expected.set_flag('l');
        expected.set_flag('a');
        assert_eq!(config, Ok(expected));
    }

    fn to_string_vec(str_slice: Vec<&str>) -> Vec<String> {
        str_slice
            .into_iter()
            .map(|elem| elem.to_string())
            .collect::<Vec<String>>()
    }
}
