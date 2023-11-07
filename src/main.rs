mod error;
mod prelude;
mod utils;

use clap::{Parser, Subcommand};
use prelude::*;
use std::{fs::File, path::PathBuf};
use utils::{command::Command, configuration::Config};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Opções para a criação de script
    #[command(subcommand)]
    command: Option<Commands>,
    /// Arquivo final para guardar o script
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrar script da linha antiga para a nova
    Migrate {
        /// Arquivo de configuração antigo
        #[arg(short, long, value_name = "FILE")]
        old: PathBuf,
    },
    /// Criar script de ONU a partir de arquivos com as informações
    Create {
        /// Arquivo .toml com a VLAN e a interface PON das ONU
        #[arg(short, long, value_name = "FILE")]
        general: PathBuf,
        /// Arquivo .csv com as informações das ONU: SN, perfil e usuário e senha PPPoE
        #[arg(short, long, value_name = "FILE")]
        onu_param: PathBuf,
    },
}

fn main() -> Result<()> {
    // Cria uma lista de comandos que será transformada em um script
    let mut script: Vec<Command> = Vec::new();

    // Opções passadas na linha de comando.
    let cli_args = Args::parse();

    if let Some(command) = cli_args.command {
        match command {
            Commands::Migrate { old } => {
                // Cria um objeto de configuração a partir de um backup de uma OLT.
                let startrun = File::open(old)?;
                let config = Config::from(startrun);

                let onu_script = config.extract_onu(1000);
                for onu in onu_script {
                    let cmd = onu.configure_script();
                    script.extend(cmd);
                }

                // Finaliza as configurações
                script.push(Command::write());
            }
            Commands::Create { general, onu_param } => {
                // Carrega os arquivos de configuração das ONU
                let general_info = File::open(general)?;
                let equipment_info = File::open(onu_param)?;

                // Adiciona as configurações das ONU no script
                let new_script = Command::onu_script_from_file(general_info, equipment_info)?;
                script.extend(new_script);
            }
        }

        // Gera um arquivo para colocar o script.
        let configure_script = Config::from(script);
        let script_file = File::create(cli_args.output)?;
        // Escreve o script no arquivo.
        configure_script.to_file(script_file)?;
    }
    Ok(())
}
