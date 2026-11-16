//! Opt-in, test-only codec assurance helpers.
//!
//! These helpers generate bounded deterministic cases and check mechanical
//! canonicality. The consumer supplies every encoder, decoder and acceptance
//! decision and retains its schema, errors, bounds and compatibility policy.

/// Asserts that every strict prefix of `canonical` is rejected.
///
/// The callback returns `true` only when its input is accepted as one complete
/// consumer value. Neither errors nor frame content are formatted on failure.
pub fn assert_every_truncation_rejected(canonical: &[u8], mut accepts: impl FnMut(&[u8]) -> bool) {
    for length in 0..canonical.len() {
        assert!(
            !accepts(&canonical[..length]),
            "codec accepted strict prefix of length {length} from a {}-byte canonical frame",
            canonical.len(),
        );
    }
}

/// Asserts rejection after each caller-selected single trailing byte.
///
/// The selected suffixes are explicit because protocols differ on whether they
/// are testing one representative byte or all 256 byte values.
pub fn assert_trailing_bytes_rejected(
    canonical: &[u8],
    trailing_bytes: &[u8],
    mut accepts: impl FnMut(&[u8]) -> bool,
) {
    let mut candidate = Vec::new();
    candidate
        .try_reserve_exact(canonical.len().saturating_add(1))
        .expect("codec qualification candidate allocation failed");
    candidate.extend_from_slice(canonical);
    candidate.push(0);
    for trailing in trailing_bytes {
        let last = candidate
            .last_mut()
            .expect("qualification candidate always contains a trailing byte");
        *last = *trailing;
        assert!(
            !accepts(&candidate),
            "codec accepted a selected trailing byte after a {}-byte canonical frame",
            canonical.len(),
        );
    }
}

/// Visits at most `maximum` deterministic single-bit mutations.
///
/// Mutations are ordered by byte index and then least- to most-significant bit.
/// The visitor owns all assertions: a mutation may be either a required refusal
/// or another valid canonical value in the consumer's protocol.
pub fn for_each_single_bit_mutation(
    canonical: &[u8],
    maximum: usize,
    mut visit: impl FnMut(usize, u8, &[u8]),
) {
    let available = canonical.len().saturating_mul(8);
    let count = available.min(maximum);
    if count == 0 {
        return;
    }
    let mut candidate = canonical.to_vec();
    for mutation in 0..count {
        let byte_index = mutation / 8;
        let bit = (mutation % 8) as u8;
        candidate[byte_index] ^= 1_u8 << bit;
        visit(byte_index, bit, &candidate);
        candidate[byte_index] ^= 1_u8 << bit;
    }
}

/// Asserts exact golden encoding and decoding without requiring content-bearing
/// `Debug` implementations.
pub fn assert_golden_round_trip<T: PartialEq>(
    value: &T,
    golden: &[u8],
    mut encode: impl FnMut(&T) -> Option<Vec<u8>>,
    mut decode: impl FnMut(&[u8]) -> Option<T>,
) {
    let encoded = encode(value).expect("golden value did not encode");
    assert!(
        encoded == golden,
        "golden encoding differed (actual {} bytes, expected {} bytes)",
        encoded.len(),
        golden.len(),
    );
    let decoded = decode(golden).expect("golden bytes did not decode");
    assert!(
        decoded == *value,
        "golden bytes decoded to a different value"
    );
}

/// If a candidate decodes, asserts that re-encoding produces those exact bytes.
/// Rejected candidates satisfy the check without imposing an error vocabulary.
pub fn assert_canonical_if_accepted<T>(
    candidate: &[u8],
    mut decode: impl FnMut(&[u8]) -> Option<T>,
    mut encode: impl FnMut(&T) -> Option<Vec<u8>>,
) {
    let Some(value) = decode(candidate) else {
        return;
    };
    let encoded = encode(&value).expect("accepted codec value did not re-encode");
    assert!(
        encoded == candidate,
        "accepted {}-byte candidate was not canonical",
        candidate.len(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(value: &u16) -> Option<Vec<u8>> {
        Some(value.to_be_bytes().to_vec())
    }

    fn decode(bytes: &[u8]) -> Option<u16> {
        let bytes: [u8; 2] = bytes.try_into().ok()?;
        Some(u16::from_be_bytes(bytes))
    }

    #[test]
    fn truncation_and_selected_trailing_cases_are_exhaustive() {
        let frame = [0x12, 0x34];
        let mut truncations = 0;
        assert_every_truncation_rejected(&frame, |candidate| {
            truncations += 1;
            decode(candidate).is_some()
        });
        assert_eq!(truncations, frame.len());

        let mut trailing = 0;
        assert_trailing_bytes_rejected(&frame, &[0, 1, u8::MAX], |candidate| {
            trailing += 1;
            decode(candidate).is_some()
        });
        assert_eq!(trailing, 3);
    }

    #[test]
    fn bit_mutations_are_bounded_deterministic_and_restored_between_visits() {
        let frame = [0_u8, 0_u8];
        let mut observed = Vec::new();
        for_each_single_bit_mutation(&frame, 10, |byte, bit, candidate| {
            observed.push((byte, bit, candidate.to_vec()));
        });
        assert_eq!(observed.len(), 10);
        assert_eq!(observed[0], (0, 0, vec![1, 0]));
        assert_eq!(observed[8], (1, 0, vec![0, 1]));
        assert_eq!(observed[9], (1, 1, vec![0, 2]));
    }

    #[test]
    fn golden_and_accepted_canonical_checks_use_consumer_codecs() {
        assert_golden_round_trip(&0x1234, &[0x12, 0x34], encode, decode);
        assert_canonical_if_accepted(&[0x12, 0x34], decode, encode);
        assert_canonical_if_accepted(&[0x12], decode, encode);
    }
}
