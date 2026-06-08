# numutil
`numutil` is a small utility crate for making numeric type conversions easier in generic contexts. Implements the following:
- `T` <---> `[u8; size_of::<T>()]` for the core types that implement to/from ne/le/be bytes methods. Exposed with trait `ByteConversion`.
- `Vec<T>` <---> `Vec<u8>` For the types implementing `ByteConversion`. Exposed with trait `VecByteConversion`.
- `T` as `U` and `U` as `T` for any pairing of `T` and `U` that you could normally do this with in concretely typed contexts. Exposed as `LossyCast<U>::_as(self) -> U` and `LossyCast<U>::_from(U) -> Self` 