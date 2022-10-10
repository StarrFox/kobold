use anyhow::bail;

use super::{BitReader, TypeDef, TypeList};

/// A type tag that defines deserialization behavior to
/// identify object types.
pub trait TypeTag: Sized {
    /// Reads the object identity from the deserializer
    /// and returns a matching type def.
    fn object_identity(reader: &mut BitReader, types: &TypeList)
        -> anyhow::Result<Option<TypeDef>>;
}

/// A [`TypeTag`] that identifies regular PropertyClasses.
pub struct PropertyClass;

impl TypeTag for PropertyClass {
    fn object_identity(
        reader: &mut BitReader,
        types: &TypeList,
    ) -> anyhow::Result<Option<TypeDef>> {
        let hash = reader.load_u32()?;
        if hash == 0 {
            Ok(None)
        } else if let Some(t) = types.list.get(&hash) {
            Ok(Some(t.clone()))
        } else {
            bail!("Failed to identify type with hash {hash}");
        }
    }
}