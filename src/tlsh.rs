use crate::util::{bucket_counts, bucket_quartiles, checksum, l_capturing, q1_ratio, q2_ratio, pack_q1q2_ratio, swap_hex, header_distance, body_distance, unpack_q1q2_ratio, pack_bitpairs};

#[derive(Debug, PartialEq)]
pub(crate) struct Tlsh {
    // The TLSH hash has been updated to include a version prefix, typically the string "T1"
    version: String,

    // Header components:
    pub(crate) checksum: u8,
    pub(crate) log_len: u8,
    pub(crate) q1_ratio: u8,
    pub(crate) q2_ratio: u8,

    // Body components:
    pub(crate) body: [u8;64],
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
        // For a reason that is unclear to me, TLSH seems to swap the hex digits of only its checksum
        // and log_len header values.
        let checksum = swap_hex(self.checksum);
        let log_len = swap_hex(self.log_len);
        let q1q2_ratio = pack_q1q2_ratio(self.q1_ratio, self.q2_ratio);
        let digest_header = [checksum, log_len, q1q2_ratio];
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

    pub(crate) fn decode(digest: &str) -> Option<Self> {
        let header_bytes = 3;
        let body_bytes = 64;
        let version_len = 2;

        // The header and body are hex encoded, so each byte is encoded as 2 characters
        if digest.len() != (header_bytes + body_bytes) * 2 + version_len {
            return None;
        }

        fn decode_hex_byte(b: &[u8]) -> Option<u8> {
            match u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16) {
                Ok(x) => Some(x),
                _ => None,
            }
        }

        let version = &digest[..version_len];
        let digest_header_bytes: Vec<u8> = digest[version_len..version_len + header_bytes*2].as_bytes()
            .chunks_exact(2)
            .map(decode_hex_byte)
            .collect::<Option<Vec<u8>>>()?;
        // The TLSH C++ implementation also appears to list the bytes for the body in reverse order
        let digest_body_bytes = digest[version_len + header_bytes*2..].as_bytes()
            .chunks_exact(2)
            .rev()
            .map(decode_hex_byte)
            .collect::<Option<Vec<u8>>>()?;

        // For a reason that is unclear to me, TLSH seems to swap the hex digits of only its checksum
        // and log_len header values.
        let checksum = swap_hex(digest_header_bytes[0]);
        let log_len = swap_hex(digest_header_bytes[1]);
        let q1q2_ratio = digest_header_bytes[2];
        let (q1_ratio, q2_ratio) = unpack_q1q2_ratio(q1q2_ratio);

        let body: [u8; 64] = match (digest_body_bytes).try_into() {
            Ok(bytes) => Some(bytes),
            _ => None,
        }?;

        Some(Tlsh {
            version: version.to_string(),
            checksum,
            log_len,
            q1_ratio,
            q2_ratio,
            body,
        })
    }

    pub(crate) fn diff(&self, other: &Tlsh) -> i32 {
        let header_diff = header_distance(self, other);
        let body_diff = body_distance(self, other);
        header_diff + body_diff
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
        let checksum = checksum(data, window_size);
        let log_len = l_capturing(data.len());
        let q1_ratio = q1_ratio(q1, q3);
        let q2_ratio = q2_ratio(q2, q3);

        // Step 4: Construct the digest body
        // First we generate 2-bit values, then pack them into bytes
        let bucketbits = buckets.map(|b| {
            if b <= q1 { 0b00_u8 }
            else if b <= q2 { 0b01 }
            else if b <= q3 { 0b10 }
            else { 0b11 }
        });
        let body = bucketbits.chunks_exact(4).map(pack_bitpairs);

        Tlsh {
            version: "T1".to_string(),
            checksum,
            log_len,
            q1_ratio,
            q2_ratio,
            body: body.collect::<Vec<u8>>().try_into().unwrap()
        }
    }
}


#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn test_encode_decode(data in prop::collection::vec(0..u8::MAX, 0..10000)) {
            let hash = Tlsh::from_data(&data);
            let enc_dec = Tlsh::decode(&hash.encode()).unwrap();
            prop_assert_eq!(enc_dec, hash);
        }
    }
}
