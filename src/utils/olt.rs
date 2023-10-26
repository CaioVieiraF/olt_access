use crate::prelude::Result;

use super::command::{CmdArg0, Command, CommandBuilder, ConfT};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    net::{Ipv4Addr, TcpStream},
    sync::Arc,
};

use ssh2::{Channel, Session};

#[derive(Debug, Deserialize, Serialize)]
pub struct Interface {
    pub shelf: u8,
    pub slot: u8,
    pub port: u8,
}

pub struct Olt {
    session: Session,
    model: OltModel,
    queue: Vec<Command>,
}

#[derive(Default)]
pub struct OltBuilder<S, M> {
    ip: S,
    model: M,
    port: Option<u16>,
}

impl Interface {
    pub fn new(shelf: u8, slot: u8, port: u8) -> Interface {
        Interface { shelf, slot, port }
    }

    pub fn interface(&self) -> (u8, u8, u8) {
        (self.shelf, self.slot, self.port)
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

    pub fn enqueue(&mut self, command: Command) {
        self.queue.push(command);
    }

    pub fn is_authenticated(&self) -> bool {
        self.session.authenticated()
    }

    pub fn run(&self) -> Result<Vec<i32>> {
        //let res = Box::new([Box::from(".")]);
        let mut res = Vec::new();
        for command in self.queue.iter() {
            print!("{}: ", command.raw());
            std::io::stdout().flush()?;

            let mut channel: Channel;
            match self.session.channel_session() {
                Ok(c) => channel = c,
                Err(e) => {
                    println!("Erro {} ao abrir a sessão: {}", e.code(), e.message());
                    continue;
                }
            }

            let mut s = String::new();
            match channel.exec(command.raw().trim()) {
                Ok(_) => {
                    channel.read_to_string(&mut s).unwrap();
                }
                Err(e) => match e.code() {
                    ssh2::ErrorCode::Session(-22) => {
                        println!("A requisição do canal foi negada.");
                    }
                    _ => panic!("{e}"),
                },
            }

            channel.close()?;
            channel.wait_close()?;

            if let Ok(c) = channel.exit_status() {
                res.push(c);
            }
        }
        Ok(res)
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
        session.userauth_password("caiof", "Lab@2023")?;

        Ok(Olt {
            model: self.model,
            session,
            queue: Vec::new(),
        })
    }
}
