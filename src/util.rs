use std::ops::{BitAnd, BitXor};

// Pearson's sample random table (used for TLSH implementation)
const V_TABLE: [u8;256] = [
    1, 87, 49, 12, 176, 178, 102, 166, 121, 193, 6, 84, 249, 230, 44, 163,
    14, 197, 213, 181, 161, 85, 218, 80, 64, 239, 24, 226, 236, 142, 38, 200,
    110, 177, 104, 103, 141, 253, 255, 50, 77, 101, 81, 18, 45, 96, 31, 222,
    25, 107, 190, 70, 86, 237, 240, 34, 72, 242, 20, 214, 244, 227, 149, 235,
    97, 234, 57, 22, 60, 250, 82, 175, 208, 5, 127, 199, 111, 62, 135, 248,
    174, 169, 211, 58, 66, 154, 106, 195, 245, 171, 17, 187, 182, 179, 0, 243,
    132, 56, 148, 75, 128, 133, 158, 100, 130, 126, 91, 13, 153, 246, 216, 219,
    119, 68, 223, 78, 83, 88, 201, 99, 122, 11, 92, 32, 136, 114, 52, 10,
    138, 30, 48, 183, 156, 35, 61, 26, 143, 74, 251, 94, 129, 162, 63, 152,
    170, 7, 115, 167, 241, 206, 3, 150, 55, 59, 151, 220, 90, 53, 23, 131,
    125, 173, 15, 238, 79, 95, 89, 16, 105, 137, 225, 224, 217, 160, 37, 123,
    118, 73, 2, 157, 46, 116, 9, 145, 134, 228, 207, 212, 202, 215, 69, 229,
    27, 188, 67, 124, 168, 252, 42, 4, 29, 108, 21, 247, 19, 205, 39, 203,
    233, 40, 186, 147, 198, 192, 155, 33, 164, 191, 98, 204, 165, 180, 117, 76,
    140, 36, 210, 172, 41, 54, 159, 8, 185, 232, 113, 196, 231, 47, 146, 120,
    51, 65, 28, 144, 254, 221, 93, 189, 194, 139, 112, 43, 71, 109, 184, 209
];

// Pearson's algorithm
pub(crate) fn bucket_mapping(salt: u8, i: u8, j: u8, k: u8) -> u8 {
    let mut h = 0;

    h = V_TABLE[(h ^ salt) as usize];
    h = V_TABLE[(h ^ i) as usize];
    h = V_TABLE[(h ^ j) as usize];
    h = V_TABLE[(h ^ k) as usize];
    h
}

// Note: each bucket is defined to hold an unsigned int in the TLSH C++ implementation
pub(crate) fn bucket_counts(data: &[u8], window_size: usize) -> [u32; 256] {
    let mut buckets: [u32; 256] = [0; 256];

    // The TLSH C++ implementation looks to use the below salts for each window in order
    const SALTS: [u8; 21] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73];
    // const SALTS: [u8; 21] = [49, 12, 178, 166, 84, 230, 197, 181, 80, 142, 200, 253, 101, 18, 222, 237, 214, 227, 22, 175, 5];

    if data.len() < window_size {
        // Too little data, ignore
        return buckets
    }

    for start in 0..data.len()-window_size+1 {
        let window = &data[start..start+window_size];
        for (i, (c1, c2, c3)) in sliding_triplets(window).iter().enumerate() {
            let b_i = bucket_mapping(SALTS[i], *c1, *c2, *c3) as usize;
            buckets[b_i] += 1;
        }
    }

    buckets
}

// Unique combinations of bytes (n-1 choose 2 combinations where n = window size) for a sliding
// window. The authors point out that if the window size is 5, this results in 6 triplets. That is,
// given a window [A, B, C, D, E], this function returns all the possible combinations where E is
// always picked (EDC, EDB, ..., EBA).
pub(crate) fn sliding_triplets(window: &[u8]) -> Vec<(u8, u8, u8)> {
    // n-1 choose 2 is ((n-1) * (n-2)) / (2*1), which is approximately (n-1)^2 / 2
    let mut triplets = Vec::with_capacity((window.len()-1).pow(2) / 2);

    // We always only pick the new element, since combinations starting with previous elements
    // have been picked in previous windows.
    //
    // Note: the TLSH C++ implementation uses a weird strategy where usually the triplets are
    // ordered in descending order primarily by the last element then secondarily by the middle
    // element (e.g. i,i-1,i-2 -> i,i-1,i-3 -> i,i-2,1-3 -> ...) except for the case when the last
    // element is the element 4 spots prior to the first element (i). In that case the weird
    // ordering is: i,i-2,i-4 -> i,i-1,i-4 -> i,i-3,i-4.
    let i = window.len()-1;
    for k in (0..i-1).rev() {
        let j_range =
            if k == i-4 {
                vec![i-2, i-1, i-3]
            } else {
                (k+1..i).rev().collect()
            };
        for j in j_range {
            triplets.push((window[i], window[j], window[k]));
        }
    }

    triplets
}

pub(crate) fn checksum(data: &[u8], window_size: usize) -> u8 {
    let mut _checksum = 0;

    // It looks like the checksum is calculated starting from the window'th size element in the
    // data. The checksum is calculated by repeatedly computing the bucket mapping algorithm using
    // the current and previous elements along with the checksum so far.
    for (curr, prev) in data.iter().skip(window_size-1).zip(data.iter().skip(window_size-2)) {
        _checksum = bucket_mapping(0, *curr, *prev, _checksum);
    }
    _checksum
}

// For a reason that is unclear to me, the TLSH hash swaps only some of its hex digits. This
// ensures that the value 1 is encoded in hex as "10" rather than "01".
pub(crate) fn swap_hex(byte: u8) -> u8 {
    byte.checked_shl(4).unwrap().bitxor(
        byte.checked_shr(4).unwrap()
    )
}

pub(crate) fn bucket_quartiles(buckets: &[u32;256]) -> (u32, u32, u32) {
    let mut _buckets = *buckets;
    _buckets.sort();

    // The quartiles are located 25% through the sorted array, 50% through the sorted array, and
    // 75% through the sorted array.
    //
    // Note: 64th bucket is at index 63, the 128th bucket is at index 127, etc.
    (_buckets[63], _buckets[127], _buckets[191])
}

pub(crate) fn l_capturing(data_len: usize) -> u8 {
    // C++ reference implementation:
    // #define LOG_1_5 0.4054651
    // #define LOG_1_3 0.26236426
    // #define LOG_1_1 0.095310180
    //
    // unsigned char l_capturing(unsigned int len) {
    //     int i;
    //     if( len <= 656 ) {
    //         i = (int) floor( std::log((float) len) / LOG_1_5 );
    //     } else if( len <= 3199 ) {
    //         i = (int) floor( std::log((float) len) / LOG_1_3 - 8.72777 );
    //     } else {
    //         i = (int) floor( std::log((float) len) / LOG_1_1 - 62.5472 );
    //     }
    //
    //     return (unsigned char) (i & 0xFF);
    // }

    // Note: the TLSH implementation uses the following log constants when calculating its log
    // length value.
    const LOG_1_5: f32 = 0.4054651;      // ln(1.5)
    const LOG_1_3: f32 = 0.26236426;     // ln(1.3)
    const LOG_1_1: f32 = 0.095310180;    // ln(1.1)

    let log_len: f32 = (data_len as f32).ln();

    // The TLSH implementation converts the floored float to an integer and then returns its
    // smallest byte
    // TODO(klinvill): why were these constants chosen?
    let int_result: i32 = if data_len <= 656 {
        (log_len / LOG_1_5).floor() as i32
    } else if data_len <= 3199 {
        (log_len / LOG_1_3 - 8.72777).floor() as i32
    } else {
        (log_len / LOG_1_1 - 62.5472).floor() as i32
    };

    // Return smallest byte, this is equivalent to taking the bytes mod 256
    int_result.to_le_bytes()[0]
}

pub(crate) fn q1q2_ratio(q1: u32, q2: u32, q3: u32) -> u8 {
    // The C++ TLSH implementation casts everything to float for this operation.
    let q1_ratio: u8 = (((q1 * 100) as f32 / q3 as f32) % 16.0) as u8;
    let q2_ratio: u8 = (((q2 * 100) as f32 / q3 as f32) % 16.0) as u8;

    // actual header byte uses the first four bits from q1 and the next four bits from q2
    q1_ratio.checked_shl(4).unwrap()
        .bitxor(
            q2_ratio.bitand(0xf)
        )
}


#[cfg(test)]
mod tests {
    use rand::prelude::SliceRandom;
    use super::*;

    #[test]
    fn test_sliding_triplets() {
        assert_eq!(
            sliding_triplets(&[1, 2, 3, 4, 5]),
            vec![
                (5, 4, 3),
                (5, 4, 2),
                (5, 3, 2),

                (5, 3, 1),
                (5, 4, 1),
                (5, 2, 1),
            ]
        );

        assert_eq!(
            sliding_triplets(&[1, 2, 3, 4, 5, 6, 7]),
            vec![
                (7, 6, 5),
                (7, 6, 4),
                (7, 5, 4),

                (7, 5, 3),
                (7, 6, 3),
                (7, 4, 3),

                (7,6,2),
                (7,5,2),
                (7,4,2),
                (7,3,2),

                (7,6,1),
                (7,5,1),
                (7,4,1),
                (7,3,1),
                (7,2,1),
            ]
        );
    }

    #[test]
    fn test_swap_hex() {
        assert_eq!(swap_hex(0), 0);
        assert_eq!(swap_hex(0xFF), 0xFF);
        assert_eq!(swap_hex(0x01), 0x10);
        assert_eq!(swap_hex(0x2A), 0xA2);
    }

    #[test]
    fn test_bucket_quartiles() {
        let mut test_sequence: Vec<u32> = Vec::with_capacity(256);
        for i in 0..256 {
            test_sequence.push(i*3);
        }

        let expected = (test_sequence[63], test_sequence[127], test_sequence[191]);

        let mut rng = rand::thread_rng();
        test_sequence.shuffle(&mut rng);
        assert_eq!(
            bucket_quartiles(&test_sequence.try_into().unwrap()),
            expected
        );
    }

    #[test]
    fn test_l_capturing() {
        // Zero should always return zero
        assert_eq!(l_capturing(0), 0);

        // Values within the ranges
        assert_eq!(l_capturing(58), 10);
        assert_eq!(l_capturing(1880), 20);
        assert_eq!(l_capturing(4210), 25);

        // Test at boundaries
        assert_eq!(l_capturing(656), 15);
        assert_eq!(l_capturing(657), 16);
        assert_eq!(l_capturing(3199), 22);
        assert_eq!(l_capturing(3200), 22);

        // Known input to cause different rounding errors if using double precision instead of float
        // precision. Reported in https://github.com/trendmicro/tlsh/issues/89.
        assert_eq!(l_capturing(190336), 65);


    }

    #[test]
    fn test_q1q2_ratio() {
        assert_eq!(q1q2_ratio(16, 16, 1), 0);
        assert_eq!(q1q2_ratio(0, 0, 1), 0);

        // Expected q1 ratio is 13, expected q2 ratio is 15. The result is them packed into a byte.
        assert_eq!(q1q2_ratio(5, 7, 4), 13u8.checked_shl(4).unwrap().bitxor(15));
    }
}
