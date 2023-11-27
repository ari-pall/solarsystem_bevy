#![allow(unused_imports)]
#![allow(dead_code)]

use {bevy::{input::mouse::MouseMotion, prelude::*},
     rand::{distributions::uniform::{SampleRange, SampleUniform},
            prelude::*},
     rust_utils::dotimes};

// const MOVE_SPEED: f32 = 0.16;
// const LOOK_SPEED: f32 = 0.002;

fn vec3(x: f32, y: f32, z: f32) -> Vec3 { Vec3 { x, y, z } }
fn vec3_from_spherical_coords(yaw: f32, pitch: f32) -> Vec3 {
  vec3(yaw.cos() * pitch.cos(),
       pitch.sin(),
       yaw.sin() * pitch.cos()).normalize()
}
#[derive(Clone, Component)]
struct Planet {
  vel: Vec3,
  color: Color,
  mass: f32
}
fn rng<T: SampleUniform>(range: impl SampleRange<T>) -> T {
  rand::thread_rng().gen_range(range)
}
impl Planet {
  fn star() -> Planet {
    Planet { vel: default(),
             color: Color::rgb(255.0, 255.0, 255.0),
             mass: 1.1 }
  }
  fn radius(&self) -> f32 { self.mass.cbrt() }
}
fn movement(mut q: Query<(&mut Transform, &Planet)>) {
  q.for_each_mut(|(mut t, p)| {
     t.translation += p.vel;
   });
}
fn gravity(mut q: Query<(Entity, &Transform, &mut Planet)>) {
  let mut pairs = q.iter_combinations_mut();
  while let Some([(e1, t1, mut p1), (e2, t2, mut p2)]) = pairs.fetch_next() {
    if e1.index() < e2.index() {
      let posdiff = t2.translation - t1.translation;
      let dist = posdiff.length();
      let g = 0.007;
      let k = g * posdiff / dist.powi(3);
      p1.vel += k * p2.mass;
      p2.vel -= k * p1.mass;
    }
  }
}
fn collisions(mut q: Query<(Entity, &mut Transform, &mut Planet)>, mut c: Commands) {
  let mut pairs = q.iter_combinations_mut();
  while let Some([(e1, mut t1, mut p1), (e2, t2, p2)]) = pairs.fetch_next() {
    if e1.index() < e2.index()
       && t1.translation.distance(t2.translation) < p1.radius() + p2.radius()
    {
      let total_mass = p1.mass + p2.mass;
      t1.translation = (t1.translation * p1.mass + t2.translation * p2.mass) / total_mass;
      *p1 = Planet { vel: (p1.vel * p1.mass + p2.vel * p2.mass) / total_mass,
                     color: p1.color,
                     mass: total_mass };
      t1.scale = Vec3::ONE * p1.radius();
      c.entity(e2).despawn();
    }
  }
}
const NUM_PLANETS: usize = 150;
fn init(mut c: Commands,
        mut mats: ResMut<Assets<StandardMaterial>>,
        mut windows: Query<&mut Window>,
        mut meshes: ResMut<Assets<Mesh>>) {
  windows.single_mut().set_maximized(true);
  c.spawn(Camera3dBundle { transform:
                            Transform::from_xyz(-60.0, 0.0, 0.0).looking_at(Vec3::Y, Vec3::Z),
                          ..default() });
  let sphere = meshes.add(shape::Icosphere::default().try_into().unwrap());
  dotimes! {NUM_PLANETS,{
    let r = rng(0.01..0.99);
    let g = rng(0.01..0.99);
    let b = rng(0.01..0.99);
    let color = Color::rgb(r, g, b);
    let mass = (rng(0.0002..0.6) as f32).powi(2);
    let speed = 0.03;
    c.spawn((
      Planet { color ,
               vel: vec3(rng(-speed..speed),rng(-speed..speed), rng(-speed..speed)),
               mass },
      PbrBundle{ mesh: sphere.clone(),
                 material: mats.add(StandardMaterial{base_color:color,
                                                     emissive: color,
                                                     perceptual_roughness:0.5,
                                                     ..default() }) ,
                 transform: Transform::from_translation(
                   vec3(r, g, b) * 80.0 - Vec3::ONE * 40.0) ,..default()}
    ));
  }}
}
fn camera_movement(mut camera: Query<&mut Transform, With<Camera>>,
                   keyboard_input: Res<Input<KeyCode>>,
                   mut er: EventReader<MouseMotion>) {
  let dir = [(KeyCode::D, Vec3::X),
             (KeyCode::A, Vec3::NEG_X),
             (KeyCode::W, Vec3::NEG_Z),
             (KeyCode::S, Vec3::Z),
             (KeyCode::ShiftLeft, Vec3::Y),
             (KeyCode::ControlLeft, Vec3::NEG_Y)].into_iter()
                                                 .filter_map(|(k, v)| {
                                                   keyboard_input.pressed(k).then_some(v)
                                                 })
                                                 .sum::<Vec3>()
                                                 .normalize_or_zero();
  if let Ok(mut t) = camera.get_single_mut() {
    for &MouseMotion { delta: Vec2 { x, y } } in er.read() {
      let rot_scale = 0.004;
      t.rotate_local_x(-y * rot_scale);
      t.rotate_local_y(-x * rot_scale);
    }
    let d = t.rotation * dir * 0.6;
    t.translation += d;
  }
}
#[bevy_main]
fn main() {
  App::new().add_plugins((DefaultPlugins))
            .insert_resource(ClearColor(Color::BLACK))
            .add_systems(Startup, init)
            .add_systems(Update,
                         (movement,
                          gravity,
                          collisions,
                          bevy::window::close_on_esc,
                          camera_movement))
            .run();
}
