use core::{fmt, mem::MaybeUninit};

use crate::ArrayPlusExtra;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, SeqAccess, Visitor},
};

impl<T, const N: usize, const EXTRA: usize> Serialize for ArrayPlusExtra<T, N, EXTRA>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.as_slice().serialize(serializer)
    }
}

impl<'de, T, const N: usize, const EXTRA: usize> Deserialize<'de> for ArrayPlusExtra<T, N, EXTRA>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ArrayVisitor(core::marker::PhantomData))
    }
}

struct ArrayVisitor<T, const N: usize, const EXTRA: usize>(core::marker::PhantomData<T>);

impl<'de, T, const N: usize, const EXTRA: usize> Visitor<'de> for ArrayVisitor<T, N, EXTRA>
where
    T: Deserialize<'de>,
{
    type Value = ArrayPlusExtra<T, N, EXTRA>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an array of {} elements", N + EXTRA)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Create our type with uninitialized data.
        let mut data: [MaybeUninit<T>; N] = [const { MaybeUninit::uninit() }; N];
        let mut extra: [MaybeUninit<T>; EXTRA] = [const { MaybeUninit::uninit() }; EXTRA];

        for (i, elem) in data.iter_mut().chain(extra.iter_mut()).enumerate() {
            *elem = MaybeUninit::new(
                seq.next_element()?
                    .ok_or_else(|| Error::invalid_length(i, &self))?,
            );
        }

        // SAFETY: All elements are initialized.
        // We use ptr::read to convert [MaybeUninit<T>; N] to [T; N].
        // This works because MaybeUninit<T> has the same layout as T.
        // We would like to us `MaybeUninit::array_assume_init`, but this is still unstable.
        Ok(unsafe {
            ArrayPlusExtra {
                data: (&data as *const _ as *const [T; N]).read(),
                extra: (&extra as *const _ as *const [T; EXTRA]).read(),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_basic() {
        let arr: ArrayPlusExtra<i32, 2, 1> = ArrayPlusExtra::new(42);
        let json = serde_json::to_string(&arr).unwrap();

        // Should serialize as an array of 3 elements.
        assert_eq!(json, "[42,42,42]");

        let deserialized: ArrayPlusExtra<i32, 2, 1> = serde_json::from_str(&json).unwrap();
        assert_eq!(arr, deserialized);
    }

    #[test]
    fn test_serialize_mixed_values() {
        let mut arr: ArrayPlusExtra<i32, 3, 2> = ArrayPlusExtra::new(0);
        arr[0] = 10;
        arr[1] = 20;
        arr[2] = 30;
        arr[3] = 40;
        arr[4] = 50;

        let json = serde_json::to_string(&arr).unwrap();
        assert_eq!(json, "[10,20,30,40,50]");

        let deserialized: ArrayPlusExtra<i32, 3, 2> = serde_json::from_str(&json).unwrap();
        assert_eq!(arr, deserialized);
    }

    #[test]
    fn test_deserialize_from_json() {
        let json = "[1,2,3,4]";
        let arr: ArrayPlusExtra<i32, 2, 2> = serde_json::from_str(json).unwrap();

        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0], 1);
        assert_eq!(arr[1], 2);
        assert_eq!(arr[2], 3);
        assert_eq!(arr[3], 4);
    }

    #[test]
    fn test_deserialize_wrong_length_fails() {
        // Too few elements.
        let json = "[1,2]";
        let result: Result<ArrayPlusExtra<i32, 2, 2>, _> = serde_json::from_str(json);
        assert!(result.is_err());

        // Too many elements.
        let json = "[1,2,3,4,5]";
        let result: Result<ArrayPlusExtra<i32, 2, 2>, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_zero_sized() {
        let arr: ArrayPlusExtra<i32, 0, 0> = ArrayPlusExtra::new(42);
        let json = serde_json::to_string(&arr).unwrap();
        assert_eq!(json, "[]");

        let deserialized: ArrayPlusExtra<i32, 0, 0> = serde_json::from_str(&json).unwrap();
        assert_eq!(arr.len(), deserialized.len());
    }

    #[test]
    fn test_serialize_extra_zero() {
        let mut arr: ArrayPlusExtra<i32, 3, 0> = ArrayPlusExtra::new(0);
        arr[0] = 1;
        arr[1] = 2;
        arr[2] = 3;

        let json = serde_json::to_string(&arr).unwrap();
        assert_eq!(json, "[1,2,3]");
    }

    #[test]
    fn test_serialize_n_zero() {
        let mut arr: ArrayPlusExtra<i32, 0, 3> = ArrayPlusExtra::new(0);
        arr[0] = 7;
        arr[1] = 8;
        arr[2] = 9;

        let json = serde_json::to_string(&arr).unwrap();
        assert_eq!(json, "[7,8,9]");
    }

    #[test]
    fn test_roundtrip_with_different_types() {
        // Test with u8.
        let arr: ArrayPlusExtra<u8, 2, 2> = ArrayPlusExtra::new(255);
        let json = serde_json::to_string(&arr).unwrap();
        let deserialized: ArrayPlusExtra<u8, 2, 2> = serde_json::from_str(&json).unwrap();
        assert_eq!(arr, deserialized);

        // Test with f64.
        let arr: ArrayPlusExtra<f64, 1, 2> = ArrayPlusExtra::new(1.5);
        let json = serde_json::to_string(&arr).unwrap();
        let deserialized: ArrayPlusExtra<f64, 1, 2> = serde_json::from_str(&json).unwrap();
        assert_eq!(arr, deserialized);
    }
}
