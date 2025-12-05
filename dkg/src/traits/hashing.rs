pub trait SecureHashing {
    fn message(&self) -> impl AsRef<[u8]>;

    fn hash(&self) -> impl AsRef<[u8]>;
}
