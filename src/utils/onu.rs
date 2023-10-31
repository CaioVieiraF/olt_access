use crate::Command;

use super::{
    command::{
        interface::InterfaceOnu,
        omci::{IngressType, Protocol},
        CmdArg0, CommandBuilder,
    },
    configuration::PppoeInfo,
};

#[derive(Debug)]
pub enum OnuModel {
    F670LV9,
}

pub struct Onu {
    id: u8,
    interface_pon: (u8, u8, u8),
    vlan: u16,
    model: Box<str>,
    sn: Box<str>,
}

impl Onu {
    pub fn new(id: u8, interface_pon: (u8, u8, u8), vlan: u16, model: &str, sn: &str) -> Onu {
        Onu {
            id,
            interface_pon,
            vlan,
            model: Box::from(model),
            sn: Box::from(sn),
        }
    }

    pub fn configure_script(&self, pppoe: Option<&PppoeInfo>) -> Vec<Command> {
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
        let tcont_profile = enter_onu_interface.clone().tcont(1).profile("1G");
        script.push(tcont_profile);

        // Cria o gemport
        let gemport_tcont = enter_onu_interface.gemport(1).tcont(1).run();
        script.push(gemport_tcont);
        script.push(Command::exit());

        // Entra na interface vport para configurar o serviço
        let enter_vport = Command::builder()
            .interface()
            .vport(self.interface_pon, self.id, 1);
        script.push(enter_vport.command.clone().into());

        // Cria o serviço com a VLAN
        let servive_port = enter_vport
            .service_port(1)
            .user_vlan(self.vlan)
            .vlan(self.vlan)
            .run();
        script.push(servive_port);
        script.push(Command::exit());

        // Entra no modo de configuração OMCI
        let enter_pon_mng = Command::builder().pon_onu_mng(self.interface_pon, self.id);
        script.push(enter_pon_mng.command.clone().into());

        // Adiciona a VLAN
        let service_gemport = enter_pon_mng.clone().service(1).gemport(1).vlan(self.vlan);
        script.push(service_gemport);

        if let Some(p) = pppoe {
            let user = &p.user;
            let pass = &p.password;
            // Cria a WAN em pppoe
            let wan_ip = enter_pon_mng
                .clone()
                .wan_ip()
                .mode()
                .pppoe(user, pass)
                .vlan_profile(self.vlan)
                .host(1);
            script.push(wan_ip);

            // Cria a regra para o acesso web
            let security_mgmt = enter_pon_mng
                .security_mgmt(1)
                .state(true)
                .mode(true)
                .ingress_type(IngressType::Iphost(1))
                .protocol(Protocol::Web);
            script.push(security_mgmt);
        }

        script.push(Command::end());
        script
    }
}
