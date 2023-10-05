use crate::{
    math::{vec2::Vec2, vec3::Vec3},
    structrures::model::{Mesh, Vertex},
};

fn read_vec3(input: &str) -> Option<Vec3> {
    let split: Vec<Option<f32>> = input.split(' ').map(|i| str::parse(i).ok()).collect();
    if split.len() != 3 {
        return None;
    }
    Some(Vec3 {
        x: split[0]?,
        y: split[1]?,
        z: split[2]?,
    })
}

fn read_vec2(input: &str) -> Option<Vec2> {
    let split: Vec<Option<f32>> = input.split(' ').map(|i| str::parse(i).ok()).collect();
    if split.len() != 2 {
        return None;
    }
    Some(Vec2 {
        x: split[0]?,
        y: split[1]?,
    })
}

fn get_indecies(input: &str) -> Option<(u32, u32, u32)> {
    let split: Vec<Option<u32>> = input.split('/').map(|i| str::parse(i).ok()).collect();
    if split.len() != 3 {
        return None;
    }
    Some((split[0]?, split[1]?, split[2]?))
}

#[test]
fn test_read_vec2() {
    let input = "1.000 2.000";
    let output = read_vec2(input).unwrap();
    let expected = Vec2::new(1.000, 2.000);
    assert_eq!(output, expected);
}
#[test]
fn test_read_vec3() {
    let input = "1.000 2.000 3.000";
    let output = read_vec3(input).unwrap();
    let expected = Vec3::new(1.000, 2.000, 3.000);
    assert_eq!(output, expected);
}
#[test]
fn test_get_indecies() {
    let input = "1/2/3";
    let output = get_indecies(input).unwrap();
    let expected = (1, 2, 3);
    assert_eq!(output, expected);
}
#[test]
fn test_loading_single() {
    parse_obj(include_str!("../../../assets/cube.obj")).unwrap();
}

///Parses the given string as a wavefront obj file
pub fn parse_obj(file: &str) -> Option<Vec<Mesh>> {
    let mut meshes = Vec::new();
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let mut vertices_indecies = Vec::new();
    let mut vertices = Vec::new();
    let mut indecies = Vec::new();

    let mut first = true;
    for l in file.lines() {
        //Hit a new object reset
        if l.starts_with("o ") {
            if first {
                first = false;
                continue;
            }
            //Insert into the out vector
            meshes.push(Mesh { vertices, indecies });

            //Clear data
            vertices = Vec::new();
            indecies = Vec::new();
            vertices_indecies = Vec::new();
            positions = Vec::new();
            normals = Vec::new();
            uvs = Vec::new();
        }
        if l.starts_with("v ") {
            //read the position
            positions.push(read_vec3(&l[2..])?);
        }
        if l.starts_with("vt ") {
            //read the position
            uvs.push(read_vec2(&l[3..])?);
        }
        if l.starts_with("vn ") {
            //read the position
            normals.push(read_vec3(&l[3..])?);
        }
        if l.starts_with("f ") {
            let component_indecies: Vec<Option<(u32, u32, u32)>> =
                l[2..].split(' ').map(get_indecies).collect();
            for i in &component_indecies {
                let i = (*i)?;

                let mut found = false;

                for j in 0..vertices_indecies.len() {
                    if vertices_indecies[j] == i {
                        //If found an existing vertex, push it's index
                        indecies.push(j as u32);
                        found = true;
                        break;
                    }
                }
                if found {
                    continue;
                }

                //Create the new vertex
                vertices_indecies.push(i);
                vertices.push(Vertex {
                    coords: positions[(i.0 - 1) as usize],
                    texture: uvs[(i.1 - 1) as usize],
                    normal: normals[(i.2 - 1) as usize],
                });

                indecies.push((vertices.len() - 1) as u32);
            }
        }
    }

    meshes.push(Mesh { vertices, indecies });

    log::error!("Read {} meshes", meshes.len());
    for i in &meshes {
        log::error!(
            "verex len = {}, ind len = {}",
            i.vertices.len(),
            i.indecies.len()
        );
    }

    return Some(meshes);
}
