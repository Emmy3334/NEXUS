//! Port column formatting for `@docker ps`.

use bollard::models::{Port, PortTypeEnum};

pub(super) fn format_ports(ports: &[Port]) -> String {
    if ports.is_empty() {
        return String::new();
    }
    let mut parts: Vec<String> = ports.iter().map(format_port).collect();
    parts.sort();
    parts.dedup();
    parts.join(", ")
}

fn format_port(port: &Port) -> String {
    let proto = match port.typ {
        Some(PortTypeEnum::UDP) => "udp",
        Some(PortTypeEnum::SCTP) => "sctp",
        _ => "tcp",
    };
    match (port.ip.as_deref(), port.public_port) {
        (Some(ip), Some(public)) => {
            format!("{ip}:{public}->{}:{proto}", port.private_port)
        }
        (_, Some(public)) => format!("0.0.0.0:{public}->{}:{proto}", port.private_port),
        _ => format!("{}:{proto}", port.private_port),
    }
}
