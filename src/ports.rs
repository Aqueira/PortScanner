pub struct Port {
    pub num: u16,
    pub name: String,
    pub port_type: PortType,
}

impl Port {
    pub fn new(num: u16, name: &str, port_type: PortType) -> Self {
        Self {
            num,
            name: name.to_string(),
            port_type,
        }
    }

    pub fn from(port: u16) -> Self {
        match port {
            20 => Self::new(port, "FTP", PortType::FTP),
            21 => Self::new(port, "Secure FTP", PortType::FTP),
            22 => Self::new(port, "SSH", PortType::SSH),
            23 => Self::new(port, "Telnet", PortType::Telnet),
            25 => Self::new(port, "SMTP", PortType::SMTP),
            53 => Self::new(port, "DNS", PortType::DNS),
            67 | 68 => Self::new(port, "DHCP", PortType::DHCP),
            110 => Self::new(port, "POP3", PortType::POP3),
            143 => Self::new(port, "IMAP", PortType::IMAP),
            161 | 162 => Self::new(port, "SNMP", PortType::SNMP),
            389 => Self::new(port, "LDAP", PortType::LDAP),
            443 => Self::new(port, "HTTPS", PortType::HTTPS),
            465 => Self::new(port, "SMTPS", PortType::SMTPS),
            587 => Self::new(port, "SMTP", PortType::SMTP),
            993 => Self::new(port, "IMAPS", PortType::IMAPS),
            995 => Self::new(port, "POP3s", PortType::POP3S),
            2083 => Self::new(port, "CPanel", PortType::CPanel),
            2086 => Self::new(port, "WHM", PortType::WHM),
            2087 => Self::new(port, "WhmSsL", PortType::WhmSsL),
            3306 => Self::new(port, "MySQL", PortType::MYSQL),
            3389 => Self::new(port, "RDP", PortType::RDP),
            5432 => Self::new(port, "PostgreSQL", PortType::PostgreSQL),
            6379 => Self::new(port, "Redis", PortType::Redis),
            8080 => Self::new(port, "HttpProxy", PortType::HttpProxy),
            8880 => Self::new(port, "HTTP", PortType::HTTP),
            8443 => Self::new(port, "HostingHttp", PortType::HostingHttp),
            80 => Self::new(port, "HTTP", PortType::HTTP),
            _ => Self::new(port, "Other", PortType::Other),
        }
    }
}

pub enum PortType {
    FTP,
    SSH,
    Telnet,
    SMTP,
    POP3,
    IMAP,
    HTTPS,
    HTTP,
    DNS,
    DHCP,
    SNMP,
    LDAP,
    SMTPS,
    IMAPS,
    POP3S,
    MYSQL,
    RDP,
    PostgreSQL,
    Redis,
    CPanel,
    HttpProxy,
    HostingHttp,
    WHM,
    WhmSsL,
    Other,
}
