
#[derive(Debug, PartialEq, Clone)]
pub struct Block(pub u8);

pub enum Texture {
    Grass,
    Stone,
    Dirt,
    Plank,
    Slab,
    Brick,
    Tnt,
    Cobweb,
    Cobblestone,
    Log,
    Missing,
}

impl Block {
    pub(crate) const EMPTY: Option< Block> = None;
    pub fn new(texture: Texture) -> Self {
        // 64 textures | 6 bits
        match texture {
            Texture::Grass => Self(0),
            Texture::Stone => Self(1),
            Texture::Dirt => Self(2),
            Texture::Plank => Self(4),
            Texture::Slab => Self(5),
            Texture::Brick => Self(7),
            Texture::Tnt => Self(8),
            Texture::Cobweb => Self(11),
            Texture::Cobblestone => Self(16),
            Texture::Log => Self(20),
            Texture::Missing => Self(254),
        }
    }
    pub fn get_texture(&self) -> Texture {
        match self.0 {
            0 => Texture::Grass,
            1 => Texture::Stone,
            2 => Texture::Dirt,
            4 => Texture::Plank,
            5 => Texture::Slab,
            7 => Texture::Brick,
            8 => Texture::Tnt,
            11 => Texture::Cobweb,
            16 => Texture::Cobblestone,
            20 => Texture::Log,
            _ => Texture::Missing
        }
    }
    pub fn get_texture_uv(&self) -> [[f32; 2]; 24] {
        let uv = match self.0 {
            0 =>
                [
                    [[3., 4.], [0., 1.]],
                    [[3., 4.], [0., 1.]],
                    [[3., 4.], [0., 1.]],
                    [[3., 4.], [0., 1.]],
                    [[0., 1.], [0., 1.]],
                    [[2., 3.], [0., 1.]],
                ],
            5 =>
                [
                    [[5., 6.], [0., 1.]],
                    [[5., 6.], [0., 1.]],
                    [[5., 6.], [0., 1.]],
                    [[5., 6.], [0., 1.]],
                    [[6., 7.], [0., 1.]],
                    [[6., 7.], [0., 1.]],
                ],
            8 =>
                [
                    [[9., 8.], [0., 1.]],
                    [[9., 8.], [0., 1.]],
                    [[8., 9.], [0., 1.]],
                    [[8., 9.], [0., 1.]],
                    [[9., 10.], [0., 1.]],
                    [[10., 11.], [0., 1.]],
                ],
            20 =>
                [
                    [[4., 5.], [1., 2.]],
                    [[4., 5.], [1., 2.]],
                    [[4., 5.], [1., 2.]],
                    [[4., 5.], [1., 2.]],
                    [[5., 6.], [1., 2.]],
                    [[5., 6.], [1., 2.]],
                ],
            _ => [[[self.0 as f32, (self.0 + 1) as f32], [(self.0 / 16) as f32, ((self.0 / 16) + 1) as f32]]; 6],
        };
        [
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
        ]
    }
}



