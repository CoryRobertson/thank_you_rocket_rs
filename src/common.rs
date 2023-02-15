/// Checks if a given ip address is a valid ipv4, at the moment really basic in implementation.
pub fn is_ip_valid(ip: &str) -> bool {
    // count how many periods exist in a given ip address, should be 3 e.g. 42.167.30.4 has three periods therefore is valid.
    let period_count = { ip.chars().filter(|char| char.eq(&'.')).count() };
    // count how many valid numbers exist in this ip address.
    let four_valid = ip
        .split('.') // split the line given by its periods
        .filter(|num_split| {
            // only keep lines that are possible to be parsed into a 8u
            num_split.parse::<u8>().is_ok()
        })
        .count()
        == 4;

    four_valid && period_count == 3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_ips() {
        let invalid_ips = [
            "12.56.78",
            "-3.254.100.88",
            "256.122.80.23",
            "129.300..00",
            "1..2.3.4",
        ];

        for ip in invalid_ips {
            assert_eq!(false, is_ip_valid(ip));
        }

        let valid_ips = [
            "12.94.122.150",
            "98.124.74.1",
            "17.38.42.56",
            "67.184.56.122",
        ];

        for ip in valid_ips {
            assert_eq!(true, is_ip_valid(ip));
        }

        for a in -100..=300 {
            let ip1 = format!("67.67.67.{}", a);
            let ip2 = format!("67.67.{}.67", a);
            let ip3 = format!("67.{}.67.67", a);
            let ip4 = format!("{}.67.67.67", a);
            let ip5 = format!("{0}.{0}.{0}.{0}", a);

            let should_be_valid = { !(a < 0 || a > 255) };

            assert_eq!(should_be_valid, is_ip_valid(&ip1));
            assert_eq!(should_be_valid, is_ip_valid(&ip2));
            assert_eq!(should_be_valid, is_ip_valid(&ip3));
            assert_eq!(should_be_valid, is_ip_valid(&ip4));
            assert_eq!(should_be_valid, is_ip_valid(&ip5));
        }
    }
}
