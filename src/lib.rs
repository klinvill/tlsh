mod util;
mod tlsh;


/// Computes the TLSH hash for a collection of bytes and returns its string representation.
pub fn hash(data: &[u8]) -> Option<String> {
    let hash_struct = tlsh::Tlsh::from_data(data);
    Some(hash_struct.encode())
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn test_hash() {
        let test_file = Path::new("test/data/0Alice.txt");
        // 0Alice.txt reference hash computed using version 4.11.2
        let expected = "T145D1A40CE601EFD21E62648F2A9554F0E199E9B01B84213B6BE0DB5E2DA71FA898DFEB07A78123B35A030227671FA2C2F725402973629B25545EB43C3312679477F3FC";

        let data = std::fs::read(test_file).unwrap();
        let result = hash(&data);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_hash_2() {
        let test_file = Path::new("test/data/test.txt");
        // test.txt reference hash computed using version 4.11.2
        let expected = "T18190022601550B51D51586E656492090540884001958151D15E25D890844BA2540232D0944C621A1804A111A1702704C475AD5AC213504F2805C3887322F14C11B4DC1";

        let data = std::fs::read(test_file).unwrap();
        let result = hash(&data);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), expected);
    }
}
