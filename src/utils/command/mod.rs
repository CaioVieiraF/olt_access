pub mod interface;
pub mod omci;

use std::{fs::File, marker::PhantomData, rc::Rc};

use crate::prelude::Result;

use super::{
    configuration::{ConfigInfo, GeneralParam},
    olt::Interface,
    onu::{Onu, OnuService, Vlan},
};

#[derive(Clone)]
pub struct ConfT;
#[derive(Clone)]
pub struct Omci;

#[derive(Clone)]
pub struct CmdArg0;
#[derive(Clone)]
pub struct CmdArg1;
#[derive(Clone)]
pub struct CmdArg2;
#[derive(Clone)]
pub struct CmdArg3;
#[derive(Clone)]
pub struct CmdArg4;

// Estrutura que armazena um comando
pub struct Command {
    raw: Rc<str>,
}

// Cria um comando a partir de um texto literal
impl From<&str> for Command {
    fn from(value: &str) -> Command {
        Command {
            raw: Rc::from(value),
        }
    }
}

// Cria um comando a partir de um ponteiro de
// um texto literal
impl From<Rc<str>> for Command {
    fn from(value: Rc<str>) -> Command {
        Command { raw: value }
    }
}

// Cria um comando a partir de um texto
impl From<String> for Command {
    fn from(value: String) -> Command {
        Command {
            raw: Rc::from(value),
        }
    }
}

impl Command {
    // Devolve o comando como um ponteiro de texto
    pub fn raw(&self) -> Rc<str> {
        self.raw.clone()
    }

    // Comando "exit"
    pub fn exit() -> Self {
        "exit".into()
    }

    // Comando "write"
    pub fn write() -> Self {
        "do write".into()
    }

    // Comando "end"
    pub fn end() -> Self {
        "end".into()
    }

    // Abstração que cria comandos conhecidos
    pub fn builder() -> CommandBuilder<ConfT, CmdArg0> {
        CommandBuilder::new()
    }

    // Abstração que gera um script de configuração de ONU
    // baseado nas informações de um arquivo
    pub fn onu_script_from_file(general_info: File, equipment_info: File) -> Result<Vec<Command>> {
        // Carrega os arquivos em estruturas conhecidas, caso esteja
        // no formato certo
        let params = GeneralParam::try_from(general_info)?;
        let configurations = ConfigInfo::from_file(equipment_info)?;
        let mut configure_script = Vec::new();

        // Itera por cada configuração para criar um script de configuração para cada ONU.
        for (i, config_info) in configurations.iter().enumerate() {
            let mut vlan = Vlan::new(params.vlan);
            vlan.pppoe(
                config_info.pppoe_user.clone(),
                config_info.pppoe_password.clone(),
            );

            let services = vec![OnuService::new(vlan)];
            // Cria a ONU
            let onu = Onu::new(
                (i + 1) as u8,
                params.interface.interface(),
                config_info.model.as_str(),
                config_info.sn.as_str(),
                services,
            );
            // Gera o script
            // Adiciona a configuração da ONU no script existente
            configure_script.extend(onu.configure_script());
        }

        Ok(configure_script)
    }
}

// Estrutura que constroi um comando
#[derive(Clone)]
pub struct CommandBuilder<T: Clone, U: Clone> {
    pub command: Rc<str>,
    command_level: PhantomData<T>,
    arg: PhantomData<U>,
}

// Construtor de comandos
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

    pub fn pon_onu_mng(self, pon_interface: (u8, u8, u8), id: u8) -> CommandBuilder<Omci, CmdArg0> {
        CommandBuilder {
            command: format!(
                "pon-onu-mng gpon_onu-{}/{}/{}:{id}",
                pon_interface.0, pon_interface.1, pon_interface.2
            )
            .into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}
