// make `std` available when testing
#![cfg_attr(not(test), no_std)]

// Derive the Format trait for defmt if the defmt feature is enabled
#[cfg_attr(not(test), derive(defmt::Format))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SharedType {
    id: u8,
    array: [u8; 4],
}

impl SharedType {
    pub fn new(id: u8) -> Self {
        Self {
            id,
            array: [0, 1, 2, 3],
        }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn array(&self) -> &[u8; 4] {
        &self.array
    }
}

pub fn transform_shared_type(shared: &mut SharedType) {
    shared.id += 1;
    for i in 0..4 {
        shared.array[i] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_type() {
        let shared = SharedType::new(1);
        assert_eq!(shared.id(), 1);
        assert_eq!(shared.array(), &[0, 1, 2, 3]);
    }

    #[test]
    fn test_transform_shared_type() {
        let mut shared = SharedType::new(1);
        transform_shared_type(&mut shared);
        assert_eq!(shared.id(), 2);
        assert_eq!(shared.array(), &[0, 1, 2, 3]);
    }
}
