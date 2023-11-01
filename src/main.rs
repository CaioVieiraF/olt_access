mod error;
mod prelude;
mod utils;

use prelude::*;
use std::fs::File;
use utils::{command::Command, configuration::Config};

fn main() -> Result<()> {
    // Cria uma lista de comandos que será transformada em um script
    let mut script: Vec<Command> = Vec::new();

    // Carrega os arquivos de configuração das ONU
    //let general_info = File::open("parameters.toml")?;
    //let equipment_info = File::open("configure_info.csv")?;

    // Adiciona as configurações das ONU no script
    //Command::onu_script_from_file(general_info, equipment_info, &mut script)?;

    // Cria um objeto de configuração a partir de um backup de uma OLT.
    let startrun = File::open("startrun.dat")?;
    let config = Config::from(startrun);

    let onu_script = config.extract_onu(1000);
    for onu in onu_script {
        let cmd = onu.configure_script();
        script.extend(cmd);
    }

    // Finaliza e grava as configurações
    script.push(Command::write());
    let onu_script = Config::from(script);

    // Gera um arquivo para colocar o script.
    let script_file = File::create("output.txt")?;
    // Escreve o script no arquivo.
    onu_script.to_file(script_file)?;

    Ok(())
}
