//! Icon loading (embedded app icon).

use std::sync::Arc;

use egui::IconData;

fn png_bytes() -> &'static [u8] {
    include_bytes!("../../../../assets/icons/icon_256.png")
}

pub fn load_icon() -> IconData {
    let bytes = png_bytes();
    let img = match image::load_from_memory(bytes) {
        Ok(img) => img.to_rgba8(),
        Err(_) => {
            // Fallback: solid 256x256 tile.
            let mut px = vec![0u8; 256 * 256 * 4];
            for v in px.chunks_mut(4).step_by(7) {
                v[0] = 255;
                v[1] = 122;
                v[2] = 69;
                v[3] = 255;
            }
            image::RgbaImage::from_raw(256, 256, px).expect("fallback icon")
        }
    };
    let (w, h) = img.dimensions();
    IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    }
}

pub fn load_icon_arc() -> Arc<IconData> {
    Arc::new(load_icon())
}
