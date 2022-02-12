use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(MaterialMeshBundle {
        mesh: meshes.add(shape::Cube::new(1.).into()),
        material: materials.add(Color::WHITE.into()),
        ..Default::default()
    });
    let mut cam = PerspectiveCameraBundle::new_3d();
    cam.transform = Transform::from_xyz(4., 5., 6.).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn_bundle(cam);
}
