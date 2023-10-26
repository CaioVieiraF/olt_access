mod error;
mod prelude;
mod utils;

use prelude::*;
use serde::Deserialize;
use std::fs::File;
use utils::{
    command::Command,
    configuration::{Config, GeneralParam},
    onu::Onu,
};

#[derive(Deserialize)]
struct ConfigInfo {
    pub sn: String,
    pub pppoe_user: String,
    pub pppoe_password: String,
    pub model: String,
}

impl ConfigInfo {
    fn from_file(file: File) -> Result<Vec<ConfigInfo>> {
        let mut infos: Vec<ConfigInfo> = Vec::new();
        let mut reader = csv::Reader::from_reader(file);
        for result in reader.deserialize() {
            let record: ConfigInfo = result?;
            infos.push(record);
        }

        Ok(infos)
    }
}

fn main() -> Result<()> {
    let mut script: Vec<Command> = Vec::new();

    // Carrega os arquivos de configuração
    let paramenter_file = File::open("parameters.toml")?;
    let equipment_file = File::open("configure_info.csv")?;
    let params = GeneralParam::try_from(paramenter_file)?;
    let configurations = ConfigInfo::from_file(equipment_file)?;

    // Itera por cada configuração para criar um script de configuração para cada ONU.
    for (i, config_info) in configurations.iter().enumerate() {
        // Cria a ONU
        let onu = Onu::new(
            (i + 1) as u8,
            params.interface.interface(),
            params.vlan,
            config_info.model.as_str(),
            config_info.sn.as_str(),
        );

        // Gera o script
        let configure_script = onu.configure_script(
            config_info.pppoe_user.as_str(),
            config_info.pppoe_password.as_str(),
        );
        script.extend(configure_script);
    }

    // Finaliza e grava as configurações
    script.push(Command::from("end"));
    script.push(Command::from("write"));

    // Cria um objeto de configuração com os comandos criados anteriormente.
    let config = Config::from(script);
    // Gera um arquivo para colocar o script.
    let script_file = File::open("output.txt").unwrap_or(File::create("output.txt")?);
    // Escreve o script no arquivo.
    config.to_file(script_file)?;

    Ok(())
}
