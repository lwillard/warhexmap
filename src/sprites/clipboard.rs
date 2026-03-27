use arboard::Clipboard;

use crate::types::SpriteImage;

/// Read an image from the system clipboard and convert it to a SpriteImage.
pub fn read_clipboard_image(clipboard: &mut Clipboard) -> Result<SpriteImage, String> {
    let img_data = clipboard
        .get_image()
        .map_err(|e| format!("Failed to read image from clipboard: {}", e))?;

    let width = img_data.width as u32;
    let height = img_data.height as u32;

    if width == 0 || height == 0 {
        return Err("Clipboard image has zero dimensions".to_string());
    }

    // arboard returns image data as RGBA bytes in img_data.bytes
    let pixels = img_data.bytes.into_owned();

    if pixels.len() != (width * height * 4) as usize {
        return Err(format!(
            "Clipboard image pixel data size mismatch: expected {}, got {}",
            width * height * 4,
            pixels.len()
        ));
    }

    Ok(SpriteImage {
        width,
        height,
        pixels,
    })
}
