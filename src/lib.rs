use std::ops::BitXor;
use crate::util::{bucket_counts, bucket_quartiles, checksum, l_capturing, q1q2_ratio, swap_hex};

mod util;



pub fn hash(data: &[u8]) -> Option<String> {
    // We use the same window size as in the paper (5). The authors state this is the same size as
    // the window in the Nilsimsa hash.
    let window_size = 5;
    hash_impl(data, window_size)
}

fn hash_impl(data: &[u8], window_size: usize) -> Option<String> {
    // Note: each bucket is defined to hold an unsigned int in the TLSH C++ implementation
    let buckets = bucket_counts(data, window_size);

    // Step 2: Calculate bucket count quartiles
    let (q1, q2, q3) = bucket_quartiles(&buckets);

    // Step 3: Construct the digest header
    // For a reason that is unclear to me, TLSH seems to swap the hex digits of only its checksum
    // and log_len header values.
    let checksum = swap_hex(checksum(data, window_size));
    let log_len = swap_hex(l_capturing(data.len()));
    let quartile_header = q1q2_ratio(q1, q2, q3);
    let header = [checksum, log_len, quartile_header];

    // Step 4: Construct the digest body
    // First we generate 2-bit values, then pack them into bytes
    let bucketbits = buckets.map(|b| {
        if b <= q1 { 0b00 as u8 }
        else if b <= q2 { 0b01 }
        else if b <= q3 { 0b10 }
        else { 0b11 }
    });
    let body = bucketbits.chunks_exact(4).map(|bitpairs| {
        // The TLSH C++ implementation appears to pack 4 buckets into a byte in reverse order, so
        // the buckets b1, b2, b3, b4 would be packed into a byte as: [b4, b3, b2, b1]. E.g. if
        // b1=00, b2=01, b3=10, b4=11, the resulting byte would be 11100100.
        bitpairs[0].bitxor(
        bitpairs[1].checked_shl(2).unwrap().bitxor(
        bitpairs[2].checked_shl(4).unwrap().bitxor(
        bitpairs[3].checked_shl(6).unwrap())))
    });

    let mut hex: Vec<String> = header.iter().copied()
        // The TLSH C++ implementation also appears to list the bytes for the body in reverse order
        .chain(body.rev())
        .map(|byte| {
            format!("{:02X}", byte)
        }).collect();

    // The TLSH hash has been updated to include a version prefix, in this case: T1.
    hex.insert(0, String::from("T1"));

    Some(hex.join(""))
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
