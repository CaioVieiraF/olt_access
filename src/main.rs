mod error;
mod prelude;
mod utils;

use prelude::*;
use std::fs::File;
use utils::{command::Command, configuration::Config};

fn main() -> Result<()> {
    // Cria uma lista de comandos que será transformada em um script
    //let mut script: Vec<Command> = Vec::new();

    // Carrega os arquivos de configuração das ONU
    //let general_info = File::open("parameters.toml")?;
    //let equipment_info = File::open("configure_info.csv")?;

    // Adiciona as configurações das ONU no script
    //Command::onu_script_from_file(general_info, equipment_info, &mut script)?;

    // Finaliza e grava as configurações
    //script.push(Command::end());
    //script.push(Command::write());

    // Cria um objeto de configuração com os comandos criados anteriormente.

    let startrun = File::open("startrun.dat").unwrap_or(File::open("script.txt")?);
    let config = Config::from(startrun);
    //config.show_config();

    let mut onu_script = config.get_onu_script();
    onu_script.push(Command::write());
    let onu_script = Config::from(onu_script);
    //onu_script.show_config();

    // Gera um arquivo para colocar o script.
    let script_file = File::create("output.txt")?;
    // Escreve o script no arquivo.
    //config.to_file(script_file)?;
    onu_script.to_file(script_file)?;

    Ok(())
}
