use std::f32::consts::PI;

use crate::{
    math::{Vec3, Vec4},
    structures::{Mesh, Vertex},
};

use super::ModelType;

pub fn generate_mesh(mesh_type: &ModelType) -> Mesh {
    match mesh_type {
        ModelType::Box(dimensions) => generate_box(*dimensions),
        ModelType::Sphere(data) => generate_sphere(data.radius, data.segments, data.rings),
    }
}

fn generate_box(dimensions: Vec3) -> Mesh {
    let mut o = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    };

    let hx = dimensions.x;
    // / 2.0
    let hy = dimensions.y;
    // / 2.0
    let hz = dimensions.z;
    // / 2.0
    o.vertices = vec![
        //Top face
        Vertex {
            coords: Vec4::new(hx, hy, hz, 1.0),
            normal: Vec3::new(0.0, 1.0, 0.0),

            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(hx, hy, -hz, 1.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, hy, hz, 1.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, hy, -hz, 1.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            ..Default::default()
        },
        //bottom
        Vertex {
            coords: Vec4::new(hx, -hy, hz, 1.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(hx, -hy, -hz, 1.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, -hy, hz, 1.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, -hy, -hz, 1.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            ..Default::default()
        },
        //forward
        Vertex {
            coords: Vec4::new(hx, hy, hz, 1.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, -hy, hz, 1.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(hx, -hy, hz, 1.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, hy, hz, 1.0),
            normal: Vec3::new(0.0, 0.0, 1.0),
            ..Default::default()
        },
        //Back
        Vertex {
            coords: Vec4::new(hx, hy, -hz, 1.0),
            normal: Vec3::new(0.0, 0.0, -1.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, -hy, -hz, 1.0),
            normal: Vec3::new(0.0, 0.0, -1.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(hx, -hy, -hz, 1.0),
            normal: Vec3::new(0.0, 0.0, -1.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, hy, -hz, 1.0),
            normal: Vec3::new(0.0, 0.0, -1.0),
            ..Default::default()
        },
        //left
        Vertex {
            coords: Vec4::new(hx, hy, hz, 1.0),
            normal: Vec3::new(1.0, 0.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(hx, -hy, -hz, 1.0),
            normal: Vec3::new(1.0, 0.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(hx, -hy, hz, 1.0),
            normal: Vec3::new(1.0, 0.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(hx, hy, -hz, 1.0),
            normal: Vec3::new(1.0, 0.0, 0.0),
            ..Default::default()
        },
        //right
        Vertex {
            coords: Vec4::new(-hx, hy, hz, 1.0),
            normal: Vec3::new(-1.0, 0.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, -hy, -hz, 1.0),
            normal: Vec3::new(-1.0, 0.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, -hy, hz, 1.0),
            normal: Vec3::new(-1.0, 0.0, 0.0),
            ..Default::default()
        },
        Vertex {
            coords: Vec4::new(-hx, hy, -hz, 1.0),
            normal: Vec3::new(-1.0, 0.0, 0.0),
            ..Default::default()
        },
    ];

    o.indices = vec![
        0, 1, 2, 2, 1, 3, //Top face
        4, 5, 6, 6, 5, 7, //Bottom face
        8, 9, 10, 9, 8, 11, //Forward face
        12, 13, 14, 13, 12, 15, //Back face
        16, 17, 18, 17, 16, 19, //Left face
        20, 21, 22, 21, 20, 23, //Left face
    ];

    o
}

fn generate_sphere(radius: f32, segments: i32, rings: i32) -> Mesh {
    assert!(rings >= 3);
    assert!(segments >= 3);

    let mut o = Mesh::default();

    //Add top vertex
    o.vertices.push(Vertex {
        coords: Vec4::new(0.0, -radius, 0.0, 1.0),
        ..Default::default()
    });

    let rings = rings - 2;

    //Step in radians
    let ring_step = f32::to_radians(180.0 / (rings + 1) as f32);
    let segment_step = f32::to_radians(360.0 / segments as f32);

    for i in 0..rings {
        //Y value for the current ring
        let theta = PI - ring_step * (i as f32 + 1.0);
        let y = f32::cos(theta);
        //iterate over segments
        for j in 0..segments {
            let z = f32::sin(theta) * f32::cos(segment_step * j as f32);
            let x = f32::sin(theta) * f32::sin(segment_step * j as f32);

            o.vertices.push(Vertex {
                coords: Vec4 { x, y, z, w: 1.0 } * radius,
                ..Default::default()
            })
        }
    }
    //Add bottom vertex
    o.vertices.push(Vertex {
        coords: Vec4::new(0.0, radius, 0.0, 1.0),
        ..Default::default()
    });
    //Indecies

    //top tris
    for i in 0..segments {
        let origin = 0;
        o.indices.push(origin as u32);
        for j in 1..3 {
            let mut value = (origin + i + j) % (segments + 1);
            if value == origin {
                value = origin + 1;
            }

            o.indices.push(value as u32);
        }
    }
    //bottom tris
    for i in 0..segments {
        let bottom = (o.vertices.len() - 1) as i32;
        o.indices.push((bottom - segments + i) as u32);
        o.indices.push(bottom as u32);
        let mut v = bottom - segments + i + 1;
        if v == bottom {
            v = bottom - segments;
        }
        o.indices.push(v as u32);
    }

    if rings == 1 {
        return o;
    }

    let segments = segments as u32;
    //Quads....
    for i in 0..(rings - 1) {
        let offset = i as u32 * segments + 1;
        for j in 0..segments {
            if (offset + j) % segments == 0 {
                o.indices.push(offset + j);
                o.indices.push(offset + j + segments);
                o.indices.push(offset + j + 1);

                o.indices.push(offset + j);
                o.indices.push(offset + j + 1);
                o.indices.push(offset + j + 1 - segments);

                continue;
            }
            //Tri 1
            o.indices.push(offset + j);
            o.indices.push(offset + j + segments);
            o.indices.push(offset + j + segments + 1);

            //Tri 2
            o.indices.push(offset + j);
            o.indices.push(offset + j + segments + 1);
            o.indices.push(offset + j + 1);
        }
    }

    o
}
