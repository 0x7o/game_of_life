use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

struct GridLine {
    shape: Rectangle,
    transform: Transform,
}

struct Grid {
    data: Vec<Vec<bool>>,
    size: i32,
    thick: i32,
    w: f32,
    h: f32,
}

impl Grid {
    fn get_meshes(&self) -> Vec<GridLine> {
        let mut meshes = vec![];
        for i in 0..self.data[0].len() { // columns
            meshes.push(
                GridLine {
                    shape: Rectangle::new(self.thick as f32, self.h),
                    transform: Transform::from_xyz(
                        0.0 + i as f32 * self.size as f32,
                        0.0,
                        0.0,
                    ),
                }
            );
        }
        for i in 0..self.data.len() { // rows
            meshes.push(
                GridLine {
                    shape: Rectangle::new(self.w, self.thick as f32),
                    transform: Transform::from_xyz(
                        0.0,
                        0.0 + i as f32 * self.size as f32,
                        0.0,
                    ),
                }
            );
        }
        meshes
    }
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            clear_color: ClearColorConfig::Custom(Color::WHITE),
            ..default()
        },
        ..default()
    });
    let grid = Grid {
        data: vec![vec![false; 1000]; 1000],
        size: 15,
        thick: 2,
        w: 1000.0 * 15.0,
        h: 1000.0 * 15.0,
    };
    let grid_meshes = grid.get_meshes();
    for mesh in grid_meshes {
        commands.spawn(MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(mesh.shape)),
            material: materials.add(Color::GRAY),
            transform: mesh.transform,
            ..default()
        });
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}