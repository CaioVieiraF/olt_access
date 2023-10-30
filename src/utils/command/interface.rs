use crate::utils::command::{CmdArg0, CmdArg1, CmdArg2, CmdArg3, CmdArg4, Command, CommandBuilder};

use crate::utils::olt::Interface;

use core::marker::PhantomData;

#[derive(Clone, Debug)]
pub struct InterfaceOlt;
#[derive(Clone, Debug)]
pub struct InterfaceOnu;
#[derive(Clone, Debug)]
pub struct InterfaceVport;

impl CommandBuilder<Interface, CmdArg0> {
    pub fn gpon_olt(self, position: (u8, u8, u8)) -> CommandBuilder<InterfaceOlt, CmdArg0> {
        let interface = format!(
            "{} gpon_olt-{}/{}/{}",
            self.command, position.0, position.1, position.2
        );

        CommandBuilder {
            command: interface.into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn gpon_onu(self, position: (u8, u8, u8), id: u8) -> CommandBuilder<InterfaceOnu, CmdArg0> {
        let interface = format!(
            "{} gpon_onu-{}/{}/{}:{id}",
            self.command, position.0, position.1, position.2
        );

        CommandBuilder {
            command: interface.into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn vport(
        self,
        position: (u8, u8, u8),
        id: u8,
        service: u8,
    ) -> CommandBuilder<InterfaceVport, CmdArg0> {
        CommandBuilder {
            command: format!(
                "{} vport-{}/{}/{}.{id}:{service}",
                self.command, position.0, position.1, position.2
            )
            .into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceOlt, CmdArg0> {
    pub fn onu(self, id: u8) -> CommandBuilder<InterfaceOlt, CmdArg1> {
        CommandBuilder {
            command: format!("onu {id}").into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceOlt, CmdArg1> {
    pub fn r#type(self, onu_type: impl Into<String>) -> CommandBuilder<InterfaceOlt, CmdArg2> {
        let command = format!("{} type {}", self.command, onu_type.into());
        CommandBuilder {
            command: command.into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceOlt, CmdArg2> {
    pub fn sn(self, sn: impl Into<String>) -> CommandBuilder<InterfaceOlt, CmdArg3> {
        CommandBuilder {
            command: format!("{} sn {}", self.command, sn.into()).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    //TODO: missing fields
}

impl CommandBuilder<InterfaceOlt, CmdArg3> {
    pub fn run(self) -> Command {
        Command { raw: self.command }
    }

    pub fn vport_mode(self) -> CommandBuilder<InterfaceOlt, CmdArg4> {
        CommandBuilder {
            command: format!("{} vport_mode", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceOlt, CmdArg4> {
    pub fn gemport(self) -> Command {
        let command = format!("{} gemport", self.command);
        Command {
            raw: command.into(),
        }
    }

    pub fn manual(self) -> Command {
        let command = format!("{} manual", self.command);
        Command {
            raw: command.into(),
        }
    }
}

impl CommandBuilder<InterfaceOnu, CmdArg0> {
    pub fn tcont(self, number: u8) -> CommandBuilder<InterfaceOnu, CmdArg1> {
        CommandBuilder {
            command: format!("tcont {number} ").into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn gemport(self, number: u8) -> CommandBuilder<InterfaceOnu, CmdArg1> {
        CommandBuilder {
            command: format!("gemport {number}").into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceOnu, CmdArg1> {
    pub fn profile(self, prof: impl Into<String>) -> Command {
        format!("{} profile {}", self.command, prof.into()).into()
    }

    pub fn tcont(self, number: u8) -> CommandBuilder<InterfaceOnu, CmdArg2> {
        CommandBuilder {
            command: format!("{} tcont {number}", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceOnu, CmdArg2> {
    pub fn run(self) -> Command {
        self.command.into()
    }
}

impl CommandBuilder<InterfaceVport, CmdArg0> {
    pub fn service_port(self, number: u8) -> CommandBuilder<InterfaceVport, CmdArg1> {
        CommandBuilder {
            command: format!("service-port {number} ").into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceVport, CmdArg1> {
    pub fn user_vlan(self, vlan: u16) -> CommandBuilder<InterfaceVport, CmdArg2> {
        CommandBuilder {
            command: format!("{} user-vlan {vlan}", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceVport, CmdArg2> {
    pub fn vlan(self, vlan: u16) -> CommandBuilder<InterfaceVport, CmdArg3> {
        CommandBuilder {
            command: format!("{} vlan {vlan}", self.command).into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}

impl CommandBuilder<InterfaceVport, CmdArg3> {
    pub fn run(self) -> Command {
        self.command.into()
    }

    //TODO missing fields
}
