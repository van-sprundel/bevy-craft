use bevy::input::mouse::MouseMotion;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy::render::options::WgpuOptions;
use bevy::render::render_resource::WgpuFeatures;
use bevy_craft_new::chunk::*;
use bevy_craft_new::debug::DebugPlugin;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WgpuOptions { features: WgpuFeatures::POLYGON_MODE_LINE, ..Default::default() })
        .insert_resource(WindowDescriptor {
            vsync: false,
            ..Default::default()
        }).add_plugins(DefaultPlugins)
        .add_plugin(WireframePlugin)
        .add_plugin(DebugPlugin)
        .init_resource::<ChunkGrid>()
        .add_state(GameState::InGame)
        .add_event::<QueueChunkEvent>()
        .add_startup_system(setup_camera)
        .add_system_set(
            SystemSet::on_enter(GameState::InGame)
                .with_system(hide_cursor)
        )
        .add_system_set(
            SystemSet::on_enter(GameState::Menu)
                .with_system(show_cursor)
        )
        .add_system_set(
            SystemSet::on_update(GameState::InGame)
                .with_system(move_camera)
                .with_system(rotate_camera)
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

fn hide_cursor(
    mut windows: ResMut<Windows>,
) {
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

fn switch_menu(
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>) {
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

fn temp_chunk_spawn(mut chunk_grid: ResMut<ChunkGrid>) {
    for a in 0..2 {
        for c in 0..2 {
            for b in 0..4 {
                let mut chunk = Chunk::new(a, b, c);
                for x in 0..32 {
                    for y in 0..32 {
                        for z in 0..32 {
                            // if (x+y+z) %2 == 0 {
                                chunk.set_block(Block::new(Texture::Dirt), x, y, z);
                            // }
                        }
                    }
                }
                chunk_grid.set_chunk(chunk);
            }
        }
    }
}

fn setup_camera(mut commands: Commands) {
    let mut camera = PerspectiveCameraBundle::new_3d();
    camera.transform = Transform::from_xyz(35., 2., 35.);
    commands.spawn_bundle(camera).insert(Camera::default());
}

struct QueueChunkEvent(Chunk);

fn queue_chunks(
    mut chunk_grid: ResMut<ChunkGrid>,
    mut ev_chunk_queue: EventWriter<QueueChunkEvent>,
) {
    chunk_grid.0.iter_mut().for_each(|c| {
        match c {
            None => {}
            Some(x) => {
                if !x.spawned {
                    x.spawned = true;
                    ev_chunk_queue.send(QueueChunkEvent(x.clone()));
                }
            }
        }
    });
}

fn spawn_chunks(
    mut wireframe_config: ResMut<WireframeConfig>,
    mut commands: Commands,
    chunk_grid: Res<ChunkGrid>,
    mut ev_chunk_queue: EventReader<QueueChunkEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    wireframe_config.global = true;
    for (i, ev) in ev_chunk_queue.iter().enumerate() {
        let c: Chunk = ev.0.clone();
        // info!("Spawning chunk");
        let mut material: StandardMaterial = match i % 10 {
            0 => Color::PINK.into(),
            1 => Color::BLUE.into(),
            2 => Color::GOLD.into(),
            3 => Color::FUCHSIA.into(),
            4 => Color::TEAL.into(),
            5 => Color::DARK_GRAY.into(),
            6 => Color::DARK_GREEN.into(),
            7 => Color::GREEN.into(),
            8 => Color::INDIGO.into(),
            9 => Color::AZURE.into(),
            _ => Color::WHITE.into()
        };
        material.unlit = true;
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(chunk_grid.generate_mesh(c.x as isize, c.y as isize, c.z as isize)),
            material: materials.add(material),
            ..Default::default()
        });
    }
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
    time: Res<Time>,
    mut query: Query<(&mut Camera, &mut Transform), With<Camera>>,
) {
    let (mut cam, mut transform) = query.single_mut();

    for x in mouse.iter() {
        // info!("{:?}",x);
        cam.yaw -= x.delta.x * cam.sens * time.delta_seconds();
        cam.pitch -= x.delta.y * cam.sens * time.delta_seconds();
    }

    cam.pitch = cam.pitch.clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

    transform.rotation = Quat::from_rotation_y(cam.yaw) * Quat::from_rotation_x(cam.pitch);
}
