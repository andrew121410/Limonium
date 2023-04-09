pub fn sort_versions_one_decimal_and_two_decimal_lowest_to_highest(versions: &mut Vec<String>) {
    versions.sort_by(|a, b| {
        let a_parts: Vec<&str> = a.split('.').collect();
        let b_parts: Vec<&str> = b.split('.').collect();

        let a_major: u32 = a_parts[0].parse().unwrap();
        let b_major: u32 = b_parts[0].parse().unwrap();
        if a_major != b_major {
            return a_major.cmp(&b_major);
        }

        let a_minor: u32 = a_parts[1].parse().unwrap();
        let b_minor: u32 = b_parts[1].parse().unwrap();
        if a_minor != b_minor {
            return a_minor.cmp(&b_minor);
        }

        if a_parts.len() == 2 && b_parts.len() == 2 {
            return std::cmp::Ordering::Equal;
        }

        if a_parts.len() == 2 {
            return std::cmp::Ordering::Less;
        }

        if b_parts.len() == 2 {
            return std::cmp::Ordering::Greater;
        }

        let a_patch: u32 = a_parts[2].parse().unwrap();
        let b_patch: u32 = b_parts[2].parse().unwrap();
        a_patch.cmp(&b_patch)
    });
}

fn sort_versions_one_decimal_and_two_decimal_highest_to_lowest(versions: &mut Vec<String>) {
    sort_versions_one_decimal_and_two_decimal_lowest_to_highest(versions);
    versions.reverse();
}

// Can sort versions with one decimal and two decimals and can sort versions with snapshots
pub fn sort_versions(versions: &mut Vec<String>) {
    versions.sort_by(|a, b| {
        let a_parts: Vec<&str> = a.split('.').collect();
        let b_parts: Vec<&str> = b.split('.').collect();

        for i in 0..3 {
            let a_part = a_parts.get(i).and_then(|s| s.parse::<u64>().ok());
            let b_part = b_parts.get(i).and_then(|s| s.parse::<u64>().ok());

            match (a_part, b_part) {
                (Some(ap), Some(bp)) => {
                    if ap != bp {
                        return ap.cmp(&bp);
                    }
                }
                (None, Some(_)) => return std::cmp::Ordering::Less,
                (Some(_), None) => return std::cmp::Ordering::Greater,
                (None, None) => (),
            }
        }

        a.cmp(b)
    });
}

#[cfg(test)]
mod number_utils_testing {
    use super::*;

    #[test]
    fn test_sort_versions_one_decimal_and_two_decimal_lowest_to_highest() {
        let mut versions = vec!["3.30".to_string(), "4.0.2".to_string(), "1.0.0".to_string(), "1.02".to_string(), "1.30".to_string(), "2.19".to_string(), "2.19.100".to_string()];
        sort_versions_one_decimal_and_two_decimal_lowest_to_highest(&mut versions);
        assert_eq!(versions, vec!["1.0.0".to_string(), "1.02".to_string(), "1.30".to_string(), "2.19".to_string(), "2.19.100".to_string(), "3.30".to_string(), "4.0.2".to_string()]);
    }

    #[test]
    fn test_sort_versions_one_decimal_and_two_decimal_highest_to_lowest() {
        let mut versions = vec!["3.30".to_string(), "4.0.2".to_string(), "1.0.0".to_string(), "1.02".to_string(), "1.30".to_string(), "2.19".to_string(), "2.19.100".to_string()];
        sort_versions_one_decimal_and_two_decimal_highest_to_lowest(&mut versions);
        assert_eq!(versions, vec!["4.0.2".to_string(), "3.30".to_string(), "2.19.100".to_string(), "2.19".to_string(), "1.30".to_string(), "1.02".to_string(), "1.0.0".to_string()]);
    }

    #[test]
    fn test_sort_versions() {
        let mut versions = vec!["3.30".to_string(), "4.0.2".to_string(), "5.0-SNAPSHOT".to_string(), "1.0.0".to_string(), "5.0.1-SNAPSHOT".to_string(), "3.0.1-SNAPSHOT".to_string(), "1.02".to_string(), "5.0".to_string(), "1.30".to_string(), "2.19".to_string(), "2.19.100".to_string()];
        sort_versions(&mut versions);
        assert_eq!(versions, vec!["1.0.0".to_string(), "1.02".to_string(), "1.30".to_string(), "2.19".to_string(), "2.19.100".to_string(), "3.0.1-SNAPSHOT".to_string(), "3.30".to_string(), "4.0.2".to_string(), "5.0-SNAPSHOT".to_string(), "5.0".to_string(), "5.0.1-SNAPSHOT".to_string()]);
    }
}