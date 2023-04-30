pub mod texture;
use std::path::{Path, PathBuf};

use crate::resource::texture::*;

#[derive(Debug)]
pub enum ResourceKind {
    Base,
    Texture(Texture),
}

#[derive(Debug)]
pub struct Resource {
    kind: ResourceKind,
    pub(crate) path: PathBuf,
}

impl Resource {
    pub fn new(path: &Path, kind: ResourceKind) -> Resource {
        Resource {
            kind,
            path: path.to_path_buf(),
        }
    }

    pub fn borrow_kind(&self) -> &ResourceKind {
        &self.kind
    }

    pub fn borrow_kind_mut(&mut self) -> &mut ResourceKind {
        &mut self.kind
    }
}
