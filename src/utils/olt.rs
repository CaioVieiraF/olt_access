use crate::prelude::{Error, Result};

use super::{command::Command, configuration::Config, onu::Onu};
use clap::Parser;
use regex::Regex;
use std::{rc::Rc, str::FromStr, sync::Arc};

#[derive(Parser, Debug, Clone, Eq, Hash, PartialEq, Default)]
pub struct Interface {
    pub level: InterfaceLevel,
    pub slot: u8,
    pub port: u8,
    pub id: Option<u8>,
}

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum InterfaceLevel {
    GponOlt,
    GponOnu,
    PonOnuMng,
    Other(Arc<str>),
}

pub struct Olt {
    model: OltModel,
    interfaces: Rc<[Interface]>,
    onu: Vec<Onu>,
    configuration: Config,
}

impl From<OltModel> for Olt {
    fn from(value: OltModel) -> Self {
        Olt {
            model: value,
            interfaces: Rc::new([Interface::default()]),
            onu: Default::default(),
            configuration: Default::default(),
        }
    }
}

impl Default for InterfaceLevel {
    fn default() -> Self {
        InterfaceLevel::Other("generic".into())
    }
}

impl Interface {
    pub fn with_id(&self, id: u8) -> Interface {
        Interface {
            id: Some(id),
            ..self.clone()
        }
    }
}

impl From<&str> for InterfaceLevel {
    fn from(value: &str) -> Self {
        match value {
            "gpon-olt" | "gpon_olt" => InterfaceLevel::GponOlt,
            "gpon-onu" | "gpon_onu" => InterfaceLevel::GponOnu,
            "pon-onu-mng" => InterfaceLevel::PonOnuMng,
            _ => InterfaceLevel::Other(value.into()),
        }
    }
}

impl FromStr for InterfaceLevel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let result = InterfaceLevel::from(s);

        Ok(result)
    }
}

impl FromStr for Interface {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        let pattern = Regex::new(
            r"(.* )?(?P<level>.*)[_\-]1\/(?P<card>[0-9]|1[0-9])\/(?P<port>[1-9]|1[0-6])(:(?P<id>[1-9]|[1-9][0-9]|1[0-2][09]))?$",
        ).unwrap();

        let interface = pattern
            .captures(value)
            .ok_or(Error::Generic("Parse interface".to_string()))?;

        let id = interface
            .name("id")
            .map(|id| id.as_str().parse::<u8>().unwrap());
        let slot = interface["card"].parse().unwrap();
        let port = interface["port"].parse().unwrap();
        Ok(Interface {
            level: InterfaceLevel::from(&interface["level"]),
            slot,
            port,
            id,
        })
    }
}

impl TryFrom<Command> for Interface {
    type Error = Error;

    fn try_from(value: Command) -> Result<Self> {
        let cmd = value.raw();
        Interface::from_str(&cmd)
    }
}

pub enum OltModel {
    Titan(Titan),
    C3xx(C3xx),
}

pub enum C3xx {
    C320,
    C350,
    C300,
}

pub enum Titan {
    C610,
    C620,
    C650,
    C600,
}
