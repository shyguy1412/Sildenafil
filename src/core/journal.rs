use std::{
    fs::{File, read_dir},
    io::{BufRead, BufReader, Lines},
    path::PathBuf,
    thread,
    time::Duration,
};

use crate::{Result, events::Event, journal};
use bondage::*;

pub struct Journal {
    known_journals: Vec<PathBuf>,
    current_journal: Option<Lines<BufReader<File>>>,
    is_final_journal: bool,
}

impl Journal {
    pub fn new() -> Self {
        Self {
            known_journals: vec![],
            current_journal: None,
            is_final_journal: false,
        }
    }

    fn open_next_journal(&mut self) -> Option<Lines<BufReader<File>>> {
        let events_location = get_linux_events_location().ok()?;
        let current_journal = self.get_next_journal(&events_location)?;

        let is_known = self
            .known_journals
            .iter()
            .any(|known_buff| *known_buff == current_journal);

        if is_known {
            console_log("Waiting for new journal");
            std::thread::park();
            return None;
        };

        console_log(current_journal.to_string_lossy().to_string());

        let file = File::open(&current_journal)
            .ok()
            .inspect(|f| {
                // console_log(format!("{:?}", f.unlock()));
            })
            .map(BufReader::new)
            .map(BufReader::lines);

        self.known_journals.push(current_journal);

        file
    }

    fn get_next_journal(&mut self, path: &String) -> Option<PathBuf> {
        fn non_numeric(char: char) -> bool {
            !char.is_numeric()
        }

        let Ok(files) = read_dir(path) else {
            return None;
        };

        let mut files: Vec<_> = files
            .filter_map(|file| file.ok())
            .filter_map(|file| file.file_name().into_string().ok())
            .map(|file| (file.replace(non_numeric, ""), file))
            .filter_map(|(date, file)| u64::from_str_radix(&date, 10).ok().map(|date| (file, date)))
            .collect();

        // files.sort_by(|a, b| b.1.cmp(&a.1));
        files.sort_by(|a, b| a.1.cmp(&b.1));

        if files.len().checked_sub(1).unwrap_or(0) == self.known_journals.len() {
            self.is_final_journal = true;
        }

        let current_journal = files
            .get(self.known_journals.len())
            .map(|(f, _)| std::path::Path::new(path).join(f));

        current_journal
    }
}

fn get_windows_events_location() -> Result<String> {
    //!This doesnt check for a steam lib on a different drive

    let user_profile = std::env::var("USERPROFILE")?;

    Ok(format!(
        "{}\\Saved Games\\Frontier Developments\\Elite Dangerous",
        user_profile
    ))
}

fn get_linux_events_location() -> Result<String> {
    //!This doesnt check for a steam lib on a different drive

    let user_home = std::env::var("HOME")?;

    Ok(format!(
        "{}/.local/share/Steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous",
        user_home
    ))
}

impl Iterator for Journal {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        if let None = self.current_journal {
            self.current_journal = self.open_next_journal();
        }

        let Some(ref mut journal) = self.current_journal else {
            return None;
        };

        let Some(line) = journal.next() else {
            if self.is_final_journal {
                match self.open_next_journal() {
                    None => (),
                    j => self.current_journal = j,
                }
                return None;
            };

            self.current_journal = None;
            return self.next();
        };

        let Ok(line) = line else {
            return None;
        };

        match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => self.next(),
        }
    }
}
