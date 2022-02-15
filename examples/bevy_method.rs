use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_craft_new::chunk::{Chunk, CHUNK_SIZE};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(add_assets)
        .add_startup_system(setup)
        .insert_resource(BlockMeshes(generate_block_meshes()))
        // .add_system(spawn)
        .add_system(update_mesh)
        .run();
}

struct MaterialHandle(Handle<StandardMaterial>);

struct BlockMeshes(Vec<Mesh>);

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
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(6., 5., 4.).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
    let mesh = meshes.add(Mesh::new(PrimitiveTopology::TriangleList));
    for x in -8..8 {
        for y in -8..8 {
            for z in -8..8 {
                let info = ChunkInfo(Chunk::full(x, y, z));
                commands.spawn().insert_bundle(ChunkBundle {
                    info: info.clone(),
                    mesh: ChunkMesh(mesh.clone()),
                    // mesh: ChunkMesh(meshes.add(generate_chunk_mesh(&query, &info.0, &block_meshes))),
                });
            }
        }
    }
    info!("Done with startup");
}

fn update_mesh(
    mut query: Query<(&ChunkInfo, &mut ChunkMesh), Changed<ChunkInfo>>,
    mut chunk_query: Query<&ChunkInfo>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut block_meshes: ResMut<BlockMeshes>,
    mut commands: Commands,
) {
    query.for_each_mut(|(ci, mut cm)| {
        cm.0 =  meshes.add(generate_chunk_mesh(&chunk_query, &ci.0, &block_meshes));
    });
}


fn spawn(mut commands: Commands,
         mut query: Query<(Entity, &mut ChunkMesh), Changed<ChunkInfo>>,
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

fn generate_chunk_mesh(
    mut query: &Query<&ChunkInfo>,
    chunk: &Chunk,
    mut block_meshes: &ResMut<BlockMeshes>,
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let (positions, normals, uvs, indices) = generate_chunk_data(&query, chunk, block_meshes);

    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    //TODO per vertex color for grass, should add color to generate_chunk_data
    // mesh.set_attribute(Mesh::ATTRIBUTE_COLOR,[]);
    mesh.set_indices(Some(Indices::U32(indices)));
    return mesh;
}

fn generate_block_meshes() -> Vec<Mesh> {
    let mut block_meshes = Vec::with_capacity(64);
    let uv = [
        [[3., 4.], [0., 1.]],
        [[3., 4.], [0., 1.]],
        [[3., 4.], [0., 1.]],
        [[3., 4.], [0., 1.]],
        [[0., 1.], [0., 1.]],
        [[2., 3.], [0., 1.]],
    ];
    let uv = [
        // top
        [uv[0][0][1] / 16., uv[0][1][1] / 16.],
        [uv[0][0][0] / 16., uv[0][1][1] / 16.],
        [uv[0][0][0] / 16., uv[0][1][0] / 16.],
        [uv[0][0][1] / 16., uv[0][1][0] / 16.],
        //bottom
        [uv[1][0][0] / 16., uv[1][1][0] / 16.],
        [uv[1][0][1] / 16., uv[1][1][0] / 16.],
        [uv[1][0][1] / 16., uv[1][1][1] / 16.],
        [uv[1][0][0] / 16., uv[1][1][1] / 16.],
        //right
        [uv[2][0][1] / 16., uv[2][1][1] / 16.],
        [uv[2][0][1] / 16., uv[2][1][0] / 16.],
        [uv[2][0][0] / 16., uv[2][1][0] / 16.],
        [uv[2][0][0] / 16., uv[2][1][1] / 16.],
        //left
        [uv[3][0][1] / 16., uv[3][1][1] / 16.],
        [uv[3][0][1] / 16., uv[3][1][0] / 16.],
        [uv[3][0][0] / 16., uv[3][1][0] / 16.],
        [uv[3][0][0] / 16., uv[3][1][1] / 16.],
        //front
        [uv[4][0][1] / 16., uv[4][1][0] / 16.],
        [uv[4][0][0] / 16., uv[4][1][0] / 16.],
        [uv[4][0][0] / 16., uv[4][1][1] / 16.],
        [uv[4][0][1] / 16., uv[4][1][1] / 16.],
        //back
        [uv[5][0][0] / 16., uv[5][1][0] / 16.],
        [uv[5][0][1] / 16., uv[5][1][0] / 16.],
        [uv[5][0][1] / 16., uv[5][1][1] / 16.],
        [uv[5][0][0] / 16., uv[5][1][1] / 16.],
    ];
    for face in 0..64 {
        let faces: [bool; 6] = [
            (face as u8 >> 5) == 1,
            (((face as u8) << 1) >> 4) == 1,
            (((face as u8) << 2) >> 3) == 1,
            (((face as u8) << 3) >> 2) == 1,
            (((face as u8) << 4) >> 1) == 1,
            ((face as u8) << 5) == 1,
        ];

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut positions = Vec::with_capacity(32 * 32 * 32); //max
        let mut normals = Vec::with_capacity(32 * 32 * 32); //max
        let mut uvs = Vec::with_capacity(32 * 32 * 32); //max
        let mut indices = Vec::with_capacity(32 * 32 * 32);
        let vertices = &[
            // Top
            ([-0.5, -0.5, 0.5], [0., 0., 1.0]),
            ([0.5, -0.5, 0.5], [0., 0., 1.0]),
            ([0.5, 0.5, 0.5], [0., 0., 1.0]),
            ([-0.5, 0.5, 0.5], [0., 0., 1.0]),
            // Bottom
            ([-0.5, 0.5, -0.5], [0., 0., -1.0]),
            ([0.5, 0.5, -0.5], [0., 0., -1.0]),
            ([0.5, -0.5, -0.5], [0., 0., -1.0]),
            ([-0.5, -0.5, -0.5], [0., 0., -1.0]),
            // Right
            ([0.5, -0.5, -0.5], [1.0, 0., 0.]),
            ([0.5, 0.5, -0.5], [1.0, 0., 0.]),
            ([0.5, 0.5, 0.5], [1.0, 0., 0.]),
            ([0.5, -0.5, 0.5], [1.0, 0., 0.]),
            // Left
            ([-0.5, -0.5, 0.5], [-1.0, 0., 0.]),
            ([-0.5, 0.5, 0.5], [-1.0, 0., 0.]),
            ([-0.5, 0.5, -0.5], [-1.0, 0., 0.]),
            ([-0.5, -0.5, -0.5], [-1.0, 0., 0.]),
            // Front
            ([0.5, 0.5, -0.5], [0., 1.0, 0.]),
            ([-0.5, 0.5, -0.5], [0., 1.0, 0.]),
            ([-0.5, 0.5, 0.5], [0., 1.0, 0.]),
            ([0.5, 0.5, 0.5], [0., 1.0, 0.]),
            // Back
            ([0.5, -0.5, 0.5], [0., -1.0, 0.]),
            ([-0.5, -0.5, 0.5], [0., -1.0, 0.]),
            ([-0.5, -0.5, -0.5], [0., -1.0, 0.]),
            ([0.5, -0.5, -0.5], [0., -1.0, 0.]),
        ];
        let mut indices_table = [
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];
        for (index, (position, normal)) in vertices.iter().enumerate() {
            let position = [
                position[0] as f32,
                position[1] as f32,
                position[2] as f32,
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
                let temp = &mut indices_table[x * 6..(x * 6) + 6];
                for u in temp.iter_mut() {
                    *u += (0 * 24) as u32;
                }
                indices.extend_from_slice(temp);
            } else {
                indices.extend_from_slice(&[0, 0, 0, 0, 0, 0]);
            }
        }
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.set_indices(Some(Indices::U32(indices)));
        block_meshes.push(mesh);
    }
    info!("Done generating block meshes!");
    block_meshes
}

fn generate_chunk_data(
    mut query: &Query<&ChunkInfo>,
    chunk: &Chunk,
    mut block_meshes: &ResMut<BlockMeshes>,
) -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u32>) {
    let mut positions = Vec::with_capacity(32 * 32 * 32); //max
    let mut normals = Vec::with_capacity(32 * 32 * 32); //max
    let mut uvs = Vec::with_capacity(32 * 32 * 32); //max
    let mut indices = Vec::with_capacity(32 * 32 * 32);

    info!("generating mesh with xyz: {} {} {}",chunk.x, chunk.y, chunk.z);

    chunk.blocks.iter().enumerate().for_each(|(i, b)| {
        if let Some(b) = b {
            let (block_x, block_y, block_z) = Chunk::index_to_coords(&i);
            let faces: [bool; 6] = get_faces(
                query,
                chunk,
                &block_x,
                &block_y,
                &block_z,
            );

            let faces_value: u8 =
                ((faces[0] as u8) << 5) +
                    ((faces[1] as u8) << 4) +
                    ((faces[2] as u8) << 3) +
                    ((faces[3] as u8) << 2) +
                    ((faces[4] as u8) << 1) +
                    ((faces[5] as u8));
            let mesh = &block_meshes.0[faces_value as usize];
            // let normal:[f32;3] = <[f32; 3]>::try_from(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).unwrap().get_bytes()).unwrap();
            // normals.push(normal);
            // info!("Value: {}",faces_value);
        }
    });
    (positions, normals, uvs, indices)
}

fn get_faces(
    mut query: &Query<&ChunkInfo>,
    c: &Chunk,
    b_x: &u16,
    b_y: &u16,
    b_z: &u16,
) -> [bool; 6] {
    let (w_x, w_y, w_z) = convert_to_world_coords(&c.x, &c.y, &c.z, b_x, b_y, b_z);
    // info!("world xyz: {} {} {}",w_x,w_y,w_z);
    let top = {
        let (w_x, w_y, w_z) = (w_x, w_y, w_z + 1);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(&w_x, &w_y, &w_z);
        if (c_x == c.x && c_y == c.y && c_z == c.z) {
            let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
            !(c.is_block(&b_x, &b_y, &b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(&b_x, &b_y, &b_z))
            } else {
                true
            }
        }
    };
    let bottom = {
        let (w_x, w_y, w_z) = (w_x, w_y, w_z - 1);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(&w_x, &w_y, &w_z);
        if (c_x == c.x && c_y == c.y && c_z == c.z) {
            let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
            !(c.is_block(&b_x, &b_y, &b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(&b_x, &b_y, &b_z))
            } else {
                true
            }
        }
    };
    let right = {
        let (w_x, w_y, w_z) = (w_x + 1, w_y, w_z);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(&w_x, &w_y, &w_z);
        if (c_x == c.x && c_y == c.y && c_z == c.z) {
            let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
            !(c.is_block(&b_x, &b_y, &b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(&b_x, &b_y, &b_z))
            } else {
                true
            }
        }
    };
    let left = {
        let (w_x, w_y, w_z) = (w_x - 1, w_y, w_z);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(&w_x, &w_y, &w_z);
        if (c_x == c.x && c_y == c.y && c_z == c.z) {
            let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
            !(c.is_block(&b_x, &b_y, &b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(&b_x, &b_y, &b_z))
            } else {
                true
            }
        }
    };
    let front = {
        let (w_x, w_y, w_z) = (w_x, w_y + 1, w_z);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(&w_x, &w_y, &w_z);
        if (c_x == c.x && c_y == c.y && c_z == c.z) {
            let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
            !(c.is_block(&b_x, &b_y, &b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(&b_x, &b_y, &b_z))
            } else {
                true
            }
        }
    };
    let back = {
        let (w_x, w_y, w_z) = (w_x, w_y - 1, w_z);
        let (c_x, c_y, c_z) = convert_to_chunk_coords(&w_x, &w_y, &w_z);
        if (c_x == c.x && c_y == c.y && c_z == c.z) {
            let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
            !(c.is_block(&b_x, &b_y, &b_z))
        } else {
            if let Some(c) = get_chunk_from_coords(query, c_x, c_y, c_z) {
                let (b_x, b_y, b_z) = convert_to_block_coords(&w_x, &w_y, &w_z);
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.0.is_block(&b_x, &b_y, &b_z))
            } else {
                true
            }
        }
    };
    [top, bottom, right, left, front, back]
}

fn block_coords_in_chunk(x: isize, y: isize, z: isize) -> (u16, u16, u16) {
    (
        (x % 32) as u16,
        (y % (32 * 32 * 32)) as u16,
        (z % (32 * 32)) as u16,
    )
}

fn get_chunk_from_coords(mut query: &Query<&ChunkInfo>, x: i16, y: i16, z: i16) -> Option<ChunkInfo> {
    let index = chunk_coords_to_index(x, y, z) as isize;
    if (0..(32 * 32 * 32)).contains(&index) {
        for c in query.iter() {
            if c.0.x == x && c.0.y == y && c.0.z == z {
                return Some(c.clone());
            }
        }
    }
    None
}

fn chunk_coords_to_index(x: i16, y: i16, z: i16) -> usize {
    let x1 = x; // incremental is the same
    let y1 = y * 32 * 32; // incremental is 1:32*32
    let z1 = z * 32; // incremental is 1:32
    // info!("chunk coord to index: {:?} {}",(x,y,z),i);
    ((x1 + y1 + z1) as isize + (CHUNK_SIZE / 2) as isize) as usize
}

fn convert_to_world_coords(
    c_x: &i16,
    c_y: &i16,
    c_z: &i16,
    b_x: &u16,
    b_y: &u16,
    b_z: &u16,
) -> (isize, isize, isize) {
    (
        (c_x * 32) as isize + (*b_x as isize),
        (c_y * 32) as isize + (*b_y as isize),
        (c_z * 32) as isize + (*b_z as isize),
    )
}

fn convert_to_block_coords(w_x: &isize, w_y: &isize, w_z: &isize) -> (u16, u16, u16) {
    (
        if *w_x < 0 { 32 + (w_x % 32) } else { w_x % 32 } as u16,
        if *w_y < 0 { 32 + (w_y % 32) } else { w_y % 32 } as u16,
        if *w_z < 0 { 32 + (w_z % 32) } else { w_z % 32 } as u16,
    )
}

fn convert_to_chunk_coords(w_x: &isize, w_y: &isize, w_z: &isize) -> (i16, i16, i16) {
    (
        if *w_x < 0 { (w_x / 32) - 1 } else { w_x / 32 } as i16,
        if *w_y < 0 { (w_y / 32) - 1 } else { w_y / 32 } as i16,
        if *w_z < 0 { (w_z / 32) - 1 } else { w_z / 32 } as i16,
    )
}
