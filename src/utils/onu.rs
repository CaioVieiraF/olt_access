use std::rc::Rc;

use crate::Command;

use super::command::{
    interface::InterfaceOnu,
    omci::{IngressType, Protocol, WanMode},
    CmdArg0, CommandBuilder,
};

pub struct Onu<'a> {
    id: u8,
    interface_pon: (u8, u8, u8),
    model: Box<str>,
    sn: Box<str>,
    services: Rc<[OnuService<'a>]>,
}

#[derive(Clone)]
pub struct OnuService<'a> {
    pub vlan: Vlan,
    pub upload: Option<&'a str>,
    pub download: Option<&'a str>,
}

#[derive(Clone)]
pub struct Vlan {
    pub id: u16,
    pub service: Option<WanMode>,
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

impl OnuService<'_> {
    pub fn new<'a>(vlan: Vlan) -> OnuService<'a> {
        OnuService {
            vlan,
            upload: None,
            download: None,
        }
    }
}

impl<'a> Onu<'a> {
    pub fn new(
        id: u8,
        interface_pon: (u8, u8, u8),
        model: &str,
        sn: &str,
        services: Vec<OnuService<'a>>,
    ) -> Onu<'a> {
        Onu {
            id,
            interface_pon,
            services: Rc::from(services),
            model: Box::from(model),
            sn: Box::from(sn),
            pppoe,
        }
    }

    pub fn configure_script(&self) -> Vec<Command> {
        // Definição das variáveis
        let tcont = 1;
        let speed_profile = "1G";
        let gemport = 1;

        // Cria um vetor vazio onde serão armazenados os comandos.
        let mut script: Vec<Command> = Vec::new();

        // Entra no modo conf t
        let enter_configure = Command::builder();
        script.push(enter_configure.command.clone().into());

        // Comando para entrar na interface pon onde está a ONU.
        let enter_interface_olt = enter_configure.interface().gpon_olt(self.interface_pon);
        script.push(enter_interface_olt.command.clone().into());

        // Comando para adicionar uma ONU não configurada.
        let add_onu = enter_interface_olt
            .onu(self.id)
            .r#type(self.model.clone())
            .sn(self.sn.clone())
            .run();
        script.push(add_onu);
        script.push(Command::exit());

        // Comando para entrar na interface pon da ONU.
        let enter_onu_interface: CommandBuilder<InterfaceOnu, CmdArg0> = Command::builder()
            .interface()
            .gpon_onu(self.interface_pon, self.id);
        script.push(enter_onu_interface.command.clone().into());

        // Configura o tcont com um perfil de velocidade padrão.
        let tcont_profile = enter_onu_interface
            .clone()
            .tcont(tcont)
            .profile(speed_profile);
        script.push(tcont_profile);

        // Cria o gemport
        let gemport_tcont = enter_onu_interface.gemport(gemport).tcont(tcont).run();
        script.push(gemport_tcont);
        script.push(Command::from("vport-mode manual"));
        script.push(Command::from("vport 1 map-type vlan"));

        for service in self.services.iter().enumerate() {
            script.push(Command::from(format!(
                "vport-map 1 {} vlan {}",
                service.0, service.1.vlan.id,
            )));
        }
        script.push(Command::exit());

        // Cria os serviços
        for (index, service) in self.services.iter().enumerate() {
            let vport_id = index as u8 + 1;
            // Entra na interface vport para configurar o serviço
            let enter_vport =
                Command::builder()
                    .interface()
                    .vport(self.interface_pon, self.id, vport_id);
            script.push(enter_vport.command.clone().into());

            let servive_port = enter_vport
                .service_port(vport_id)
                .user_vlan(service.vlan.id)
                .vlan(service.vlan.id)
                .run();
            script.push(servive_port);
        }
        script.push(Command::exit());

        // Entra no modo de configuração OMCI
        let enter_pon_mng = Command::builder().pon_onu_mng(self.interface_pon, self.id);
        script.push(enter_pon_mng.command.clone().into());

        for (index, service) in self.services.iter().enumerate() {
            let service_id = index as u8 + 1;
            // Adiciona a VLAN
            let service_gemport = enter_pon_mng
                .clone()
                .service(service_id)
                .gemport(gemport)
                .vlan(service.vlan.id);
            script.push(service_gemport);

            if let Some(p) = service.vlan.service.clone() {
                // Cria a WAN em pppoe
                let wan_ip = enter_pon_mng
                    .clone()
                    .wan_ip()
                    .mode(p)
                    .vlan_profile(service.vlan.id)
                    .host(service_id);
                script.push(wan_ip);

                // Cria a regra para o acesso web
                let security_mgmt = enter_pon_mng
                    .clone()
                    .security_mgmt(service_id)
                    .state(true)
                    .mode(true)
                    .ingress_type(IngressType::Iphost(1))
                    .protocol(Protocol::Web);
                script.push(security_mgmt);
            }
        }
        script.push(Command::end());
        script
    }
}
