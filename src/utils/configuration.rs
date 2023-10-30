use crate::{prelude::Error, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    ops::RangeToInclusive,
    rc::Rc,
};

use crate::Command;
use serde::Deserialize;

use super::{olt::Interface, onu::Onu};

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

#[derive(Deserialize)]
pub struct ConfigInfo {
    pub sn: String,
    pub pppoe_user: String,
    pub pppoe_password: String,
    pub model: String,
}

impl ConfigInfo {
    pub fn from_file(file: File) -> Result<Vec<ConfigInfo>> {
        let mut infos: Vec<ConfigInfo> = Vec::new();
        let mut reader = csv::Reader::from_reader(file);
        for result in reader.deserialize() {
            let record: ConfigInfo = result?;
            infos.push(record);
        }

        Ok(infos)
    }
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
        let commands: Box<[Rc<str>]> = content
            .lines()
            .map(|a| a.unwrap())
            .map(|s| s.trim().to_string())
            .filter(|s| s != "!")
            .map(Rc::from)
            .collect();
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

    pub fn get_onu_script(&self) -> Vec<Command> {
        let mut interfaces: Vec<Interface> = Vec::new();
        let mut onu_prot: Vec<(u8, String, String)> = Vec::new();
        let mut script: Vec<Command> = Vec::new();
        let vlan = 1000;
        let mut pppoe_users: Vec<(String, String)> = Vec::new();

        for c in self.commands.iter() {
            if !c.contains(' ') {
                continue;
            }

            let cmd_args: Vec<String> = c.split(' ').map(String::from).collect();
            if let Ok(i) = Interface::try_from(c.to_string()) {
                if c.contains("olt") {
                    interfaces.push(i);
                }
            } else if cmd_args.len() == 6
                && cmd_args.first().unwrap() == "onu"
                && cmd_args.get(2).unwrap() == "type"
                && cmd_args.get(4).unwrap() == "sn"
            {
                let id: u8 = cmd_args.get(1).unwrap().parse().unwrap();
                let r#type = cmd_args.get(3).unwrap();
                let sn = cmd_args.get(5).unwrap();

                onu_prot.push((id, sn.to_string(), r#type.to_string()));
            } else if cmd_args.len() >= 10 && cmd_args[0] == "wan-ip" {
                let username = &cmd_args[5];
                let password = &cmd_args[7];
                pppoe_users.push((username.to_string(), password.to_string()));
            }
        }

        interfaces.reverse();
        for inter in interfaces {
            while let Some(i) = onu_prot.pop() {
                let onu = Onu::new(i.0, inter.interface(), vlan, &i.2, &i.1);
                let config = onu.configure_script(None);
                script.extend(config);

                if i.0 == 1 {
                    break;
                }
            }
        }

        script
    }
}
