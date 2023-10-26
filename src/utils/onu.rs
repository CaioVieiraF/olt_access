use crate::Command;

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

    pub fn configure_script(&self, pppoe_user: &str, pppoe_password: &str) -> Vec<Command> {
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
        script.push(Command::from("exit"));

        // Comando para entrar na interface pon da ONU.
        let enter_onu_interface = Command::from(format!(
            "interface gpon_onu-{}/{}/{}:{}",
            self.interface_pon.0, self.interface_pon.1, self.interface_pon.2, self.id
        ));
        script.push(enter_onu_interface);

        // Configura o tcont com um perfil de velocidade padrão.
        script.push(Command::from("tcont 1 profile 1G"));

        // Cria o gemport
        script.push(Command::from("gemport 1 tcont 1"));
        script.push(Command::from("exit"));

        // Entra na interface vport para configurar o serviço
        let enter_vport = Command::from(format!(
            "interface vport-{}/{}/{}.{}:1",
            self.interface_pon.0, self.interface_pon.1, self.interface_pon.2, self.id
        ));
        script.push(enter_vport);

        // Cria o serviço com a VLAN
        let servive_port = Command::from(format!(
            "service-port 1 user-vlan {} vlan {}",
            self.vlan, self.vlan
        ));
        script.push(servive_port);
        script.push(Command::from("exit"));

        // Entra no modo de configuração OMCI
        let enter_pon_mng = Command::from(format!(
            "pon-onu-mng gpon_onu-{}/{}/{}:{}",
            self.interface_pon.0, self.interface_pon.1, self.interface_pon.2, self.id
        ));
        script.push(enter_pon_mng);

        // Adiciona a VLAN
        let service_gemport = Command::from(format!("service 1 gemport 1 vlan {}", self.vlan));
        script.push(service_gemport);

        let wan_ip = Command::from(format!("wan-ip ipv4 mode pppoe username {pppoe_user} password {pppoe_password} vlan-profile {} host 1", self.vlan));
        script.push(wan_ip);

        // Cria a regra para o acesso web
        script.push(Command::from(
            "security-mgmt 1 mode forward state enable ingress-type iphost 1 protocol web",
        ));
        script.push(Command::from("end"));

        script
    }
}
