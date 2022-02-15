#![feature(vec_retain_mut)]

use std::sync::Arc;

use bevy::input::mouse::MouseMotion;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::reflect::List;
use bevy::render::options::WgpuOptions;
use bevy::render::primitives::Plane;
use bevy::render::render_phase::Draw;
use bevy::render::render_resource::WgpuFeatures;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures_lite::future;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

use bevy_craft_new::block::{Block, Texture};
use bevy_craft_new::chunk::*;
use bevy_craft_new::debug::DebugPlugin;

lazy_static! {
    static ref CHUNK_GRID: Mutex<ChunkGrid> = {
        let c = ChunkGrid::default();
        Mutex::new(c)
    };
}
fn main() {
    App::new()
        .insert_resource(ClearColor(Color::hex("85ABFF").unwrap()))
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures::POLYGON_MODE_LINE,
            ..Default::default()
        })
        .insert_resource(WindowDescriptor {
            vsync: true,
            title:"Bevy craft".to_owned(),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPlugin)
        .add_plugin(WireframePlugin)
        .add_state(GameState::InGame)
        .add_startup_system(setup_camera)
        .add_system_set(SystemSet::on_enter(GameState::InGame).with_system(hide_cursor))
        .add_system_set(SystemSet::on_enter(GameState::Menu).with_system(show_cursor))
        .add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(move_camera)
                .with_system(rotate_camera),
        )
        .add_system(switch_menu)
        .add_startup_system(temp_chunk_spawn)
        .add_system(queue_chunks)
        .add_system(spawn_chunks)
        .run();
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    InGame,
    Menu,
}

fn hide_cursor(mut windows: ResMut<Windows>) {
    let w = windows.get_primary_mut().unwrap();
    w.set_cursor_visibility(false);
    w.set_cursor_position(Vec2::new(w.width() / 2., w.height() / 2.));
    w.set_cursor_lock_mode(true);
}

fn show_cursor(mut windows: ResMut<Windows>) {
    let w = windows.get_primary_mut().unwrap();
    w.set_cursor_visibility(true);
    w.set_cursor_lock_mode(false);
}

fn switch_menu(keyboard_input: Res<Input<KeyCode>>, mut state: ResMut<State<GameState>>) {
    keyboard_input.get_just_pressed().for_each(|x| {
        if *x == KeyCode::Escape {
            if *state.current() == GameState::Menu {
                state.set(GameState::InGame).unwrap();
            } else {
                state.set(GameState::Menu).unwrap();
            }
        }
    });
}

fn temp_chunk_spawn() {
    for a in 0..8 {
        for c in 0..8 {
            for b in 0..4 {
                let mut chunk = Chunk::new(a, b, c);
                for x in 0..32 {
                    for y in 0..32 {
                        for z in 0..32 {
                            if !(a == 0 && b == 1 && c == 0) {
                                chunk.set_block(Block::new(Texture::Log), x, y, z);
                            }
                        }
                    }
                }
                CHUNK_GRID.lock().set_chunk(chunk);
            }
        }
    }
    info!("Finished generating chunks")
}

fn setup_camera(mut commands: Commands) {
    let mut camera = PerspectiveCameraBundle::new_3d();
    camera.transform = Transform::from_xyz(50., 2., 50.);
    commands.spawn_bundle(camera).insert(Camera::default());
}

fn queue_chunks(thread_pool: Res<AsyncComputeTaskPool>) {
    let mut chunks = CHUNK_GRID.lock().chunks.to_vec();
    for (i, c) in chunks.iter_mut().enumerate() {
        if let Some(c) = c {
            if !c.spawned {
                info!("Queueing chunk {}", i);
                // let task = thread_pool.spawn(async move {
                //    c
                // });
                CHUNK_GRID.lock().add_to_queue(c.clone());
                c.spawned = true;
                CHUNK_GRID.lock().chunks[i] = Some(c.clone());
            }
        }
    }
}

fn spawn_chunks(
    mut wireframe_config: ResMut<WireframeConfig>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    // mut chunk_tasks: Query<(Entity, &mut Task<Chunk>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // wireframe_config.global = true;
    let mut queued_chunks = CHUNK_GRID.lock().queued_chunks.to_vec();
    queued_chunks.iter_mut().for_each(|chunk| {
        if !chunk.spawned {
            // if let Some(c) = future::block_on(future::poll_once(task)) {
            let texture = assets.load("TEXTURE_UV_MAP.png");
            let mut material = StandardMaterial::default();
            // material.base_color = Color::hex("78AC30").unwrap();
            material.base_color_texture = Some(texture.clone());
            material.unlit = true;

            // let mesh = CHUNK_GRID.lock().generate_chunk_mesh(chunk);
            // commands.spawn_bundle(MaterialMeshBundle {
            //     mesh: meshes.add(mesh),
            //     material: materials.add(material),
            //     ..Default::default()
            // });

            info!("Done spawning chunk!");
            // }
            chunk.spawned = true;
        }
    });
    CHUNK_GRID.lock().queued_chunks = queued_chunks;
}

#[derive(Component)]
struct Camera {
    speed: f32,
    sens: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            speed: 10.,
            sens: 0.125,
            yaw: 0.,
            pitch: 0.,
        }
    }
}

fn move_camera(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&Camera, &mut Transform), With<Camera>>,
) {
    let (cam, mut transform) = query.single_mut();

    let mut vec = Vec3::ZERO;
    let mut speed = cam.speed;
    for i in keyboard.get_pressed() {
        match i {
            KeyCode::W => vec += transform.forward(),
            KeyCode::A => vec += transform.left(),
            KeyCode::S => vec += transform.back(),
            KeyCode::D => vec += transform.right(),
            KeyCode::Space => vec += Vec3::Y,
            KeyCode::LControl => vec -= Vec3::Y,
            KeyCode::LShift => speed *= 5.,
            _ => {}
        }
    }

    vec = vec.normalize_or_zero();
    vec *= time.delta_seconds();
    vec *= speed;

    transform.translation += vec;
}

fn rotate_camera(
    mut mouse: EventReader<MouseMotion>,
    mut query: Query<(&mut Camera, &mut Transform), With<Camera>>,
) {
    let (mut cam, mut transform) = query.single_mut();

    for x in mouse.iter() {
        // info!("{:?}",x);
        cam.yaw -= x.delta.x * cam.sens * 0.04;
        cam.pitch -= x.delta.y * cam.sens * 0.04;
    }

    cam.pitch = cam
        .pitch
        .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

    transform.rotation = Quat::from_rotation_y(cam.yaw) * Quat::from_rotation_x(cam.pitch);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of_val;
    use std::time::Instant;

    #[test]
    fn startup_speed() {
        // let time = Instant::now();
        // App::new()
        //     .add_plugins(MinimalPlugins).run();
        // println!("First run elapsed: {}", time.elapsed().as_micros());
        // let iters = 100;
        // for _ in 0..iters {
        //     App::new()
        //         .add_plugins(MinimalPlugins).run();
        // }
        // println!("First run elapsed: {}", time.elapsed().as_micros() / iters);
    }
}
