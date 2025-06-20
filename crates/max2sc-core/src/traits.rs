//! Conversion traits for max2sc

/// Trait for converting objects to SuperCollider representation
pub trait ToSuperCollider {
    type Output;
    type Error;

    fn to_supercollider(&self) -> Result<Self::Output, Self::Error>;
}

/// Trait for creating objects from Max representations
pub trait FromMax<T> {
    type Error;

    fn from_max(max_obj: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}
