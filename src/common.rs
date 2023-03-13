/// Checks if a given ip address is a valid ipv4, at the moment really basic in implementation.
pub fn is_ip_valid(ip: &str) -> bool {
    // count how many periods exist in a given ip address, should be 3 e.g. 42.167.30.4 has three periods therefore is valid.
    let period_count = { ip.chars().filter(|char| char.eq(&'.')).count() }; // count how many valid numbers exist in this ip address.
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

/// A struct intended to store a list of all of the requests a user has made, to a limit of <limit> number of requests.
pub struct PreviousRequestsList {
    list: Vec<String>,
    limit: usize,
}

impl PreviousRequestsList {
    /// Returns a new PreviousRequestsList with the given limit
    pub fn new(limit: usize) -> Self {
        Self {
            list: vec![],
            limit,
        }
    }

    /// Returns the list of requests
    pub fn get_list(&self) -> &Vec<String> {
        &self.list
    }

    pub fn get(&self, index: usize) -> Option<&String> {
        self.list.get(index)
    }

    /// Returns a mutable list of requests
    // pub fn get_list_mut(&mut self) -> &mut Vec<String> {
    //     &mut self.list
    // }

    /// Pushes new request string while respecting the limit of the PreviousRequestsList struct.
    /// if the length of the list > the limit, then the oldest request is removed.
    pub fn push(&mut self, request: &str) {
        self.list.push(request.to_string());
        if self.list.len() > self.limit {
            self.list.remove(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_req_list() {
        let mut list = PreviousRequestsList::new(4);

        list.push("fake request 1");
        list.push("fake request 2");
        list.push("fake request 3");
        list.push("fake request 4");

        assert_eq!(list.get_list().len(), 4);

        list.push("fake request 5");

        assert_eq!(list.get_list().len(), 4);

        assert_eq!(list.get(0).unwrap(), "fake request 2");
        assert_eq!(list.get(1).unwrap(), "fake request 3");

        list.push("fake request 6");

        assert_eq!(list.get(0).unwrap(), "fake request 3");
        assert_eq!(list.get(1).unwrap(), "fake request 4");
    }

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
            assert!(!is_ip_valid(ip));
        }

        let valid_ips = [
            "12.94.122.150",
            "98.124.74.1",
            "17.38.42.56",
            "67.184.56.122",
        ];

        for ip in valid_ips {
            assert!(is_ip_valid(ip));
        }

        for a in -100..=300 {
            let ip1 = format!("67.67.67.{}", a);
            let ip2 = format!("67.67.{}.67", a);
            let ip3 = format!("67.{}.67.67", a);
            let ip4 = format!("{}.67.67.67", a);
            let ip5 = format!("{0}.{0}.{0}.{0}", a);

            let should_be_valid = (0..=255).contains(&a);

            assert_eq!(should_be_valid, is_ip_valid(&ip1));
            assert_eq!(should_be_valid, is_ip_valid(&ip2));
            assert_eq!(should_be_valid, is_ip_valid(&ip3));
            assert_eq!(should_be_valid, is_ip_valid(&ip4));
            assert_eq!(should_be_valid, is_ip_valid(&ip5));
        }
    }
}
