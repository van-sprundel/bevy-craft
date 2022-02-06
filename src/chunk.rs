use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use crate::block::Block;

const CHUNK_SIZE: usize = 32 * 32 * 32;

#[derive(Component)]
pub struct ChunkGrid(pub Box<[Option<Chunk>; CHUNK_SIZE]>);

impl ChunkGrid {
    pub fn new() -> Self {
        Self(Box::new([Chunk::EMPTY; CHUNK_SIZE]))
    }
    pub fn set_chunk(&mut self, chunk: Chunk) {
        let x = chunk.x as isize;
        let y = chunk.y as isize;
        let z = chunk.z as isize;
        let index = Self::chunk_coords_to_index(x, y, z);
        info!("setting chunk at xyz: {}{}{} i: {}",x,y,z,index);
        self.0[index] = Some(chunk);
    }
    pub fn chunk_index_to_coords(index: usize) -> (usize, usize, usize) {
        let index = index - (CHUNK_SIZE / 2);
        let x = (index % 32) * 32; // 0..32 then resets to 0
        let y = (index / (32 * 32)) * 32; // 0..1 is equal to a 32 * 32 block area
        let z = ((index / 32) % 32) * 32;
        // 0..1 is equal to a 32 block area
        // info!("chunk index to coords: {} {:?}",index,(x,y,z));
        (x, y, z)
    }
    pub fn chunk_coords_to_index(x: isize, y: isize, z: isize) -> usize {
        let x1 = x; // incremental is the same
        let y1 = y * 32 * 32; // incremental is 1:32*32
        let z1 = z * 32;         // incremental is 1:32
        // info!("chunk coord to index: {:?} {}",(x,y,z),i);
        ((x1 + y1 + z1) + (CHUNK_SIZE / 2) as isize) as usize
    }
    pub fn get_faces(&self, c_x: isize, c_y: isize, c_z: isize, b_x: usize, b_y: usize, b_z: usize) -> [bool; 6] {
        let (w_x, w_y, w_z) = Self::convert_to_world_coords(c_x, c_y, c_z, b_x, b_y, b_z);
        // info!("world xyz: {} {} {}",w_x,w_y,w_z);
        let top = {
            let (w_x, w_y, w_z) = (w_x, w_y, w_z + 1);
            let (c_x, c_y, c_z) = Self::convert_to_chunk_coords(w_x, w_y, w_z);
            let (b_x, b_y, b_z) = Self::convert_to_block_coords(w_x, w_y, w_z);
            if let Some(c) = self.get_chunk_from_coords(c_x, c_y, c_z) {
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        };
        let bottom = {
            let (w_x, w_y, w_z) = (w_x, w_y, w_z - 1);
            let (c_x, c_y, c_z) = Self::convert_to_chunk_coords(w_x, w_y, w_z);
            let (b_x, b_y, b_z) = Self::convert_to_block_coords(w_x, w_y, w_z);
            if let Some(c) = self.get_chunk_from_coords(c_x, c_y, c_z) {
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        };
        let right = {
            let (w_x, w_y, w_z) = (w_x + 1, w_y, w_z);
            let (c_x, c_y, c_z) = Self::convert_to_chunk_coords(w_x, w_y, w_z);
            let (b_x, b_y, b_z) = Self::convert_to_block_coords(w_x, w_y, w_z);
            if let Some(c) = self.get_chunk_from_coords(c_x, c_y, c_z) {
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        };
        let left = {
            let (w_x, w_y, w_z) = (w_x - 1, w_y, w_z);
            let (c_x, c_y, c_z) = Self::convert_to_chunk_coords(w_x, w_y, w_z);
            let (b_x, b_y, b_z) = Self::convert_to_block_coords(w_x, w_y, w_z);
            if let Some(c) = self.get_chunk_from_coords(c_x, c_y, c_z) {
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        };
        let front = {
            let (w_x, w_y, w_z) = (w_x, w_y + 1, w_z);
            let (c_x, c_y, c_z) = Self::convert_to_chunk_coords(w_x, w_y, w_z);
            let (b_x, b_y, b_z) = Self::convert_to_block_coords(w_x, w_y, w_z);
            if let Some(c) = self.get_chunk_from_coords(c_x, c_y, c_z) {
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        };
        let back = {
            let (w_x, w_y, w_z) = (w_x, w_y - 1, w_z);
            let (c_x, c_y, c_z) = Self::convert_to_chunk_coords(w_x, w_y, w_z);
            let (b_x, b_y, b_z) = Self::convert_to_block_coords(w_x, w_y, w_z);
            if let Some(c) = self.get_chunk_from_coords(c_x, c_y, c_z) {
                // info!("block xyz: {} {} {}",b_x,b_y,b_z);
                !(c.is_block(b_x, b_y, b_z))
            } else {
                true
            }
        };
        [top, bottom, right, left, front, back]
    }
    pub fn generate_mesh(&self, chunk_x: isize, chunk_y: isize, chunk_z: isize) -> Mesh {
        let mut positions = Vec::with_capacity(32 * 32 * 32); //max
        let mut normals = Vec::with_capacity(32 * 32 * 32); //max
        let mut uvs = Vec::with_capacity(32 * 32 * 32); //max
        let mut indices = Vec::with_capacity(32 * 32 * 32);
        info!("generating mesh with xyz: {} {} {}",chunk_x,chunk_y,chunk_z);
        if let Some(chunk) = self.get_chunk_from_coords(chunk_x, chunk_y, chunk_z) {
            chunk.blocks.iter().enumerate().for_each(|(i, b)| {
                match b {
                    None => {}
                    Some(b) => {
                        let (block_x, block_y, block_z) = Chunk::index_to_coords(i);
                        let faces: [bool; 6] = self.get_faces(chunk_x, chunk_y, chunk_z, block_x, block_y, block_z);
                        // info!("faces: {:?}",faces);
                        let uv = b.get_texture_uv();

                        for (index, (position, normal)) in VERTICES.iter().enumerate() {
                            let (x, y, z) = Chunk::index_to_coords(i);
                            let position = [
                                position[0] + (x as isize + (32 * chunk_x)) as f32,
                                position[1] + (y as isize + (32 * chunk_y)) as f32,
                                position[2] + (z as isize + (32 * chunk_z)) as f32];
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
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
            mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
            mesh.set_indices(Some(Indices::U32(indices)));
            return mesh;
        }

        return panic!();
    }
    pub fn block_coords_in_chunk(x: isize, y: isize, z: isize) -> (usize, usize, usize) {
        (
            (x % 32) as usize,
            (y % (32 * 32 * 32)) as usize,
            (z % (32 * 32)) as usize
        )
    }
    pub fn get_chunk_from_coords(&self, x: isize, y: isize, z: isize) -> &Option<Chunk> {
        let index = Self::chunk_coords_to_index(x, y, z) as isize;
        if (0..(32 * 32 * 32)).contains(&index) {
            return &self.0[index as usize];
        }

        &None
    }
    fn convert_to_world_coords(c_x: isize, c_y: isize, c_z: isize, b_x: usize, b_y: usize, b_z: usize) -> (isize, isize, isize) {
        ((c_x * 32) + b_x as isize, (c_y * 32) + b_y as isize, (c_z * 32) + b_z as isize)
    }
    fn convert_to_block_coords(w_x: isize, w_y: isize, w_z: isize) -> (usize, usize, usize) {
        (
            if w_x < 0 { 32 + (w_x % 32) } else { w_x % 32 } as usize,
            if w_y < 0 { 32 + (w_y % 32) } else { w_y % 32 } as usize,
            if w_z < 0 { 32 + (w_z % 32) } else { w_z % 32 } as usize
        )
    }
    fn convert_to_chunk_coords(w_x: isize, w_y: isize, w_z: isize) -> (isize, isize, isize) {
        (
            if w_x < 0 { (w_x / 32) - 1 } else { w_x / 32 },
            if w_y < 0 { (w_y / 32) - 1 } else { w_y / 32 },
            if w_z < 0 { (w_z / 32) - 1 } else { w_z / 32 },
        )
    }
}

impl Default for ChunkGrid {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub blocks: Box<[Option<Block>; CHUNK_SIZE]>,
    pub spawned: bool,
}

impl Chunk {
    pub fn new(x: i16, y: i16, z: i16) -> Self {
        Self {
            blocks: Box::new([Block::EMPTY; 32 * 32 * 32]),
            spawned: false,
            x,
            y,
            z,
        }
    }
    pub fn set_block(&mut self, block: Block, x: usize, y: usize, z: usize) {
        self.blocks[Self::coords_to_index(x as u16, y as u16, z as u16)] = Some(block);
    }
    pub fn get_faces(&self, block_index: usize) -> [bool; 6] {
        let (x, y, z) = Self::index_to_coords(block_index);
        [
            self.is_block(x, y, z.wrapping_add(1)), // top
            self.is_block(x, y, z.wrapping_sub(1)), // bottom
            self.is_block(x.wrapping_add(1), y, z), // right
            self.is_block(x.wrapping_sub(1), y, z), // left
            self.is_block(x, y.wrapping_add(1), z), // front
            self.is_block(x, y.wrapping_sub(1), z), // back
        ]
    }
    pub fn get_block(&self, x: usize, y: usize, z: usize) -> &Option<Block> {
        if (0..32).contains(&(x as i32)) && (0..32).contains(&(y as i32)) && (0..32).contains(&(z as i32)) {
            // not even allowed to go beyond u16 size
            &self.blocks[Self::coords_to_index(x as u16, y as u16, z as u16)]
        } else {
            &None
        }
    }
    pub fn is_block(&self, x: usize, y: usize, z: usize) -> bool {
        if (0..32).contains(&(x as i32)) && (0..32).contains(&(y as i32)) && (0..32).contains(&(z as i32)) {
            // not even allowed to go beyond u16 size
            self.blocks[Self::coords_to_index(x as u16, y as u16, z as u16)] != None
        } else {
            false
        }
    }
    pub fn index_to_coords(index: usize) -> (usize, usize, usize) {
        let x = index % 32; // 0..32 then resets to 0
        let y = index / (32 * 32); // 0..1 is equal to a 32 * 32 block area
        let z = (index / 32) % 32; // 0..1 is equal to a 32 block area
        (x, y, z)
    }
    pub fn coords_to_index(x: u16, y: u16, z: u16) -> usize {
        let x = x; // incremental is the same
        let y = y * 32 * 32; // incremental is 1:32*32
        let z = z * 32; // incremental is 1:32
        (x + y + z) as usize
    }

    const EMPTY: Option<Chunk> = None;
}

const INDICES: [u32; 36] = [
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 10, 11, 8, // right
    12, 13, 14, 14, 15, 12, // left
    16, 17, 18, 18, 19, 16, // front
    20, 21, 22, 22, 23, 20, // back
];

const VERTICES: &[([f32; 3], [f32; 3]); 24] = &[
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

#[cfg(test)]
mod tests {
    use std::mem::size_of_val;
    use std::time::Instant;

    use super::*;

    #[test]
    fn block_size() {
        struct Block;
        println!("Size struct: {} bits", size_of_val(&Block {}));

        struct Block1(u16);
        println!("Size struct with u16: {} bits", size_of_val(&Block1(42)));

        #[derive(Default)]
        struct Block2 {
            index1: u16,
            index2: u16,
            index3: u16,
        }
        println!("Size struct with 3 fields: {} bits", size_of_val(&Block2::default()));
    }

    #[test]
    fn chunk_size() {
        const SIZE: usize = 32 * 32 * 32;
        let x = [42; SIZE];
        println!("Size default: {} bits", size_of_val(&x));

        let y: Vec<i32> = vec![42; SIZE];
        println!("Size with vec: {} bits", size_of_val(&y));

        let y = vec![42; SIZE].into_boxed_slice();
        println!("Size with box from vec: {} bits", size_of_val(&y));

        let y = Box::new(x);
        println!("Size with box from array: {} bits", size_of_val(&y));

        let x = [Some(42); SIZE];
        let y = Box::new(x);
        println!("Size with optional box from array: {} bits", size_of_val(&y));

        let x = [&Some(42); SIZE];
        let y = Box::new(x);
        println!("Size with referenced optional box from array: {} bits", size_of_val(&y));

        let x = [Some(&42); SIZE];
        let y = Box::new(x);
        println!("Size with optional box from array with reference: {} bits", size_of_val(&y));
    }

    #[test]
    fn access_speed() {
        const SIZE: usize = 32 * 32 * 32;
        let x = [42; SIZE];
        let time = Instant::now();
        for _ in 0..100 {
            for i in 0..SIZE {
                x[i];
            }
        }
        println!("Default elapsed: {}μs", time.elapsed().as_micros() / 100);

        let y: Vec<i32> = vec![42; SIZE];
        let time = Instant::now();
        for _ in 0..100 {
            for i in 0..SIZE {
                x[i];
            }
        }
        println!("Vec elapsed: {}μs", time.elapsed().as_micros() / 100);

        let y = vec![42; SIZE].into_boxed_slice();
        let time = Instant::now();
        for _ in 0..100 {
            for i in 0..SIZE {
                x[i];
            }
        }
        println!("Boxed slice elapsed: {}μs", time.elapsed().as_micros() / 100);

        let y = Box::new(x);
        let time = Instant::now();
        for _ in 0..100 {
            for i in 0..SIZE {
                x[i];
            }
        }
        println!("Boxed array elapsed: {}μs", time.elapsed().as_micros() / 100);

        let x = [Some(42); SIZE];
        let y = Box::new(x);
        let time = Instant::now();
        for i in 0..SIZE {
            y[i];
        }
        println!("Optional boxed array elapsed: {}μs", time.elapsed().as_micros());

        let x = [&Some(42); SIZE];
        let y = Box::new(x);
        let time = Instant::now();
        for _ in 0..100 {
            for i in 0..SIZE {
                x[i];
            }
        }
        println!("Optional reference boxed array elapsed: {}μs", time.elapsed().as_micros() / 100);

        let x = [Some(&42); SIZE];
        let y = Box::new(x);
        let time = Instant::now();
        for _ in 0..100 {
            for i in 0..SIZE {
                x[i];
            }
        }
        println!("Optional boxed array with reference elapsed: {}μs", time.elapsed().as_micros() / 100);
    }

    #[test]
    fn chunk_calculations() {
        assert_eq!(0, Chunk::coords_to_index(0, 0, 0));
        assert_eq!(1, Chunk::coords_to_index(1, 0, 0));
        assert_eq!(32, Chunk::coords_to_index(0, 0, 1));
        assert_eq!(1024, Chunk::coords_to_index(0, 1, 0));
        assert_eq!(1025, Chunk::coords_to_index(1, 1, 0));

        assert_eq!((0, 0, 0), Chunk::index_to_coords(0));
        assert_eq!((1, 0, 0), Chunk::index_to_coords(1));
        assert_eq!((0, 0, 1), Chunk::index_to_coords(32));
        assert_eq!((0, 1, 0), Chunk::index_to_coords(1024));
        assert_eq!((1, 1, 0), Chunk::index_to_coords(1025));

        let mut c = Chunk::new(0, 0, 0);

        c.set_block(Block(0), 1, 2, 3);
        assert_ne!(None, *c.get_block(1, 2, 3));
        assert_eq!(Some(Block(0)), *c.get_block(1, 2, 3));
        assert_eq!(2145, Chunk::coords_to_index(1, 2, 3));
        assert_eq!((1, 2, 3), Chunk::index_to_coords(2145));

        let mut c = Chunk::new(0, 0, 0);

        c.set_block(Block(0), 31, 30, 29);
        assert_ne!(None, *c.get_block(31, 30, 29));
        assert_eq!(Some(Block(0)), *c.get_block(31, 30, 29));
        assert_eq!(31679, Chunk::coords_to_index(31, 30, 29));
        assert_eq!((31, 30, 29), Chunk::index_to_coords(31679));
    }

    #[test]
    fn chunkgrid_calculations() {
        // assert_eq!(0, ChunkGrid::coords_to_index(0, 0, 0));
        // assert_eq!(1, ChunkGrid::coords_to_index(1, 0, 0));
        // assert_eq!(32, ChunkGrid::coords_to_index(0, 0, 1));
        // assert_eq!(1024, ChunkGrid::coords_to_index(0, 1, 0));
        // assert_eq!(1025, ChunkGrid::coords_to_index(1, 1, 0));

        assert_eq!((0, 0, 0), ChunkGrid::chunk_index_to_coords(0));
        assert_eq!((32, 0, 0), ChunkGrid::chunk_index_to_coords(1));
        assert_eq!((0, 0, 32), ChunkGrid::chunk_index_to_coords(32));
        assert_eq!((0, 32, 0), ChunkGrid::chunk_index_to_coords(1024));
        assert_eq!((32, 32, 0), ChunkGrid::chunk_index_to_coords(1025));
    }
}
