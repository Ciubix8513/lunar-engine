pub fn calculate_bpr(width: u32, format: wgpu::TextureFormat) -> u64 {
    let mut bpr = u64::from(width * format.block_size(None).unwrap());
    if bpr % 256 != 0 {
        bpr = bpr + (256 - (bpr % 256));
    }
    bpr
}
