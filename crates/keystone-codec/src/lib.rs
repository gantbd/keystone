//! Allocation-conscious bounded codec helpers.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecodeLimits {
    pub max_input_bytes: usize,
    pub max_field_bytes: usize,
    pub max_fields: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    InputLimit,
    FieldLimit,
    FieldCountLimit,
    Truncated,
    LengthOverflow,
}

pub struct Decoder<'a> {
    input: &'a [u8],
    position: usize,
    fields: usize,
    limits: DecodeLimits,
}

impl<'a> Decoder<'a> {
    pub fn new(input: &'a [u8], limits: DecodeLimits) -> Result<Self, DecodeError> {
        if input.len() > limits.max_input_bytes {
            return Err(DecodeError::InputLimit);
        }
        Ok(Self {
            input,
            position: 0,
            fields: 0,
            limits,
        })
    }

    pub fn is_finished(&self) -> bool {
        self.position == self.input.len()
    }

    pub fn read_u8(&mut self) -> Result<u8, DecodeError> {
        self.check_field()?;
        let value = *self
            .input
            .get(self.position)
            .ok_or(DecodeError::Truncated)?;
        self.position += 1;
        self.fields += 1;
        Ok(value)
    }

    pub fn read_u64_be(&mut self) -> Result<u64, DecodeError> {
        self.check_field()?;
        let end = self.checked_end(8)?;
        let bytes = self
            .input
            .get(self.position..end)
            .ok_or(DecodeError::Truncated)?;
        let value = u64::from_be_bytes(bytes.try_into().map_err(|_| DecodeError::Truncated)?);
        self.position = end;
        self.fields += 1;
        Ok(value)
    }

    pub fn read_bytes(&mut self) -> Result<&'a [u8], DecodeError> {
        self.check_field()?;
        let length_end = self.checked_end(4)?;
        let length_bytes = self
            .input
            .get(self.position..length_end)
            .ok_or(DecodeError::Truncated)?;
        let length = u32::from_be_bytes(
            length_bytes
                .try_into()
                .map_err(|_| DecodeError::Truncated)?,
        );
        let length = usize::try_from(length).map_err(|_| DecodeError::LengthOverflow)?;
        if length > self.limits.max_field_bytes {
            return Err(DecodeError::FieldLimit);
        }
        let end = length_end
            .checked_add(length)
            .ok_or(DecodeError::LengthOverflow)?;
        let value = self
            .input
            .get(length_end..end)
            .ok_or(DecodeError::Truncated)?;
        self.position = end;
        self.fields += 1;
        Ok(value)
    }

    fn check_field(&self) -> Result<(), DecodeError> {
        if self.fields >= self.limits.max_fields {
            return Err(DecodeError::FieldCountLimit);
        }
        Ok(())
    }

    fn checked_end(&self, length: usize) -> Result<usize, DecodeError> {
        self.position
            .checked_add(length)
            .ok_or(DecodeError::LengthOverflow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    LengthOverflow,
}

pub fn encode_bytes(output: &mut Vec<u8>, value: &[u8]) -> Result<(), EncodeError> {
    let length = u32::try_from(value.len()).map_err(|_| EncodeError::LengthOverflow)?;
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> DecodeLimits {
        DecodeLimits {
            max_input_bytes: 16,
            max_field_bytes: 4,
            max_fields: 2,
        }
    }

    #[test]
    fn bounded_length_prefix_round_trips() {
        let mut encoded = Vec::new();
        encode_bytes(&mut encoded, b"data").unwrap();
        let mut decoder = Decoder::new(&encoded, limits()).unwrap();
        assert_eq!(decoder.read_bytes(), Ok(b"data".as_slice()));
        assert!(decoder.is_finished());
    }

    #[test]
    fn oversized_field_refuses_before_consuming_payload() {
        let input = [0, 0, 0, 5, 1, 2, 3, 4, 5];
        let mut decoder = Decoder::new(&input, limits()).unwrap();
        assert_eq!(decoder.read_bytes(), Err(DecodeError::FieldLimit));
    }
}
