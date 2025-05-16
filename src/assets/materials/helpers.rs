use wgpu::VertexBufferLayout;

///Returns the default vertex buffer bindings
#[must_use]
pub const fn vertex_binding() -> [VertexBufferLayout<'static>; 2] {
    [
        //Vertex data
        wgpu::VertexBufferLayout {
            array_stride: 32,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Position
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                //UV
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                //Normals
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 20,
                    shader_location: 2,
                },
            ],
        },
        //Transform data
        //Encoding a matrix as 4 vec4
        //Just that i can do instanced rendering
        wgpu::VertexBufferLayout {
            array_stride: 64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 32,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 48,
                    shader_location: 6,
                },
            ],
        },
    ]
}

#[must_use]
///Returns whether or not storage buffers are available on the current device
pub const fn storage_buffer_available() -> bool {
    // let features = DEVICE.get().unwrap().features();

    // features.contains(wgpu::Features::STORAGE_RESOURCE_BINDING_ARRAY)

    //This is a mess but eh
    #[cfg(feature = "webgl")]
    let o = false;
    #[cfg(not(feature = "webgl"))]
    let o = true;

    o
}

///Errors returned by the preprocessor
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Error {
    ///Syntax of the preprocessor is not valid
    InvalidSyntax(&'static str),
    ///A block with the specified index does not exist
    BlockDoesNotExist,
}

///processes a shader, picking which one of the blocks to use.
///
///Consider the following shader:
///```wgsl
///### 0
///let a: f32 = 0.0;
///### 1
///let a: u32 = 1;
///###
///```
///
///The preprocessor, can pick which one of the blocks to use, based on `block_index`.
///
///# Errors
///
///Will return an error if the syntax of the preprocessor is invalid
pub fn preprocess_shader(shader: &str, block_index: u32) -> Result<String, Error> {
    let lines = shader.lines();
    let blocks = lines
        .enumerate()
        .filter(|i| i.1.contains("###"))
        .collect::<Vec<_>>();

    if blocks.is_empty() {
        return Ok(shader.into());
    } else if blocks.len() < 3 {
        return Err(Error::InvalidSyntax("Must have at least 2 blocks"));
    }
    //We have at least 3 blocks, now make sure that they have the correct syntax

    let mut indecies = Vec::new();

    for (i, (l, b)) in blocks.iter().enumerate() {
        let ind = b.replace("###", " ");
        let index = ind.trim();

        let last = i == blocks.len() - 1;

        if index.is_empty() && !last {
            return Err(Error::InvalidSyntax("A block does not have an index"));
        }

        if last && !index.is_empty() {
            return Err(Error::InvalidSyntax(
                "The last block separator must not have an index",
            ));
        }

        let ind = if last {
            Ok(u32::MAX)
        } else {
            index.parse::<u32>()
        };

        if ind.is_err() && !last {
            return Err(Error::InvalidSyntax(
                "Block index must be a non negative number",
            ));
        }
        let index = ind.unwrap();

        if indecies.iter().filter(|(_, i)| i == &index).count() != 0 {
            return Err(Error::InvalidSyntax("Block indecies must be unique"));
        }

        indecies.push((*l, index));
    }

    if indecies.iter().filter(|(_, i)| i == &block_index).count() == 0 {
        return Err(Error::BlockDoesNotExist);
    }

    //Now we are sure that the block syntax is correct and the requested block exists

    //Get block contents of the requested block
    let block = indecies
        .iter()
        .enumerate()
        .find(|(_, (_, i))| i == &block_index)
        .unwrap();
    let block_beginning = block.1 .0;
    let block_end = blocks[block.0 + 1].0;

    let block_contents = shader.lines().collect::<Vec<_>>()[block_beginning + 1..block_end]
        .iter()
        .fold(String::new(), |s, i| s + i + "\n");

    let output = shader.lines().collect::<Vec<_>>()[blocks.last().unwrap().0 + 1..]
        .iter()
        .fold(block_contents, |s, i| s + i + "\n");

    Ok(output)
}

#[test]
fn preprocessor_test() {
    let shader = "### 0
:neofox_snug:
### 1
:neocat_floof:
###
meow meow";

    assert_eq!(
        preprocess_shader(shader, 1).unwrap().trim(),
        ":neocat_floof:\nmeow meow"
    );

    assert_eq!(
        preprocess_shader(shader, 0).unwrap().trim(),
        ":neofox_snug:\nmeow meow"
    );
}
