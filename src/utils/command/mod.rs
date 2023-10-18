#![allow(unused)]

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

pub struct CommandBuilder<T, U> {
    command: Box<str>,
    command_level: PhantomData<T>,
    arg: PhantomData<U>,
}

impl CommandBuilder<ConfT, CmdArg0> {
    pub fn new() -> Self {
        CommandBuilder {
            command: Box::from(""),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn interface(self) -> CommandBuilder<Interface, CmdArg0> {
        CommandBuilder {
            command: Box::from("interface"),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}
