use std::ops::BitXor;
use crate::util::{bucket_counts, bucket_quartiles, checksum, l_capturing, q1q2_ratio, swap_hex};

pub(crate) struct Tlsh {
    // The TLSH hash has been updated to include a version prefix, typically the string "T1"
    version: String,

    // Header components:
    checksum: u8,
    log_len: u8,
    q1q2_ratio: u8,

    // Body components:
    body: [u8;64],
}

impl Tlsh {
    /// Builds the TLSH hash for a collection of bytes
    pub(crate) fn from_data(data: &[u8]) -> Self {
        // We use the same window size as in the paper (5). The authors state this is the same size
        // as the window in the Nilsimsa hash.
        let window_size = 5;
        Tlsh::tlsh_impl(data, window_size)
    }

    /// Converts a TLSH hash to its string representation
    pub(crate) fn encode(&self) -> String {
        let digest_header = [self.checksum, self.log_len, self.q1q2_ratio];
        let mut hex: Vec<String> = digest_header.iter()
            // The TLSH C++ implementation also appears to list the bytes for the body in reverse order
            .chain(self.body.iter().rev())
            .map(|byte| {
                format!("{:02X}", byte)
            }).collect();

        // The TLSH hash has been updated to include a version prefix
        hex.insert(0, self.version.clone());

        hex.join("")
    }

    fn tlsh_impl(data: &[u8], window_size: usize) -> Self {
        // Note: each bucket is defined to hold an unsigned int in the TLSH C++ implementation
        let buckets = bucket_counts(data, window_size);

        // Step 2: Calculate bucket count quartiles
        // Note(klinvill): the original TLSH hash paper only uses the first half of the buckets when
        // computing the quartiles and the digest body. This seems like a bad idea to me since
        // an adversary could craft a file that has a very similar signature in the lower 128 buckets
        // to a benign file, while putting malicious/deviant code that mostly differs in the upper 128
        // buckets. As a result, I've opted to instead use the 256 bucket variant that uses all 256
        // buckets.
        let (q1, q2, q3) = bucket_quartiles(&buckets);

        // Step 3: Construct the digest header
        // For a reason that is unclear to me, TLSH seems to swap the hex digits of only its checksum
        // and log_len header values.
        let checksum = swap_hex(checksum(data, window_size));
        let log_len = swap_hex(l_capturing(data.len()));
        let quartile_header = q1q2_ratio(q1, q2, q3);

        // Step 4: Construct the digest body
        // First we generate 2-bit values, then pack them into bytes
        let bucketbits = buckets.map(|b| {
            if b <= q1 { 0b00_u8 }
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

        Tlsh {
            version: "T1".to_string(),
            checksum,
            log_len,
            q1q2_ratio: quartile_header,
            body: body.collect::<Vec<u8>>().try_into().unwrap()
        }
    }
}
