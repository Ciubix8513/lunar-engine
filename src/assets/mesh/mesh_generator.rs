#![allow(clippy::too_many_lines)]

use std::f32::consts::PI;

use crate::{
    math::{Vec3, Vec4},
    structures::{Mesh, Vertex},
};

use super::ModelType;

#[must_use]
pub fn generate_mesh(mesh_type: &ModelType) -> Mesh {
    match mesh_type {
        ModelType::Box(dimensions) => generate_box(*dimensions),
        ModelType::Sphere(data) => generate_sphere(data.radius, data.segments, data.rings),
    }
}

#[must_use]
fn generate_box(dimensions: Vec3) -> Mesh {
    let mut o = Mesh::default();

    let hx = dimensions.x / 2.0;
    let hy = dimensions.y / 2.0;
    let hz = dimensions.z / 2.0;

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
        20, 21, 22, 21, 20, 23, //Right face
    ];

    o
}

#[must_use]
fn generate_sphere(radius: f32, segments: u32, rings: u32) -> Mesh {
    assert!(rings >= 1, "A sphere must have at least one ring");
    assert!(segments >= 3, "A sphere must have at least 3 segments");

    let mut o = Mesh::default();

    //Add top vertex
    o.vertices.push(Vertex {
        coords: Vec4::new(0.0, -radius, 0.0, 1.0),
        normal: Vec3::new(0.0, -1.0, 0.0),
        ..Default::default()
    });

    //Step in radians
    let ring_step = f32::to_radians(180.0 / (rings + 1) as f32);
    let segment_step = f32::to_radians(360.0 / segments as f32);

    for i in 0..rings {
        //Y value for the current ring
        let theta = ring_step.mul_add(-(i as f32 + 1.0), PI);

        let y = f32::cos(theta);
        let sin_theta = f32::sin(theta);
        //iterate over segments
        for j in 0..segments {
            //Spherical coords
            let z = sin_theta * f32::cos(segment_step * j as f32);
            let x = sin_theta * f32::sin(segment_step * j as f32);

            o.vertices.push(Vertex {
                coords: Vec4 { x, y, z, w: 1.0 } * radius,
                //This should work since the length is 1
                normal: Vec3 { x, y, z },
                ..Default::default()
            });
        }
    }

    //Add bottom vertex
    o.vertices.push(Vertex {
        coords: Vec4::new(0.0, radius, 0.0, 1.0),
        normal: Vec3::new(0.0, 1.0, 0.0),
        ..Default::default()
    });

    //Indecies

    //top tris
    for i in 0..segments {
        o.indices.push(0);
        for j in 1..3 {
            let mut value = (i + j) % (segments + 1);
            if value == 0 {
                value = 1;
            }
            o.indices.push(value);
        }
    }

    //bottom tris
    let bottom = (o.vertices.len() - 1) as u32;

    for i in 0..segments {
        o.indices.push(bottom - segments + i);
        o.indices.push(bottom);

        let mut v = bottom - segments + i + 1;
        if v == bottom {
            v = bottom - segments;
        }
        o.indices.push(v);
    }

    //Don't do quads if there are no quads
    if rings == 1 {
        return o;
    }

    //Quads
    for i in 0..(rings - 1) {
        //Offset of the first vertex of the ring
        let offset = i * segments + 1;

        for j in 0..segments {
            let j = offset + j;

            if j % segments == 0 {
                o.indices.push(j);
                o.indices.push(j + segments);
                o.indices.push(j + 1);

                o.indices.push(j);
                o.indices.push(j + 1);
                o.indices.push(j + 1 - segments);

                continue;
            }
            //Tri 1
            o.indices.push(j);
            o.indices.push(j + segments);
            o.indices.push(j + segments + 1);

            //Tri 2
            o.indices.push(j);
            o.indices.push(j + segments + 1);
            o.indices.push(j + 1);
        }
    }

    o
}
