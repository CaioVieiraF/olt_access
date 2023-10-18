use crate::Result;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub struct Config {
    command_count: u8,
    // pub commands: Vec<String>,
    pub commands: Box<[String]>,
}

impl Config {
    pub fn from(path: &'static str) -> Result<Config> {
        let file = File::open(path)?;
        let content = BufReader::new(file);
        // file.read_to_string(&mut content)?;
        let commands: Box<[String]> = content.lines().map(|a| a.unwrap()).collect();
        let command_count = commands.len() as u8;

        Ok(Config {
            command_count,
            commands,
        })
    }

    pub fn show_config(&self) {
        for line in self.commands.iter() {
            println!("{line}");
        }
    }
}
