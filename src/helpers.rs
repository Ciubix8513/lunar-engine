#![allow(clippy::missing_errors_doc, dead_code)]
use std::io::Cursor;

use image::{ImageBuffer, ImageError, Rgba};

use crate::DEVICE;

pub fn arr_to_image(
    img: &[u8],
    bytes_per_row: u32,
    width: u32,
    height: u32,
    format: image::ImageOutputFormat,
) -> Result<Vec<u8>, ImageError> {
    let img = img
        .chunks_exact(4)
        .map(|i| {
            let mut array = [0; 4];
            array.copy_from_slice(i);
            let tmp = Rgba(array);
            let mut out = tmp;
            out[0] = tmp[2];
            out[2] = tmp[0];
            out
        })
        .collect::<Vec<Rgba<u8>>>();

    log::debug!("Collected the array");
    log::debug!("BPR = {bytes_per_row}, width = {width}");
    let mut image_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(width, height);
    let mut x = 0;
    let mut y = 0;

    for i in img {
        //Remove padding
        if width != bytes_per_row && x >= width {
            x += 1;
            if x == bytes_per_row {
                x = 0;
                y += 1;
            }
            continue;
        }

        *image_buffer.get_pixel_mut(x, y) = i;

        x += 1;
        //This is kind of a hacky fix, but it works
        if x == width && width == bytes_per_row {
            x = 0;
            y += 1;
        }
    }
    log::debug!("Iterated through the array");
    let mut byte_stream = Vec::new();
    image_buffer.write_to(&mut Cursor::new(&mut byte_stream), format)?;

    Ok(byte_stream)
}

pub fn create_uniform_matrix(label: Option<&str>) -> wgpu::Buffer {
    let device = DEVICE.get().unwrap();

    device.create_buffer(&wgpu::BufferDescriptor {
        label,
        size: std::mem::size_of::<crate::math::mat4x4::Mat4x4>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub fn calculate_bpr(width: u32, format: wgpu::TextureFormat) -> u64 {
    let mut bpr = u64::from(width * format.block_copy_size(None).unwrap());
    if bpr % 256 != 0 {
        bpr = bpr + (256 - (bpr % 256));
    }
    bpr
}
