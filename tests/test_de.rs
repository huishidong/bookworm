// Suppose we have JSON that looks like [[1, 2], [3, 4, 5], [6]] and we need to deserialize it into a flat representation like
//  vec![1, 2, 3, 4, 5, 6]. Allocating a brand new Vec<T> for each subarray would be slow. Instead we would like to allocate a
// single Vec<T> and then deserialize each subarray into it. This requires stateful deserialization using the DeserializeSeed trait.

use serde::de::{Deserialize, DeserializeSeed, Deserializer, SeqAccess, Visitor};
use std::fmt;
use std::marker::PhantomData;

// A DeserializeSeed implementation that uses stateful deserialization to
// append array elements onto the end of an existing vector. The preexisting
// state ("seed") in this case is the Vec<T>. The `deserialize` method of
// `ExtendVec` will be traversing the inner arrays of the JSON input and
// appending each integer into the existing Vec.
struct ExtendVec<'a, T: 'a>(&'a mut Vec<T>);

impl<'de, T> DeserializeSeed<'de> for ExtendVec<'_, T>
where
    T: Deserialize<'de>,
{
    // The return type of the `deserialize` method. This implementation
    // appends onto an existing vector but does not create any new data
    // structure, so the return type is ().
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Visitor implementation that will walk an inner array of the JSON
        // input.
        struct ExtendVecVisitor<'a, T: 'a>(&'a mut Vec<T>);

        impl<'de, T> Visitor<'de> for ExtendVecVisitor<'_, T>
        where
            T: Deserialize<'de>,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "an array of integers")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<(), A::Error>
            where
                A: SeqAccess<'de>,
            {
                // Decrease the number of reallocations if there are many elements
                if let Some(size_hint) = seq.size_hint() {
                    self.0.reserve(size_hint);
                }

                // Visit each element in the inner array and push it onto
                // the existing vector.
                while let Some(elem) = seq.next_element()? {
                    self.0.push(elem);
                }
                Ok(())
            }
        }

        deserializer.deserialize_seq(ExtendVecVisitor(self.0))
    }
}

// Visitor implementation that will walk the outer array of the JSON input.
struct FlattenedVecVisitor<T>(PhantomData<T>);

impl<'de, T> Visitor<'de> for FlattenedVecVisitor<T>
where
    T: Deserialize<'de>,
{
    // This Visitor constructs a single Vec<T> to hold the flattened
    // contents of the inner arrays.
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "an array of arrays")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Vec<T>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        // Create a single Vec to hold the flattened contents.
        let mut vec = Vec::new();

        // Each iteration through this loop is one inner array.
        while let Some(()) = seq.next_element_seed(ExtendVec(&mut vec))? {
            // Nothing to do; inner array has been appended into `vec`.
        }

        // Return the finished vec.
        Ok(vec)
    }
}

#[test]
fn test_flattened_vec() -> Result<(), Box<dyn std::error::Error>> {
    // Example JSON input
    let json_input = r#"
        [
            [1, 2],
            [3, 4, 5],
            [6]
        ]
    "#;

    // Deserialize the JSON input into a flattened Vec<u64>
    let mut deserializer = serde_json::Deserializer::from_str(json_input);
    let visitor = FlattenedVecVisitor(PhantomData);
    let flattened: Vec<u64> = deserializer.deserialize_seq(visitor)?;
    assert!(flattened == vec![1, 2, 3, 4, 5, 6]);
    Ok(())
}
