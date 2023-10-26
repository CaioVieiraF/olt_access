use crate::{prelude::Error, Result};
use std::{
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    rc::Rc,
};

use crate::Command;
use serde::Deserialize;

use super::olt::Interface;

#[derive(Deserialize)]
pub struct GeneralParam {
    pub vlan: u16,
    pub interface: Interface,
}

pub struct Config {
    command_count: u32,
    // pub commands: Vec<String>,
    pub commands: Box<[Rc<str>]>,
}

impl TryFrom<File> for GeneralParam {
    type Error = Error;
    fn try_from(value: File) -> Result<Self> {
        let mut file = String::new();
        let mut reader = BufReader::new(value);
        reader.read_to_string(&mut file)?;

        let toml_config: GeneralParam = toml::from_str(&file)?;

        Ok(toml_config)
    }
}

impl From<File> for Config {
    fn from(value: File) -> Self {
        let content = BufReader::new(value);
        // file.read_to_string(&mut content)?;
        let commands: Box<[Rc<str>]> = content.lines().map(|a| a.unwrap()).map(Rc::from).collect();
        let command_count = commands.len() as u32;

        Config {
            command_count,
            commands,
        }
    }
}

impl From<Vec<Command>> for Config {
    fn from(value: Vec<Command>) -> Self {
        let commands = value.iter().map(|c| c.raw()).collect();
        Config {
            command_count: value.len() as u32,
            commands,
        }
    }
}

impl Config {
    pub fn to_file(&self, mut file: File) -> Result<File> {
        for i in self.commands.iter() {
            writeln!(file, "{}", i)?;
        }
        Ok(file)
    }

    pub fn show_config(&self) {
        for line in self.commands.iter() {
            println!("{line}");
        }
    }
}
