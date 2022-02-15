use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_craft_new::chunk::{Chunk, CHUNK_SIZE, INDICES, VERTICES};


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(add_assets)
        .add_startup_system(setup)
        // .add_system(spawn)
        .run();
}

fn add_assets(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let box_material_handle = materials.add(Color::rgb(1.0, 0.2, 0.3).into());
    commands.insert_resource(MaterialHandle(box_material_handle));
}

fn setup(
    mut commands: Commands,
    mut query: Query<&ChunkInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(6., 5., 4.).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    for x in -8..8 {
        for y in -8..8 {
            for z in -8..8 {
                let info = ChunkInfo(Chunk::full(x, y, z));
                commands.spawn().insert_bundle(ChunkBundle {
                    info: info.clone(),
                    mesh: ChunkMesh(meshes.add(shape::Cube::new(1.).into()))
                    // mesh: ChunkMesh(meshes.add(generate_chunk_mesh(&query, &info.0))),
                });
            }
        }
    }
    info!("Done with startup");
}

struct MaterialHandle(Handle<StandardMaterial>);

fn spawn(mut commands: Commands,
         mut query: Query<(Entity, &mut ChunkMesh), Added<ChunkInfo>>,
         material_handle: Res<MaterialHandle>,
) {
    for (e, chunk_mesh) in query.iter() {
        info!("It found an entity!");
        commands.entity(e).insert_bundle(PbrBundle {
            mesh: chunk_mesh.0.clone(),
            material: material_handle.0.clone(),
            ..Default::default()
        });
    }
}

#[derive(Component, Clone)]
struct ChunkInfo(Chunk);

#[derive(Component)]
struct ChunkMesh(Handle<Mesh>);

#[derive(Bundle)]
struct ChunkBundle {
    info: ChunkInfo,
    mesh: ChunkMesh,
}

fn generate_chunk_mesh(mut query: &Query<&ChunkInfo>, chunk: &Chunk) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let (positions, normals, uvs, indices) = generate_chunk_data(&query, chunk);

    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    //TODO per vertex color for grass, should add color to generate_chunk_data
    // mesh.set_attribute(Mesh::ATTRIBUTE_COLOR,[]);
    mesh.set_indices(Some(Indices::U32(indices)));
    return mesh;
}

fn generate_chunk_data(
    mut query: &Query<&ChunkInfo>,
    chunk: &Chunk,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    // let mut positions = Vec::with_capacity(32 * 32 * 32); //max
    // let mut normals = Vec::with_capacity(32 * 32 * 32); //max
    // let mut uvs = Vec::with_capacity(32 * 32 * 32); //max
    // let mut indices = Vec::with_capacity(32 * 32 * 32);
    let mut positions = vec![]; //max
    let mut normals = vec![]; //max
    let mut uvs = vec![]; //max
    let mut indices = vec![];
    //max
    info!(
            "generating mesh with xyz: {} {} {}",
            chunk.x, chunk.y, chunk.z
        );
    chunk.blocks.iter().enumerate().for_each(|(i, b)| {
        match b {
            None => {}
            Some(b) => {
                let (block_x, block_y, block_z) = Chunk::index_to_coords(i);
                let faces: [bool; 6] = get_faces(
                    query,
                    chunk,
                    block_x,
                    block_y,
                    block_z,
                );

                // info!("faces: {:?}",faces);
                let uv = b.get_texture_uv();

                for (index, (position, normal)) in VERTICES.iter().enumerate() {
                    let (x, y, z) = Chunk::index_to_coords(i);
                    let position = [
                        position[0] + (x as isize + (32 * chunk.x as isize)) as f32,
                        position[1] + (y as isize + (32 * chunk.y as isize)) as f32,
                        position[2] + (z as isize + (32 * chunk.z as isize)) as f32,
                    ];
                    if faces[index / 4] {
                        positions.push(position);
                        normals.push(*normal);
                        uvs.push(uv[index]);
                    } else {
                        positions.push([0., 0., 0.]);
                        normals.push([0., 0., 0.]);
                        uvs.push([0., 0.]);
                    }
                }
                for x in 0..6 {
                    if faces[x] {
                        let temp = &mut INDICES[x * 6..(x * 6) + 6];
                        for u in temp.iter_mut() {
                            *u += (i * 24) as u32;
                        }
                        indices.extend_from_slice(temp);
                    } else {
                        indices.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
                    }
                }
            }
        }
    });
    (positions, normals, uvs, indices)
}

fn get_faces(
    mut query: &Query<&ChunkInfo>,
    c: &Chunk,
    b_x: usize,
    b_y: usize,
    b_z: usize,
) -> [bool; 6] {
    let (w_x, w_y, w_z) = convert_to_world_coords(c.x as isize, c.y as isize, c.z as isize, b_x, b_y, b_z);
    // info!("world xyz: {} {} {}",w_x,w_y,w_z);
    let top = {
        let (w_x, w_y, w_z) = (w_x, w_y, w_z + 1);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(w_x, w_y, w_z);
        if (c_x == c.x as isize && c_y == c.y as isize && c_z == c.z as isize) {
            let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
            !(c.is_block(b_x, b_y, b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        }
    };
    let bottom = {
        let (w_x, w_y, w_z) = (w_x, w_y, w_z - 1);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(w_x, w_y, w_z);
        if (c_x == c.x as isize && c_y == c.y as isize && c_z == c.z as isize) {
            let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
            !(c.is_block(b_x, b_y, b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        }
    };
    let right = {
        let (w_x, w_y, w_z) = (w_x + 1, w_y, w_z);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(w_x, w_y, w_z);
        if (c_x == c.x as isize && c_y == c.y as isize && c_z == c.z as isize) {
            let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
            !(c.is_block(b_x, b_y, b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        }
    };
    let left = {
        let (w_x, w_y, w_z) = (w_x - 1, w_y, w_z);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(w_x, w_y, w_z);
        if (c_x == c.x as isize && c_y == c.y as isize && c_z == c.z as isize) {
            let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
            !(c.is_block(b_x, b_y, b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        }
    };
    let front = {
        let (w_x, w_y, w_z) = (w_x, w_y + 1, w_z);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(w_x, w_y, w_z);
        if (c_x == c.x as isize && c_y == c.y as isize && c_z == c.z as isize) {
            let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
            !(c.is_block(b_x, b_y, b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        }
    };
    let back = {
        let (w_x, w_y, w_z) = (w_x, w_y - 1, w_z);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(w_x, w_y, w_z);
        if (c_x == c.x as isize && c_y == c.y as isize && c_z == c.z as isize) {
            let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
            !(c.is_block(b_x, b_y, b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(w_x, w_y, w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        }
    };
    [top, bottom, right, left, front, back]
}

fn block_coords_in_chunk(x: isize, y: isize, z: isize) -> (usize, usize, usize) {
    (
        (x % 32) as usize,
        (y % (32 * 32 * 32)) as usize,
        (z % (32 * 32)) as usize,
    )
}

fn get_chunk_from_coords(mut query: &Query<&ChunkInfo>, x: isize, y: isize, z: isize) -> Option<ChunkInfo> {
    let index = chunk_coords_to_index(x, y, z) as isize;
    if (0..(32 * 32 * 32)).contains(&index) {
        for c in query.iter() {
            if c.0.x as isize == x && c.0.y as isize == y && c.0.z as isize == z {
                return Some(c.clone());
            }
        }
    }
    None
}

fn chunk_coords_to_index(x: isize, y: isize, z: isize) -> usize {
    let x1 = x; // incremental is the same
    let y1 = y * 32 * 32; // incremental is 1:32*32
    let z1 = z * 32; // incremental is 1:32
    // info!("chunk coord to index: {:?} {}",(x,y,z),i);
    ((x1 + y1 + z1) + (CHUNK_SIZE / 2) as isize) as usize
}

fn convert_to_world_coords(
    c_x: isize,
    c_y: isize,
    c_z: isize,
    b_x: usize,
    b_y: usize,
    b_z: usize,
) -> (isize, isize, isize) {
    (
        (c_x * 32) + b_x as isize,
        (c_y * 32) + b_y as isize,
        (c_z * 32) + b_z as isize,
    )
}

fn convert_to_block_coords(w_x: isize, w_y: isize, w_z: isize) -> (usize, usize, usize) {
    (
        if w_x < 0 { 32 + (w_x % 32) } else { w_x % 32 } as usize,
        if w_y < 0 { 32 + (w_y % 32) } else { w_y % 32 } as usize,
        if w_z < 0 { 32 + (w_z % 32) } else { w_z % 32 } as usize,
    )
}

fn convert_to_chunk_coords(w_x: isize, w_y: isize, w_z: isize) -> (isize, isize, isize) {
    (
        if w_x < 0 { (w_x / 32) - 1 } else { w_x / 32 },
        if w_y < 0 { (w_y / 32) - 1 } else { w_y / 32 },
        if w_z < 0 { (w_z / 32) - 1 } else { w_z / 32 },
    )
}
