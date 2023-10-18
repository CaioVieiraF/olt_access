mod error;
mod prelude;
mod utils;

use crate::utils::olt::Titan::C620;
use prelude::*;
use std::net::Ipv4Addr;
use utils::{
    configuration::Config,
    olt::{Olt, OltModel, Titan},
};

fn main() -> Result<()> {
    let config = Config::from("test.txt")?;

    config.show_config();

    let addr = Ipv4Addr::new(172, 16, 10, 5);
    let olt_model = OltModel::Titan(Titan::C650);

    let olt = Olt::builder().model(olt_model).ip(addr).build()?;
    // olt.configure()
    Ok(())
}
