use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin)
            .insert_resource(UiTimer(Timer::from_seconds(0.1, true)))
            .add_startup_system(infotext_system)
            .add_system(change_text_system);
    }
}

#[derive(Component)]
struct TextChanges;

fn infotext_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.spawn_bundle(UiCameraBundle::default());

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text {
                sections: vec![TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: font.clone(),
                        font_size: 14.0,
                        color: Color::WHITE,
                    },
                }],
                alignment: Default::default(),
            },
            ..Default::default()
        })
        .insert(TextChanges);
}

struct UiTimer(Timer);

fn change_text_system(
    time: Res<Time>,
    mut timer: ResMut<UiTimer>,
    diagnostics: Res<Diagnostics>,
    mut query: Query<&mut Text, With<TextChanges>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        for mut text in query.iter_mut() {
            let mut fps = 0.0;
            if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
                if let Some(fps_avg) = fps_diagnostic.average() {
                    fps = fps_avg;
                }
            }

            let mut frame_time = time.delta_seconds_f64();
            if let Some(frame_time_diagnostic) =
                diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
            {
                if let Some(frame_time_avg) = frame_time_diagnostic.average() {
                    frame_time = frame_time_avg;
                }
            }

            text.sections[0].value =
                format!("{:.1} fps \n{:.3} ms/frame", fps, frame_time * 1000.0,);
        }
    }
}
