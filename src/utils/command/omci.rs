use std::{fmt::format, marker::PhantomData};

use crate::utils::command::{CmdArg0, CmdArg1, CmdArg2, CmdArg3, CmdArg4, Command, CommandBuilder};

use super::Omci;

#[derive(Debug)]
pub enum WanMode {
    PPPoE,
    DHCP,
}

#[derive(Debug)]
pub enum IngressType {
    Iphost(u8),
    Wan,
    Lan,
}

#[derive(Debug)]
pub enum Protocol {
    Web,
    Telnet,
    Ssh,
    Ftp,
    Snmp,
    Tr069,
    Https,
}

impl CommandBuilder<Omci, CmdArg0> {
    pub fn service(self, number: u8) -> CommandBuilder<Omci, CmdArg1> {
        CommandBuilder {
            command: format!("service {number}").into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn wan_ip(self) -> CommandBuilder<Omci, CmdArg1> {
        CommandBuilder {
            command: "wan-ip ipv4".into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn security_mgmt(self, number: u8) -> CommandBuilder<Omci, CmdArg1> {
        CommandBuilder {
            command: format!("security-mgmt {number}").into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    //TODO missing fields
}

impl CommandBuilder<Omci, CmdArg1> {
    //service gemport
    pub fn gemport(self, number: u8) -> CommandBuilder<Omci, CmdArg2> {
        CommandBuilder {
            command: format!("{} gemport {number}", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    // wan-ip
    pub fn mode(self) -> CommandBuilder<Omci, CmdArg2> {
        CommandBuilder {
            command: format!("{} mode", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    //security-mgmt
    pub fn state(self, enable: bool) -> CommandBuilder<Omci, CmdArg2> {
        let state = if enable { "enable" } else { "disable" };

        CommandBuilder {
            command: format!("{} state {state}", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<Omci, CmdArg2> {
    //service gemport
    pub fn run(self) -> Command {
        self.command.into()
    }

    pub fn vlan(self, vlan: u16) -> Command {
        format!("{} vlan {vlan}", self.command).into()
    }

    //wan-ip
    pub fn pppoe(
        self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> CommandBuilder<Omci, CmdArg3> {
        CommandBuilder {
            command: format!(
                "{} pppoe username {} password {}",
                self.command,
                username.into(),
                password.into()
            )
            .into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    //security-mgmt
    pub fn mode(self, value: bool) -> CommandBuilder<Omci, CmdArg3> {
        let mode = if value { "forward" } else { "discard" };

        CommandBuilder {
            command: format!("{} mode {mode}", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    //TODO missing fields
}

impl CommandBuilder<Omci, CmdArg3> {
    //wan-ip
    pub fn vlan_profile(self, vlan: u16) -> CommandBuilder<Omci, CmdArg4> {
        CommandBuilder {
            command: format!("{} vlan-profile {vlan}", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    //security-mgmt
    pub fn ingress_type(self, r#type: IngressType) -> CommandBuilder<Omci, CmdArg4> {
        let ingrs = match r#type {
            IngressType::Iphost(x) => format!("iphost {x}"),
            _ => format!("{:?}", r#type),
        };
        CommandBuilder {
            command: format!("{} ingress-type {ingrs}", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<Omci, CmdArg4> {
    //wan-ip
    pub fn host(self, number: u8) -> Command {
        format!("{} host {number}", self.command).into()
    }

    //security-mgmt
    pub fn protocol(self, prot: Protocol) -> Command {
        format!("{} protocol {:?}", self.command, prot).into()
    }
}
