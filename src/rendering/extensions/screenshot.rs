use std::{io::Write, path::PathBuf, thread};

use chrono::{Datelike, Timelike};
use lunar_png::{CompressionLevel, Image, PngEncodingOptions};

use crate::{
    internal::{DEVICE, FORMAT, QUEUE, RESOLUTION},
    rendering::extensions::RenderingExtension,
};

///An extension for capturing screenshots
///
///<div class="warning">
///DO NOT put this extension into your regular render loop, only add it for individual frames, when
///you need to take a screenshot
///</div>
#[derive(Default)]
pub struct Screenshot {
    ///Configuration of the screenshot extension
    pub config: ScreenshotConfig,
    buffer: Option<wgpu::Buffer>,
    size: u32,
    resolution: (u32, u32),
    data: Vec<u8>,
    screenshot_saved: bool,
    padding: u32,
    fmt_size: u32,
}

///Configuration of the screenshot taking tool
#[derive(Clone)]
pub struct ScreenshotConfig {
    ///Filename prefix of the screenshots
    ///
    ///default  value is `lunar-engine`
    pub filename_prefix: String,

    ///The directory where the screenshots folder  is
    ///
    ///By default set to save in
    ///`~/Pictures` on Linux
    ///
    ///For platform compatibility reasons it is not recommended to change this
    pub save_directory_prefix: PathBuf,
    ///The actual name of the Screenshots folder
    ///
    ///By default set to `lunar-engine/screenshots`
    pub save_directory: PathBuf,
    ///Whether the filename includes time
    ///
    ///if yes the filename will be:
    ///
    ///`filename_prefix-yyyy-mm-dd-hh-mm-ss.png`
    ///
    ///The default value is yes
    pub save_time: bool,

    ///Whether the screenshot should be saved immediately upon capture, or if the save function
    ///needs to be explicitly called
    ///
    ///The default value is yes
    pub save_on_capture: bool,

    ///Compression level of the saved screenshot, `CompressionLevel::Fast` is
    ///recommended for most cases
    ///
    ///The default value is `Fast`
    pub copmression: CompressionLevel,

    ///Whether to copy the screenshot to the clipboard
    ///
    ///The default value is `true`
    pub copy_to_clipboard: bool,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        //This is so cursed
        //
        //Theoretically this can only fail on linux so no need to do platform specific stuff?
        let save_directory_prefix = directories::UserDirs::new()
            .and_then(|i| i.picture_dir().map(|i| i.to_path_buf()))
            .unwrap_or_else(|| {
                std::env::var("HOME")
                    .expect("Could not find home directory")
                    .into()
            });

        Self {
            filename_prefix: "lunar-engine".into(),
            save_time: true,
            save_on_capture: true,
            copmression: lunar_png::CompressionLevel::Fast,
            save_directory_prefix,
            save_directory: "lunar-engine/screenshots/".into(),
            copy_to_clipboard: true,
        }
    }
}

impl Screenshot {
    ///Saves the taken screenshot manually
    pub fn save_image(&self) -> Result<(), std::io::Error> {
        save_image(
            self.config.clone(),
            self.data.clone(),
            self.resolution,
            lunar_png::ImageType::Rgba8,
        )
    }
}

fn save_image(
    config: ScreenshotConfig,
    data: Vec<u8>,
    size: (u32, u32),
    image_type: lunar_png::ImageType,
) -> Result<(), std::io::Error> {
    let time = chrono::Local::now();
    let year = time.year();
    let month = time.month();
    let day = time.day();
    let hour = time.hour();
    let minute = time.minute();
    let sec = time.second();

    let filename = format!(
        "{}-{year}-{month}-{day}-{hour}-{minute}-{sec}.png",
        config.filename_prefix
    );

    let path = &config.save_directory_prefix.join(config.save_directory);
    std::fs::create_dir_all(path)?;

    let path = path.join(filename);

    log::info!(
        "Saving screenshot as {}",
        path.to_str().unwrap_or("Couldn't convert path to string")
    );

    let img = lunar_png::encode_png(
        &Image {
            data,
            width: size.0,
            height: size.1,
            img_type: image_type,
        },
        &PngEncodingOptions {
            compression: config.copmression,
            write_timestamp: true,
        },
    );

    let mut f = std::fs::File::create(&path)?;

    f.write_all(&img)?;

    if config.copy_to_clipboard {
        crate::set_clipboard_png(img.clone());
    }

    Ok(())
}

impl RenderingExtension for Screenshot {
    fn render(
        &mut self,
        _: &mut wgpu::CommandEncoder,
        _: &crate::ecs::World,
        _: &mut crate::asset_managment::AssetStore,
        _: &super::AttachmentData,
    ) {
        //DO nothing :3
    }

    fn post_render(&mut self, attachment_data: &super::AttachmentData) {
        if !crate::APP_INFO
            .get()
            .unwrap()
            .read()
            .unwrap()
            .screenshot_supported
        {
            //Screenshot feature not supported
            return;
        }

        let resolution = RESOLUTION.read().unwrap();
        let device = DEVICE.get().unwrap();

        if resolution.width != self.resolution.0 || resolution.height != self.resolution.1 {
            self.resolution.0 = resolution.width;
            self.resolution.1 = resolution.height;

            let fmt = FORMAT.get().unwrap();
            let fmt_size = fmt.block_copy_size(None).unwrap();

            self.fmt_size = fmt_size;
            let padding = 256 - ((fmt_size * resolution.width) % 256);

            self.padding = padding;

            let needed_size = (fmt_size * resolution.width + padding) * resolution.height;

            //re create the buffer
            if needed_size != self.size {
                self.size = needed_size;
                self.buffer = Some(device.create_buffer(&wgpu::wgt::BufferDescriptor {
                    label: Some("Screenshot copy buffer"),
                    size: needed_size as u64,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }));
            }
        }

        //hmm i can do a new command encoder, or just recycle that one...

        let mut enc = device.create_command_encoder(&wgpu::wgt::CommandEncoderDescriptor {
            label: Some("Screenshot command encoder"),
        });

        let img_width = resolution.width * self.fmt_size;

        let remainder = 256 - (img_width % 256);

        log::info!("img_width: {img_width}");
        log::info!("Padding: {remainder}");
        log::info!("Total width = {}", img_width + remainder);

        enc.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfoBase {
                texture: attachment_data.color.texture(),
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfoBase {
                buffer: self.buffer.as_ref().unwrap(),
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(resolution.width * self.fmt_size + remainder),
                    rows_per_image: Some(resolution.height),
                },
            },
            wgpu::Extent3d {
                width: resolution.width,
                height: resolution.height,
                depth_or_array_layers: 1,
            },
        );

        let cmd = enc.finish();
        let queue = QUEUE.get().unwrap();
        queue.submit([cmd]);

        let slice = self.buffer.as_ref().unwrap().slice(..);
        slice.map_async(wgpu::MapMode::Read, |i| {
            i.unwrap();
        });

        device.poll(wgpu::wgt::PollType::Wait).unwrap();

        self.data = slice
            .get_mapped_range()
            .iter()
            .copied()
            .collect::<Vec<_>>()
            .chunks((self.resolution.0 * self.fmt_size + self.padding) as usize)
            .flat_map(|i| {
                i[0..(self.resolution.0 * self.fmt_size) as usize]
                    .into_iter()
                    .collect::<Vec<_>>()
            })
            .copied()
            .collect();

        self.buffer.as_ref().unwrap().unmap();
        self.screenshot_saved = true;

        if self.config.save_on_capture {
            let c = self.config.clone();
            let d = self.data.clone();
            let size = self.resolution;

            thread::spawn(move || {
                let r = save_image(c, d, size, lunar_png::ImageType::Rgba8);

                if let Err(e) = r {
                    log::error!("Could not save the screenshot {e:?}");
                }
            });
        }
    }

    fn get_priority(&self) -> u32 {
        0
    }
}
