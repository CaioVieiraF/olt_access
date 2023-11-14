use std::{collections::HashMap, rc::Rc};

use regex::Regex;

use crate::{
    prelude::{Error, Result},
    Command,
};

use super::{
    command::{
        interface::InterfaceOnu,
        omci::{IngressType, Protocol, WanMode},
        CmdArg0, CommandBuilder,
    },
    configuration::{Config, ConfigField, NestedCommand},
    olt::Interface,
};

pub struct Onu {
    interface: Interface,
    model: Box<str>,
    sn: Box<str>,
    services: Rc<[OnuService]>,
}

#[derive(Clone)]
pub struct OnuService {
    pub vlan: Vlan,
    pub upload: Option<Box<str>>,
    pub download: Option<Box<str>>,
}

#[derive(Clone)]
pub struct Vlan {
    pub id: u16,
    pub service: Option<WanMode>,
}

impl TryFrom<&Command> for Vlan {
    type Error = Error;

    fn try_from(value: &Command) -> Result<Self> {
        let wan_pattern = Regex::new(
            r"^wan-ip [0-9] mode (?P<service>(pppoe username (?P<username>.*) password (?P<password>.*))|(dhcp)) vlan-profile (?P<vlan>.*) host [0-9]",
        ).unwrap();

        if let Some(info) = wan_pattern.captures(value.as_str()) {
            let id = info["vlan"].parse::<u16>().unwrap();
            let mut new_vlan = Vlan::new(id);

            if info["service"].is_empty() {
                if info["username"].is_empty() {
                    new_vlan.dhcp()
                } else {
                    new_vlan.pppoe(&info["username"], &info["password"])
                }
            }

            Ok(new_vlan)
        } else {
            Err(Error::Generic("Parse vlan".to_string()))
        }
    }
}

impl Vlan {
    pub fn new(id: u16) -> Vlan {
        Vlan { id, service: None }
    }

    pub fn pppoe(&mut self, username: impl Into<String>, password: impl Into<String>) {
        self.service = Some(WanMode::PPPoE {
            username: username.into(),
            password: password.into(),
        });
    }

    pub fn dhcp(&mut self) {
        self.service = Some(WanMode::Dhcp);
    }
}

impl OnuService {
    pub fn new(vlan: Vlan) -> OnuService {
        OnuService {
            vlan,
            upload: None,
            download: None,
        }
    }
}

impl Onu {
    pub fn new(interface: Interface, model: &str, sn: &str, services: Vec<OnuService>) -> Onu {
        Onu {
            interface,
            services: Rc::from(services),
            model: Box::from(model),
            sn: Box::from(sn),
        }
    }

    pub fn interface(&self) -> &Interface {
        &self.interface
    }

    pub fn set_service(&mut self, services: Rc<[OnuService]>) {
        self.services = services;
    }

    pub fn configure_script(&self) -> Config {
        // Definição das variáveis
        let tcont = 1;
        let speed_profile = "1G";
        let gemport = 1;

        // Cria um mapa vazio onde serão armazenados os comandos.
        let mut script = HashMap::new();
        let interface_field = ConfigField::from("if-intf");
        let xpon_field = ConfigField::from("xpon");
        let msan_field = ConfigField::from("MSAN");

        script.insert(interface_field.clone(), Vec::new());
        script.insert(xpon_field.clone(), Vec::new());
        script.insert(msan_field.clone(), Vec::new());

        // Entra no modo conf t
        let enter_configure = Command::builder();

        // Comando para entrar na interface pon onde está a ONU.
        let enter_interface_olt = enter_configure.interface().gpon_olt(self.interface());
        let mut interface_olt = NestedCommand::from(enter_interface_olt.command.clone());

        // Comando para adicionar uma ONU não configurada.
        let add_onu = enter_interface_olt
            .onu(self.interface.id.unwrap())
            .r#type(self.model.clone())
            .sn(self.sn.clone())
            .run();
        interface_olt.nest(add_onu.into());
        script
            .get_mut(&interface_field)
            .unwrap()
            .push(interface_olt);

        // Comando para entrar na interface pon da ONU.
        let enter_onu_interface: CommandBuilder<InterfaceOnu, CmdArg0> =
            Command::builder().interface().gpon_onu(self.interface());
        let mut interface_onu = NestedCommand::from(enter_onu_interface.command.clone());
        script
            .get_mut(&interface_field)
            .unwrap()
            .push(interface_onu.clone());

        // Configura o tcont com um perfil de velocidade padrão.
        let tcont_profile = enter_onu_interface
            .clone()
            .tcont(tcont)
            .profile(speed_profile);
        interface_onu.nest(tcont_profile.into());

        // Cria o gemport
        let gemport_tcont = enter_onu_interface.gemport(gemport).tcont(tcont).run();
        interface_onu.nest(gemport_tcont.into());
        interface_onu.nest(Command::from("vport-mode manual").into());
        interface_onu.nest(Command::from("vport 1 map-type vlan").into());

        for service in self.services.iter().enumerate() {
            interface_onu.nest(
                Command::from(
                    format!("vport-map 1 {} vlan {}", service.0, service.1.vlan.id,).as_str(),
                )
                .into(),
            );
        }

        script.get_mut(&xpon_field).unwrap().push(interface_onu);

        // Cria os serviços
        for (index, service) in self.services.iter().enumerate() {
            let vport_id = index as u8 + 1;
            // Entra na interface vport para configurar o serviço
            let enter_vport = Command::builder()
                .interface()
                .vport(self.interface(), vport_id);
            let mut interface_vport = NestedCommand::from(enter_vport.command.clone());
            script
                .get_mut(&interface_field)
                .unwrap()
                .push(interface_vport.clone());

            let servive_port = enter_vport
                .service_port(vport_id)
                .user_vlan(service.vlan.id)
                .vlan(service.vlan.id)
                .run();
            interface_vport.nest(servive_port.into());

            script.get_mut(&msan_field).unwrap().push(interface_vport);
        }

        // Entra no modo de configuração OMCI
        let enter_pon_mng = Command::builder().pon_onu_mng(self.interface());
        let mut pon_onu_mng = NestedCommand::from(enter_pon_mng.command.clone());

        for (index, service) in self.services.iter().enumerate() {
            let service_id = index as u8 + 1;
            // Adiciona a VLAN
            let service_gemport = enter_pon_mng
                .clone()
                .service(service_id)
                .gemport(gemport)
                .vlan(service.vlan.id);
            pon_onu_mng.nest(service_gemport.into());

            if let Some(p) = service.vlan.service.clone() {
                // Cria a WAN em pppoe
                let wan_ip = enter_pon_mng
                    .clone()
                    .wan_ip()
                    .mode(p)
                    .vlan_profile(service.vlan.id)
                    .host(service_id);
                pon_onu_mng.nest(wan_ip.into());

                // Cria a regra para o acesso web
                let security_mgmt = enter_pon_mng
                    .clone()
                    .security_mgmt(service_id)
                    .state(true)
                    .mode(true)
                    .ingress_type(IngressType::Iphost(1))
                    .protocol(Protocol::Web);
                pon_onu_mng.nest(security_mgmt.into());
            }
        }

        script.get_mut(&xpon_field).unwrap().push(pon_onu_mng);
        Config(script)
    }
}
