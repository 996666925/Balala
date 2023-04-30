use std::path::*;

use bytemuck::{Pod, Zeroable};
use glow::NativeTexture;

#[derive(Debug)]
pub struct Texture {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) gpu_tex: Option<NativeTexture>,
    pub(crate) need_upload: bool,
    pub(crate) pixels: Vec<u8>,
}

impl Texture {
    pub fn load(path: &Path) -> Result<Texture, image::ImageError> {
        let image = match image::open(path)? {
            image::DynamicImage::ImageRgba8(img) => img,
            other => other.into_rgba8(),
        };
        let width = image.width();
        let height = image.height();
        let pixels = image.into_raw();

        Ok(Texture {
            pixels,
            need_upload: true,
            width,
            height,
            gpu_tex: None,
        })
    }
}
