#[derive(Debug, Deserialize)]
pub struct DestroyItem {
    destroy : bool
}

impl Verify for DestroyItem {
    fn default() -> Self {
        DestroyItem {
            destroy: false
        }
    }
}