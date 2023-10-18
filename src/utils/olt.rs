use crate::prelude::Result;

use super::command::{CmdArg0, CommandBuilder, ConfT};
use std::{
    net::{Ipv4Addr, TcpStream},
    sync::Arc,
};

use ssh2::Session;

#[derive(Debug)]
pub struct Interface {
    position: (u8, u8, u8),
    prefix: Arc<str>,
}

impl Default for Interface {
    fn default() -> Self {
        Interface {
            position: Default::default(),
            prefix: Arc::from(""),
        }
    }
}

impl Interface {
    pub fn position(&self) -> (u8, u8, u8) {
        self.position
    }

    pub fn prefix(&self) -> Arc<str> {
        self.prefix.clone()
    }
}

pub struct Olt {
    session: Session,
    model: OltModel,
}

#[derive(Default)]
pub struct OltBuilder<S, M> {
    ip: S,
    model: M,
    port: Option<u16>,
}

impl Interface {
    pub fn new(position: (u8, u8, u8), prefix: impl Into<String>) -> Interface {
        Interface {
            position,
            prefix: prefix.into().into(),
        }
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

impl Olt {
    pub fn builder() -> OltBuilder<NoSession, NoModel> {
        OltBuilder::new()
    }

    pub fn configure(&self) -> CommandBuilder<ConfT, CmdArg0> {
        CommandBuilder::new()
    }
}

#[derive(Default)]
pub struct NoModel;
#[derive(Default)]
pub struct NoSession;

impl OltBuilder<NoSession, NoModel> {
    pub fn new() -> Self {
        OltBuilder::default()
    }
}

impl<S, M> OltBuilder<S, M> {
    pub fn model(self, olt_model: OltModel) -> OltBuilder<S, OltModel> {
        OltBuilder {
            model: olt_model,
            ip: self.ip,
            port: self.port,
        }
    }

    pub fn ip(self, ip: Ipv4Addr) -> OltBuilder<Ipv4Addr, M> {
        OltBuilder {
            ip,
            model: self.model,
            port: self.port,
        }
    }

    pub fn port(self, port: u16) -> Self {
        OltBuilder {
            port: Some(port),
            ..self
        }
    }
}

impl OltBuilder<Ipv4Addr, OltModel> {
    pub fn build(self) -> Result<Olt> {
        let mut session = Session::new()?;

        let port = self.port.unwrap_or(22);
        let addr = self.ip.to_string() + ":" + port.to_string().as_str();
        let stream = TcpStream::connect(addr)?;

        session.set_tcp_stream(stream);
        session.handshake()?;

        Ok(Olt {
            model: self.model,
            session,
        })
    }
}
