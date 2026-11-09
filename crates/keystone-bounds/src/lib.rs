//! Bounded owned values and checked counters.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LimitExceeded {
    pub limit: usize,
    pub actual: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedBytes {
    bytes: Vec<u8>,
    limit: usize,
}

impl BoundedBytes {
    pub fn try_new(bytes: Vec<u8>, limit: usize) -> Result<Self, LimitExceeded> {
        if bytes.len() > limit {
            return Err(LimitExceeded {
                limit,
                actual: bytes.len(),
            });
        }
        Ok(Self { bytes, limit })
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedString {
    value: String,
    limit: usize,
}

impl BoundedString {
    pub fn try_new(value: String, limit: usize) -> Result<Self, LimitExceeded> {
        if value.len() > limit {
            return Err(LimitExceeded {
                limit,
                actual: value.len(),
            });
        }
        Ok(Self { value, limit })
    }

    pub fn as_str(&self) -> &str {
        &self.value
    }

    pub fn len(&self) -> usize {
        self.value.len()
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundedVec<T> {
    values: Vec<T>,
    limit: usize,
}

impl<T> BoundedVec<T> {
    pub fn try_new(values: Vec<T>, limit: usize) -> Result<Self, LimitExceeded> {
        if values.len() > limit {
            return Err(LimitExceeded {
                limit,
                actual: values.len(),
            });
        }
        Ok(Self { values, limit })
    }

    pub fn try_push(&mut self, value: T) -> Result<(), LimitExceeded> {
        if self.values.len() >= self.limit {
            return Err(LimitExceeded {
                limit: self.limit,
                actual: self.values.len().saturating_add(1),
            });
        }
        self.values.push(value);
        Ok(())
    }

    pub fn as_slice(&self) -> &[T] {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn limit(&self) -> usize {
        self.limit
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CounterExhausted;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CheckedCounter(u64);

impl CheckedCounter {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn current(self) -> u64 {
        self.0
    }

    pub fn take_next(&mut self) -> Result<u64, CounterExhausted> {
        let current = self.0;
        let next = current.checked_add(1).ok_or(CounterExhausted)?;
        self.0 = next;
        Ok(current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failed_push_preserves_the_bounded_vector() {
        let mut values = BoundedVec::try_new(vec![1], 1).unwrap();
        assert_eq!(
            values.try_push(2),
            Err(LimitExceeded {
                limit: 1,
                actual: 2
            })
        );
        assert_eq!(values.as_slice(), &[1]);
    }

    #[test]
    fn exhausted_counter_does_not_wrap() {
        let mut counter = CheckedCounter::new(u64::MAX);
        assert_eq!(counter.take_next(), Err(CounterExhausted));
        assert_eq!(counter.current(), u64::MAX);
    }
}
