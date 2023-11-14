mod error;
mod prelude;
mod utils;

#[macro_use]
extern crate log;
use clap::{Parser, Subcommand};
use prelude::*;
use std::{fs::File, path::PathBuf};
use utils::{
    command::Command,
    configuration::{Config, ConfigField},
    olt::Interface,
};

#[derive(Parser, Debug, PartialEq)]
#[command(author, version, about)]
struct Args {
    /// Opções para a criação de script
    #[command(subcommand)]
    command: Commands,
    /// Arquivo final para guardar o script
    #[arg(short, long, value_name = "FILE")]
    output: PathBuf,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Commands {
    /// Migrar script da linha antiga para a nova
    Migrate {
        /// Arquivo de configuração antigo
        #[arg(short, long, value_name = "ARQUIVO")]
        old: PathBuf,
        /// Arquivo de configuração base
        #[arg(short, long, value_name = "ARQUIVO")]
        base: PathBuf,
    },
    /// Criar script de ONU a partir de arquivos com as informações
    Create {
        /// Arquivo .csv com as informações das ONU: SN, perfil e usuário e senha PPPoE
        #[arg(short, long, value_name = "ARQUIVO.csv")]
        onu_param: PathBuf,

        /// Vlan que será utilizada para configurar as ONU
        #[arg(short, long, value_name = "VLAN_ID")]
        vlan: u16,

        /// Interface PON em que as ONU estão situadas
        #[arg(short, long, value_name = "gpon_olt-1/x/y")]
        interface: Interface,
    },

    Show {
        /// Arquivo para mostrar
        #[arg(long, value_name = "ARQUIVO")]
        from: PathBuf,
        #[arg(long, value_name = "CAMPO")]
        field: Option<ConfigField>,
    },
}

fn main() -> Result<()> {
    // Inicia o serviço de log
    env_logger::init();

    // Cria uma lista de comandos que será transformada em um script
    let mut script = Config::default();

    // Opções passadas na linha de comando.
    let cli_args = Args::parse();

    // Verifica o comando utilizado
    match cli_args.command {
        Commands::Migrate { old, base } => {
            // Cria um objeto de configuração a partir de um backup de uma OLT.
            let startrun = File::open(old)?;
            let base_file = File::open(base)?;
            let config = Config::from(base_file);
            script += config;

            let config = Config::from(startrun);

            let onu_script = config.extract_onu();
            for onu in onu_script {
                let cmd = onu.configure_script();
                script += cmd;
            }
        }
        Commands::Create {
            onu_param,
            vlan,
            interface,
        } => {
            // Carrega o arquivo de configuração das ONU
            let equipment_info = File::open(onu_param)?;

            // Adiciona as configurações das ONU no script
            let new_script = Command::onu_script_from_file(equipment_info, vlan, interface)?;
            script += new_script;
        }
        Commands::Show { from, field } => {
            let file = File::open(from)?;
            let config = Config::from(file);

            if let Some(f) = field {
                if let Some(c) = config.0.get(&f) {
                    println!("===============");
                    for command in c {
                        println!("{command:#?}");
                    }
                    println!("===============");
                } else {
                    println!("O campo `{f}` campo não existe.");
                }
            } else {
                for (field, commands) in config.0.iter() {
                    println!("===============");
                    println!("Showing commands in {field}");
                    for command in commands {
                        println!("{command:#?}");
                    }
                    println!("===============");
                }
            }
        }
    }

    // Gera um arquivo para colocar o script.
    let script_file = File::create(cli_args.output)?;
    // Escreve o script no arquivo.
    script.to_file(script_file)?;
    Ok(())
}
