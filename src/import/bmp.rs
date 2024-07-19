#![allow(clippy::cast_possible_truncation)]
use lunar_png::Image;

//Packed may cause issues with incorrect signature
#[repr(C, packed)]
#[derive(Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct Header {
    signature: [u8; 2],
    size: u32,
    reserved: u32,
    data_offset: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod)]
struct InfoHeader {
    header_size: u32,
    width: u32,
    height: u32,
    color_planes: u16,
    bpp: u16,
    ///Ignored for simplicity
    compression_method: u32,
    raw_bitmap_size: u32,
    ppm_widh: i32,
    ppm_height: i32,
    num_colors: u32,
    important_colors: u32,
}

const HEADER_SIZE: usize = 54;

///Parses a byte array as a bmp image
///# Errors
///fails if the file isn't 32 bit or has multiple color planes
pub fn parse(data: &[u8]) -> Result<Image, Box<dyn std::error::Error + Send>> {
    if data.len() <= HEADER_SIZE {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Wrong header size",
        )));
    }

    let header: &Header = bytemuck::from_bytes(&data[..14]);
    if header.signature != [0x42, 0x4D] {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Wrong signature",
        )));
    }
    if header.size != data.len() as u32 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid size",
        )));
    }

    let info_header: &InfoHeader = bytemuck::from_bytes(&data[14..54]);
    // if info_header.header_size != 40 {
    //     return Err(format!(
    //         "YOUR INFO HEADER IS TOO BIG. YOU SHOULD FEEL BAD {:?}",
    //         info_header
    //     )
    //     .into());
    // }
    if info_header.color_planes != 1 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Wrong number of color planes != 1",
        )));
    }
    // if info_header.compression_method != 0 {
    //     let a = info_header.compression_method;
    //     return Err(format!("Compression not supported found compression {}", a).into());
    // }
    // (info_header.bpp == 24)  ||
    if !info_header.bpp == 32 {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Unsported bits per pixel",
        )));
    }
    let out = data[header.data_offset as usize..]
        .chunks(4)
        .flat_map(|c| [c[2], c[1], c[0], c[3]])
        .collect();

    Ok(Image {
        img_type: lunar_png::ImageType::Rgba8,
        width: info_header.width,
        height: info_header.height,
        data: out,
    })
}
