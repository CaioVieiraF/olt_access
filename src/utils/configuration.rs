use crate::Command;
use crate::Result;
use regex::Regex;
use serde::Deserialize;
use std::fmt::Display;
use std::ops::AddAssign;
use std::sync::Arc;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    str::FromStr,
};

use super::olt::InterfaceLevel;
use super::{
    olt::Interface,
    onu::{Onu, OnuService, Vlan},
};

#[derive(Clone, Debug)]
pub struct NestedCommand {
    pub command: Command,
    pub nested: Option<Vec<NestedCommand>>,
}

impl Iterator for NestedCommand {
    type Item = NestedCommand;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(vector) = &self.nested {
            vector.iter().next().cloned()
        } else {
            None
        }
    }
}

impl NestedCommand {
    pub fn nest(&mut self, child: NestedCommand) {
        if let Some(n) = self.nested.as_mut() {
            n.push(child);
        } else {
            self.nested = Some(vec![child]);
        }
    }

    pub fn as_str(&self) -> &str {
        self.command.as_str()
    }

    pub fn raw(&self) -> String {
        let mut result = format!("{}\n", self.as_str());

        if let Some(n) = &self.nested {
            for cmd in n {
                result += "  ";
                result += &cmd.raw();
            }

            result += "$\n";
        }

        result
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Debug)]
pub struct ConfigField(Arc<str>);

impl ConfigField {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl Display for ConfigField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for ConfigField {
    fn default() -> Self {
        ConfigField("raw".into())
    }
}

impl From<&str> for ConfigField {
    fn from(value: &str) -> Self {
        ConfigField(Arc::from(value))
    }
}

#[derive(Debug, Default)]
pub struct Config(pub HashMap<ConfigField, Vec<NestedCommand>>);

impl AddAssign for Config {
    fn add_assign(&mut self, rhs: Self) {
        for (key, value) in rhs.0 {
            if let Some(i) = self.0.get_mut(&key) {
                i.extend(value);
            } else {
                self.0.insert(key, value);
            }
        }
    }
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

impl From<Command> for NestedCommand {
    fn from(value: Command) -> Self {
        NestedCommand {
            command: value,
            nested: None,
        }
    }
}

impl From<File> for Config {
    fn from(value: File) -> Self {
        let content = BufReader::new(value);
        let mut result: HashMap<ConfigField, Vec<NestedCommand>> = HashMap::new();
        let field_pattern = Regex::new(r"!<(?P<name>.*)>").unwrap();
        let nest_pattern = Regex::new(r"^ {2,}(?P<command>.*)").unwrap();

        let mut buffer = None;
        for command in content.lines().flatten() {
            info!("{buffer:?}");
            info!("{command}");
            if command.is_empty() {
                continue;
            }
            if let Some(s) = field_pattern.captures(&command) {
                if s["name"].starts_with('/') {
                    buffer = None;
                } else {
                    let field = ConfigField::from(&s["name"]);
                    buffer = Some(field.clone());
                    result.insert(field, Vec::new());
                }
                continue;
            }

            if command.contains('$') {
                continue;
            }

            let current_command = result.get_mut(&buffer.clone().unwrap()).unwrap();
            let new_command = NestedCommand::from(Command::from(command.as_str().trim()));

            if nest_pattern.captures(&command).is_some() {
                let level: Vec<&str> = command.split("  ").collect();
                let level = level.len() - 2;
                let mut current_command = current_command.last_mut().unwrap();
                for _ in 0..level {
                    if let Some(ref mut n) = current_command.nested {
                        current_command = n.last_mut().unwrap();
                    }
                }
                current_command.nest(new_command);
            } else {
                current_command.push(new_command);
            }
        }

        Config(result)
    }
}

impl From<Vec<Command>> for Config {
    fn from(value: Vec<Command>) -> Self {
        let commands: Vec<NestedCommand> = value
            .iter()
            .map(|c| NestedCommand::from(c.clone()))
            .collect();
        let mut result = HashMap::new();
        result.insert(ConfigField::default(), commands);
        Config(result)
    }
}

impl Config {
    pub fn to_file(&self, mut file: File) -> Result<File> {
        let mut script = String::new();
        for (key, i) in self.0.iter() {
            let field = (format!("!<{}>", key.0), format!("!</{}>", key.0));
            script.push_str(&field.0);
            script.push('\n');
            for command in i {
                script.push_str(&command.raw());
            }
            script.push_str(&field.1);
            script.push('\n');
        }

        writeln!(file, "{}", script)?;
        Ok(file)
    }

    pub fn extract_onu(&self) -> Vec<Onu> {
        let mut onu_instances: Vec<Onu> = Vec::new();
        let creation_pattern = Regex::new(
            r"^onu (?P<id>[1-9]|[1-9][0-9]|1[0-2][0-9]) type (?P<type>.*) sn (?P<sn>.*)$",
        )
        .unwrap();

        let field = self.0.get(&ConfigField::from("xpon")).unwrap();
        //.unwrap_or(self.0.get(&ConfigField::default()).unwrap());

        for c in field {
            if let Ok(i) = Interface::from_str(c.as_str()) {
                if i.level == InterfaceLevel::GponOlt {
                    if let Some(inter) = &c.nested {
                        for onu in inter {
                            if let Some(o) = creation_pattern.captures(onu.as_str()) {
                                let id = o["id"].parse::<u8>().unwrap();
                                let interface = i.with_id(id);
                                let new_onu =
                                    Onu::new(interface, &o["type"], &o["sn"], Vec::default());
                                onu_instances.push(new_onu);
                            }
                        }
                    }
                } else if i.level == InterfaceLevel::PonOnuMng {
                    info!("{}", c.command);
                    let mut services = Vec::new();
                    let mut current_onu = onu_instances
                        .iter_mut()
                        .filter(|o| o.interface() == &i)
                        .collect::<Vec<&mut Onu>>();
                    let current_onu = current_onu.first_mut().unwrap();
                    if let Some(value) = &c.nested {
                        for infos in value {
                            let vlan = Vlan::try_from(&infos.command);
                            if let Ok(v) = vlan {
                                services.push(OnuService::new(v));
                            }
                        }
                    }

                    current_onu.set_service(services.into());
                }
            }
        }

        onu_instances
    }
}
