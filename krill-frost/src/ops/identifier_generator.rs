use std::marker::PhantomData;

use frost_core::Ciphersuite;

use crate::{
    KrillError, RandomBytes,
};

pub struct IdentifierGenerator<C: Ciphersuite>(PhantomData<C>);

impl<C> IdentifierGenerator<C>
where
   <<<C as frost_core::Ciphersuite>::Group as frost_core::Group>::Field as frost_core::Field>::Scalar: std::convert::From<u128>,
   C:Ciphersuite
{
    pub fn hashed_identifier(
        identifier: impl AsRef<[u8]>,
    ) -> Result<frost_core::Identifier<C>, KrillError> {
        let identifier_bytes = *blake3::hash(identifier.as_ref()).as_bytes();

        let scalar_data = u128::from_le_bytes(identifier_bytes[0..16].try_into().or(Err(
            KrillError::ToByteArray("Unable to cast the slice tto a [0u8;16] byte array"),
        ))?);

        frost_core::Identifier::<C>::new(scalar_data.into())
            .or(Err(KrillError::IdentifierDerivationNotSupported))
    }

    pub fn random_identifier() -> Result<frost_core::Identifier<C>, KrillError> {
        let identifier = RandomBytes::<32>::generate();
        frost_core::Identifier::<C>::derive(&*identifier.take())
            .or(Err(KrillError::IdentifierDerivationNotSupported))
    }
}
