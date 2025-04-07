use cloudflare_ddns::domain::config::{DdnsConfig, IpType};
use cloudflare_ddns::domain::error::DomainError;

#[cfg(test)]
mod ip_type_tests {
    use super::*;

    #[test]
    fn test_ip_type_display() {
        assert_eq!(IpType::IPv4.to_string(), "ipv4");
        assert_eq!(IpType::IPv6.to_string(), "ipv6");
    }

    #[test]
    fn test_ip_type_try_from() {
        // 有效類型
        assert_eq!(IpType::try_from("ipv4").unwrap(), IpType::IPv4);
        assert_eq!(IpType::try_from("ipv6").unwrap(), IpType::IPv6);
        assert_eq!(IpType::try_from("IPv4").unwrap(), IpType::IPv4);
        assert_eq!(IpType::try_from("IPv6").unwrap(), IpType::IPv6);

        // 無效類型
        assert!(IpType::try_from("invalid").is_err());
    }
}

#[cfg(test)]
mod ddns_config_tests {
    use super::*;

    fn create_valid_config() -> DdnsConfig {
        DdnsConfig {
            api_token: "api_token".to_string(),
            zone_id: "zone_id".to_string(),
            record_id: "record_id".to_string(),
            record_name: "example.com".to_string(),
            update_interval: 300,
            ip_type: IpType::IPv4,
        }
    }

    #[test]
    fn test_valid_config() {
        let config = create_valid_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_api_token() {
        let mut config = create_valid_config();
        config.api_token = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                DomainError::Validation(msg) => {
                    assert!(msg.contains("API token"));
                }
                _ => panic!("應該返回 Validation"),
            }
        }
    }

    #[test]
    fn test_invalid_zone_id() {
        let mut config = create_valid_config();
        config.zone_id = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                DomainError::Validation(msg) => {
                    assert!(msg.contains("Zone ID"));
                }
                _ => panic!("應該返回 Validation"),
            }
        }
    }

    #[test]
    fn test_invalid_record_id() {
        let mut config = create_valid_config();
        config.record_id = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                DomainError::Validation(msg) => {
                    assert!(msg.contains("Record ID"));
                }
                _ => panic!("應該返回 Validation"),
            }
        }
    }

    #[test]
    fn test_invalid_record_name() {
        let mut config = create_valid_config();
        config.record_name = "".to_string();
        let result = config.validate();
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                DomainError::Validation(msg) => {
                    assert!(msg.contains("Record name"));
                }
                _ => panic!("應該返回 Validation"),
            }
        }
    }

    #[test]
    fn test_invalid_update_interval() {
        let mut config = create_valid_config();
        config.update_interval = 4;
        let result = config.validate();
        assert!(result.is_err());
        if let Err(e) = result {
            match e {
                DomainError::Validation(msg) => {
                    assert!(msg.contains("Update interval"));
                }
                _ => panic!("應該返回 Validation"),
            }
        }
    }
} 