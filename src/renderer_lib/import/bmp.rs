use crate::structrures::image::{Image, Pixel};

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

pub fn parse(data: &[u8]) -> Result<Image, Box<dyn std::error::Error>> {
    if data.len() <= HEADER_SIZE {
        return Err("Invalid data, header too small".into());
    }

    let header: &Header = bytemuck::from_bytes(&data[..14]);
    if header.signature != [0x42, 0x4D] {
        return Err("Invalid data, wrong signature".into());
    }
    if header.size != data.len() as u32 {
        return Err("Invalid data, invalid size".into());
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
        return Err("Invalid data, wrong number of color planes != 1".into());
    }
    // if info_header.compression_method != 0 {
    //     let a = info_header.compression_method;
    //     return Err(format!("Compression not supported found compression {}", a).into());
    // }
    // (info_header.bpp == 24)  ||
    if !info_header.bpp == 32 {
        return Err("Unsported bits per pixel".into());
    }
    let mut out = Vec::new();

    for i in data[header.data_offset as usize..].chunks(4) {
        out.push(*bytemuck::from_bytes::<Pixel>(i))
    }

    for i in out.iter_mut() {
        *i = Pixel {
            r: i.b,
            g: i.g,
            b: i.r,
            a: i.a,
        };
    }

    Ok(Image {
        width: info_header.width,
        height: info_header.height,
        data: out,
    })
}

#[test]
fn test_image_loading() {
    parse(include_bytes!("../../../assets/test.bmp")).unwrap();
}
