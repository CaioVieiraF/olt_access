pub mod interface;
pub mod omci;

use std::{fs::File, marker::PhantomData, rc::Rc};

use crate::prelude::Result;

use super::{
    configuration::Config,
    configuration::ConfigInfo,
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
#[derive(Clone, Debug)]
pub struct Command(pub Rc<str>);

// Cria um comando a partir de um texto literal
impl From<&str> for Command {
    fn from(value: &str) -> Command {
        Command(value.into())
    }
}
// Cria um comando a partir de um ponteiro de texto
impl From<Rc<str>> for Command {
    fn from(value: Rc<str>) -> Command {
        Command(value)
    }
}
// Cria um comando a partir de um texto literal
impl From<String> for Command {
    fn from(value: String) -> Command {
        Command(value.into())
    }
}
impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Command {
    // Devolve o comando como um texto
    pub fn as_str(&self) -> &str {
        &self.0
    }
    // Devolve o comando como um ponteiro de texto
    pub fn raw(&self) -> Rc<str> {
        self.0.clone()
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
    pub fn onu_script_from_file(
        equipment_info: File,
        vlan: u16,
        interface: Interface,
    ) -> Result<Config> {
        // Carrega o arquivo em uma estrutura conhecida, caso esteja
        // no formato certo
        let configurations = ConfigInfo::from_file(equipment_info)?;
        let mut configure_script = Config::default();

        // Itera por cada configuração para criar um script de configuração para cada ONU.
        for config_info in configurations.iter() {
            let mut vlan = Vlan::new(vlan);
            vlan.pppoe(
                config_info.pppoe_user.clone(),
                config_info.pppoe_password.clone(),
            );

            let services = vec![OnuService::new(vlan)];
            // Cria a ONU
            let onu = Onu::new(
                interface.clone(),
                config_info.model.as_str(),
                config_info.sn.as_str(),
                services,
            );
            // Gera o script
            // Adiciona a configuração da ONU no script existente
            configure_script += onu.configure_script();
        }

        Ok(configure_script)
    }
}

// Estrutura que constroi um comando
#[derive(Clone)]
pub struct CommandBuilder<T: Clone, U: Clone> {
    pub command: Command,
    command_level: PhantomData<T>,
    arg: PhantomData<U>,
}

// Construtor de comandos
impl CommandBuilder<ConfT, CmdArg0> {
    pub fn new() -> Self {
        CommandBuilder {
            command: Command::from("configure terminal"),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn interface(self) -> CommandBuilder<Interface, CmdArg0> {
        CommandBuilder {
            command: Command::from("interface"),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }

    pub fn pon_onu_mng(self, interface: &Interface) -> CommandBuilder<Omci, CmdArg0> {
        CommandBuilder {
            command: format!(
                "pon-onu-mng gpon_onu-1/{}/{}:{}",
                interface.slot,
                interface.port,
                interface.id.unwrap()
            )
            .into(),
            command_level: PhantomData,
            arg: PhantomData,
        }
    }
}
