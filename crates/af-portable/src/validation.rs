mod content;
mod refs;

pub(crate) use content::validate_d3;
pub(crate) use refs::validate_d2;

use crate::codec::CanonicalPortableBundle;

#[derive(Debug, Clone)]
pub struct ReferenceValidatedPortableBundle {
    pub(crate) canonical: CanonicalPortableBundle,
}

#[derive(Debug, Clone)]
pub struct ContentValidatedPortableBundle {
    pub(crate) referenced: ReferenceValidatedPortableBundle,
}

impl ContentValidatedPortableBundle {
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        self.referenced.canonical.canonical_bytes()
    }

    /// Immutable D0-D3 raw typed input, not a Checked capability.
    #[must_use]
    pub fn value(&self) -> &af_core::Value {
        self.referenced.canonical.value()
    }
}
