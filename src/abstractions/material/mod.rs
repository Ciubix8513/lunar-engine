pub mod texture_unlit;

pub trait Material<'a, 'b> {
    fn render(&'a self, render_pass: &mut wgpu::RenderPass<'b>)
    where
        'a: 'b;
}
