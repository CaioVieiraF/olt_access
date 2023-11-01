use crate::{prelude::Error, Result};
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    rc::Rc,
};

use crate::Command;
use serde::Deserialize;

use super::{
    olt::Interface,
    onu::{Onu, OnuService, Vlan},
};

#[derive(Deserialize)]
pub struct GeneralParam {
    pub vlan: u16,
    pub interface: Interface,
}

pub struct Config {
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

        Config { commands }
    }
}

impl From<Vec<Command>> for Config {
    fn from(value: Vec<Command>) -> Self {
        let commands = value.iter().map(|c| c.raw()).collect();
        Config { commands }
    }
}

struct OnuTypeSn {
    r#type: String,
    sn: String,
    service: Vlan,
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

    pub fn extract_onu(&self, vlan_id: u16) -> Vec<Onu> {
        let mut onu_list: HashMap<Interface, HashMap<u8, OnuTypeSn>> = HashMap::new();
        let mut onu_instances: Vec<Onu> = Vec::new();

        let mut pon_buffer: Option<(Interface, u8)> = None;
        let mut interface_buffer: Option<Interface> = None;
        for c in self.commands.iter() {
            if !c.contains(' ') {
                continue;
            }

            let cmd_args: Vec<String> = c.split(' ').map(String::from).collect();
            if let Ok(i) = Interface::try_from(c.to_string()) {
                if c.contains("olt") {
                    interface_buffer = Some(i);
                } else if c.contains("pon-onu-mng") {
                    println!("{c}");
                    let id: Vec<_> = cmd_args.last().unwrap().split(':').collect();
                    let id = id.last().unwrap().parse().unwrap();

                    pon_buffer = Some((i, id));
                }
            } else if cmd_args.len() == 6
                && cmd_args.first().unwrap() == "onu"
                && cmd_args.get(2).unwrap() == "type"
                && cmd_args.get(4).unwrap() == "sn"
            {
                let id: u8 = cmd_args.get(1).unwrap().parse().unwrap();

                let onu = OnuTypeSn {
                    r#type: cmd_args.get(3).unwrap().to_string(),
                    sn: cmd_args.get(5).unwrap().to_string(),
                    service: Vlan::new(vlan_id),
                };

                if let Some(ref i) = interface_buffer {
                    match onu_list.get_mut(i) {
                        Some(h) => {
                            h.insert(id, onu);
                        }
                        None => {
                            let mut new_map = HashMap::new();
                            new_map.insert(id, onu);
                            onu_list.insert(i.clone(), new_map);
                        }
                    }
                }
            } else if cmd_args.len() >= 10 && cmd_args[0] == "wan-ip" {
                if let Some(ref b) = pon_buffer {
                    let id = b.1;
                    if let Some(o) = onu_list.get_mut(&b.0) {
                        if let Some(k) = o.get_mut(&id) {
                            if cmd_args[3] == "pppoe" {
                                k.service.pppoe(cmd_args[5].clone(), cmd_args[7].clone());
                            } else if cmd_args[3] == "dhcp" {
                                k.service.dhcp();
                            }
                        }
                    }
                }
            }
        }

        for (key, value) in onu_list {
            for (id, onu_info) in value {
                let services = vec![OnuService::new(onu_info.service)];
                let new_onu = Onu::new(
                    id,
                    key.interface(),
                    &onu_info.r#type,
                    &onu_info.sn,
                    services,
                );
                onu_instances.push(new_onu);
            }
        }

        onu_instances
    }
}
