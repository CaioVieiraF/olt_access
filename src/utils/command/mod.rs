pub mod interface;

use std::{marker::PhantomData, rc::Rc};

use super::olt::Interface;

pub struct ConfT;

pub struct CmdArg0;
pub struct CmdArg1;
pub struct CmdArg2;
pub struct CmdArg3;
pub struct CmdArg4;

pub struct Command {
    raw: Rc<str>,
}

impl From<&str> for Command {
    fn from(value: &str) -> Command {
        Command {
            raw: Rc::from(value),
        }
    }
}

impl From<Rc<str>> for Command {
    fn from(value: Rc<str>) -> Command {
        Command { raw: value }
    }
}

impl From<String> for Command {
    fn from(value: String) -> Command {
        Command {
            raw: Rc::from(value),
        }
    }
}

impl Command {
    pub fn raw(&self) -> Rc<str> {
        self.raw.clone()
    }

    pub fn builder() -> CommandBuilder<ConfT, CmdArg0> {
        CommandBuilder::new()
    }
}

pub struct CommandBuilder<T, U> {
    pub command: Rc<str>,
    command_level: PhantomData<T>,
    arg: PhantomData<U>,
}

impl CommandBuilder<ConfT, CmdArg0> {
    pub fn new() -> Self {
        CommandBuilder {
            command: Rc::from("configure terminal"),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn interface(self) -> CommandBuilder<Interface, CmdArg0> {
        CommandBuilder {
            command: Rc::from("interface"),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}
