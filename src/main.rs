use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use bevy_egui::{egui::Color32, prelude::*};
use bevy_rapier2d::prelude::*;
use rand::RngExt;

const BALL_RADIUS: f32 = 10.;
const BALL_RESTITUTION: Restitution = Restitution::coefficient(0.7);

const PEG_RADIUS: f32 = 5.;
const PEG_RESTITUTION: Restitution = Restitution::coefficient(0.5);

const PEG_HORIZONTAL_SPACING: f32 = 50.;
const PEG_VERTICAL_SPACING: f32 = 50.;
const PEG_SPAWN_JITTER: f32 = 0.05;

const PEG_CENTER_SPACING: f32 = 2. * PEG_RADIUS + PEG_HORIZONTAL_SPACING;
const PEG_LAYER_SPACING: f32 = 2. * PEG_RADIUS + PEG_VERTICAL_SPACING;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1000.))
        .add_plugins(RapierDebugRenderPlugin::default())
        .init_resource::<RedrawBoard>()
        .init_resource::<PegLayers>()
        .init_resource::<NumberOfBalls>()
        .add_systems(Startup, (setup_camera_system, setup_physics, setup_board))
        .add_systems(EguiPrimaryContextPass, ui_system)
        .add_systems(
            Update,
            ((destroy_board, setup_board)
                .chain()
                .run_if(resource_equals::<RedrawBoard>(RedrawBoard(true))),),
        )
        .run();
}

fn setup_camera_system(mut commands: Commands) {
    commands.spawn((Camera2d, Transform::from_scale(Vec3::ONE * 3.)));
}

fn setup_physics(mut commands: Commands) {
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(BALL_RADIUS))
        .insert(Ccd::enabled())
        .insert(BALL_RESTITUTION)
        .insert(Transform::from_xyz(0.0, 400.0, 0.0));
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(BALL_RADIUS))
        .insert(Ccd::enabled())
        .insert(BALL_RESTITUTION)
        .insert(Transform::from_xyz(0.0, 420.0, 0.0));
    commands
        .spawn(RigidBody::Dynamic)
        .insert(Collider::ball(BALL_RADIUS))
        .insert(Ccd::enabled())
        .insert(BALL_RESTITUTION)
        .insert(Transform::from_xyz(0.0, 440.0, 0.0));
}

#[derive(Resource, Default, PartialEq, Eq)]
struct RedrawBoard(bool);

#[derive(Resource)]
struct PegLayers(u8);

impl Default for PegLayers {
    fn default() -> Self {
        Self(7)
    }
}

#[derive(Resource)]
struct NumberOfBalls(u32);

impl Default for NumberOfBalls {
    fn default() -> Self {
        Self(1000)
    }
}

fn ui_system(
    mut redraw_board: ResMut<RedrawBoard>,
    mut peg_layers: ResMut<PegLayers>,
    mut number_of_balls: ResMut<NumberOfBalls>,
    mut contexts: EguiContexts,
    mut camera_transform: Single<&mut Transform, With<Camera>>,
) {
    if let Ok(ctx) = contexts.ctx_mut() {
        egui::SidePanel::left("left_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Galton board")
                        .color(Color32::WHITE)
                        .heading(),
                );
                egui::Grid::new("settings").show(ui, |ui| {
                    // peg layers
                    ui.label("Number of peg layers");
                    redraw_board.0 = ui
                        .add(egui::Slider::new(&mut peg_layers.0, 3..=30))
                        .changed();
                    ui.end_row();

                    // number of balls
                    ui.label("Number of balls");
                    ui.horizontal(|ui| {
                        if ui.button("-100").clicked() {
                            number_of_balls.0 = number_of_balls.0.saturating_sub(100);
                        }
                        if ui.button("-10").clicked() {
                            number_of_balls.0 = number_of_balls.0.saturating_sub(10);
                        }
                        if ui.button("-1").clicked() {
                            number_of_balls.0 = number_of_balls.0.saturating_sub(1);
                        }
                        ui.add(egui::DragValue::new(&mut number_of_balls.0).speed(1));
                        if ui.button("+1").clicked() {
                            number_of_balls.0 = number_of_balls.0.saturating_add(1);
                        }
                        if ui.button("+10").clicked() {
                            number_of_balls.0 = number_of_balls.0.saturating_add(10);
                        }
                        if ui.button("+100").clicked() {
                            number_of_balls.0 = number_of_balls.0.saturating_add(100);
                        }
                    });
                    ui.end_row();
                });

                // Move the camera off so its centered in the area remaining from the side panel
                camera_transform.translation.x =
                    ui.available_width() * camera_transform.scale.x / -2.;
            });
    }
}

#[derive(Component)]
struct Board;

fn setup_board(mut commands: Commands, peg_layers: Res<PegLayers>) {
    let mut rng = rand::rng();

    commands
        .spawn((Board, Transform::default()))
        .with_children(|parent| {
            let mut horizontal_offset_base = 0.;

            // Spawn pegs
            for layer in 0..peg_layers.0 {
                for i in 0..=layer {
                    let x_jitter = rng.random_range(-PEG_SPAWN_JITTER..=PEG_SPAWN_JITTER);
                    let y_jitter = rng.random_range(-PEG_SPAWN_JITTER..=PEG_SPAWN_JITTER);
                    parent.spawn((
                        Collider::ball(PEG_RADIUS),
                        PEG_RESTITUTION,
                        Ccd::enabled(),
                        Transform::from_xyz(
                            x_jitter + horizontal_offset_base + (i as f32) * PEG_CENTER_SPACING,
                            y_jitter - (layer as f32) * PEG_LAYER_SPACING,
                            0.,
                        ),
                    ));
                }
                horizontal_offset_base -= PEG_CENTER_SPACING / 2.;
            }

            // Spawn bucket walls
            horizontal_offset_base -= PEG_CENTER_SPACING / 2.;
            let bucket_wall_half_height = 200.;
            let last_layer_y = ((peg_layers.0 - 1) as f32) * PEG_LAYER_SPACING;
            let bucket_wall_center = -last_layer_y - bucket_wall_half_height;
            for i in 0..peg_layers.0 + 2 {
                parent.spawn((
                    Collider::cuboid(PEG_RADIUS * 0.5, bucket_wall_half_height),
                    Ccd::enabled(),
                    Transform::from_xyz(
                        horizontal_offset_base + (i as f32) * PEG_CENTER_SPACING,
                        bucket_wall_center,
                        0.,
                    ),
                ));
            }

            // Spawn side barriers
            let point1 = Vec2::from((horizontal_offset_base, -last_layer_y));
            let point2 = Vec2::from((-PEG_HORIZONTAL_SPACING / 2., PEG_VERTICAL_SPACING));
            let half_distance = point1.distance(point2) / 2.;
            let midpoint = point1.midpoint(point2);
            let angle = (point1 - point2).to_angle();
            let mut transform = Transform::from_translation(midpoint.extend(0.));
            transform.rotate_z(angle + FRAC_PI_2);
            parent.spawn((
                Collider::cuboid(PEG_RADIUS * 0.5, half_distance),
                PEG_RESTITUTION,
                Ccd::enabled(),
                transform,
            ));
            transform.translation.x = -transform.translation.x;
            transform.rotate_z(-2. * (angle + FRAC_PI_2));
            parent.spawn((
                Collider::cuboid(PEG_RADIUS * 0.5, half_distance),
                PEG_RESTITUTION,
                Ccd::enabled(),
                transform,
            ));
        });
}

fn destroy_board(mut commands: Commands, query: Single<Entity, With<Board>>) {
    commands.entity(*query).despawn();
}
