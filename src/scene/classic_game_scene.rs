use conrod_core::widget::envelope_editor::EnvelopePoint;
use conrod_core::{Colorable, Positionable, Sizeable, Widget};
use std::collections::HashMap;

use std::io::{BufReader, Cursor};

use hecs::{Entity, World};
use instant::Instant;
use rapier3d::prelude::*;
use winit::event::{MouseButton, VirtualKeyCode};
use winit::event_loop::ControlFlow;

use crate::animation::InOutAnimation;
use crate::audio::{AudioContext, AUDIO_FILE_SHOOT};
use crate::audio::{Sink, AUDIO_FILE_SHOOTED};
use crate::database::Database;
use crate::entity::target::Target;

use crate::frustum::ObjectBound;
use crate::gui::ConrodHandle;
use crate::input_manager::InputManager;
use crate::physics::GamePhysics;

use crate::renderer::render_objects::{MaterialType, ShapeType};
use crate::renderer::Renderer;
use crate::scene::classic_score_scene::ClassicScoreScene;
use crate::scene::pause_scene::PauseScene;
use crate::scene::{MaybeMessage, Message, Scene, SceneOp, Value};
use crate::systems::setup_player_collider::setup_player_collider;
use crate::systems::spawn_target::spawn_target;
use crate::systems::update_player_movement::update_player_position;
use crate::timer::{Stopwatch, Timer};
use crate::util::lerp;
use crate::window::Window;
use conrod_core::widget_ids;
use nalgebra::Vector3;
use rand::distributions::Uniform;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

enum TargetSpawnState {
    Primary,
    Secondary,
}

widget_ids! {
    pub struct ClassicGameSceneIds {
        // The main canvas
        canvas,
        canvas_duration,
        duration_label,
        start_duration_label
    }
}

pub struct Score {
    pub hit: u16,
    pub miss: u16,
    pub score: i32,
    pub total_shoot_time: f32,
}

impl Score {
    pub fn new() -> Self {
        Self {
            hit: 0,
            miss: 0,
            score: 0,
            total_shoot_time: 0.0,
        }
    }

    pub fn write_message(&self, message: &mut Message) {
        message.insert("hit", Value::I32(self.hit as i32));
        message.insert("miss", Value::I32(self.miss as i32));
        message.insert("score", Value::I32(self.score));
        message.insert(
            "avg_hit_time",
            Value::F32(self.total_shoot_time / self.hit.max(1) as f32),
        );
    }
}

pub struct ClassicGameScene {
    ids: ClassicGameSceneIds,
    world: World,
    physics: GamePhysics,
    player_rigid_body_handle: RigidBodyHandle,
    game_timer: Timer,
    delta_shoot_time: Stopwatch,
    shoot_timer: Timer,
    game_start_timer: Timer,
    score: Score,
    game_running: bool,
    rng: SmallRng,
    shoot_animation: InOutAnimation,
    target_spawn_state: TargetSpawnState,
    secondary_delete_duration: Timer,
    entity_to_remove: Vec<Entity>,
}

impl ClassicGameScene {
    pub fn new(_renderer: &mut Renderer, conrod_handle: &mut ConrodHandle) -> Self {
        let mut world = World::new();
        let mut physics = GamePhysics::new();

        // Ground
        physics.collider_set.insert(
            ColliderBuilder::new(SharedShape::cuboid(999.999, 0.1, 999.999))
                .translation(Vector3::new(0.0, 0.05, 0.0))
                .build(),
        );

        let player_rigid_body_handle = setup_player_collider(&mut physics);

        spawn_target(
            &mut world,
            &mut physics,
            Vector3::new(0.0, 3.0, -10.0),
            Target::new(),
        );

        Self {
            world,
            physics,
            player_rigid_body_handle,
            ids: ClassicGameSceneIds::new(conrod_handle.get_ui_mut().widget_id_generator()),
            score: Score::new(),
            delta_shoot_time: Stopwatch::new(),
            game_timer: Timer::new(100.0),
            game_start_timer: Timer::new_finished(),
            shoot_timer: Timer::new_finished(),
            game_running: false,
            target_spawn_state: TargetSpawnState::Primary,
            rng: SmallRng::from_entropy(),
            shoot_animation: InOutAnimation::new(3.0, 5.0),
            secondary_delete_duration: Timer::new(1.0),
            entity_to_remove: Vec::new(),
        }
    }
}

impl Scene for ClassicGameScene {
    fn init(
        &mut self,
        _message: MaybeMessage,
        window: &mut Window,
        renderer: &mut Renderer,
        _conrod_handle: &mut ConrodHandle,
        audio_context: &mut AudioContext,
        _database: &mut Database,
    ) {
        renderer.is_render_gui = true;
        renderer.is_render_game = true;

        // {
        //     let entity = self.world.reserve_entity();
        let (objects, ref mut bound) = renderer.render_objects.next_static();
        objects.position = nalgebra::Vector3::new(0.0, 0.0, -20.0);
        objects.shape_type_material_ids.0 = ShapeType::Box;
        objects.shape_type_material_ids.1 = MaterialType::Checker;
        objects.shape_data1 = nalgebra::Vector4::new(20.0, 12.0, 1.0, 0.0);
        *bound = ObjectBound::Sphere(20.0);
        //     self.physics.collider_set.insert(
        //         ColliderBuilder::new(SharedShape::cuboid(
        //             objects.shape_data1.x,
        //             objects.shape_data1.y,
        //             objects.shape_data1.z,
        //         ))
        //         .translation(objects.position)
        //         .user_data(entity.to_bits() as u128)
        //         .build(),
        //     );
        //     self.world.spawn_at(entity, (Wall,));
        // }

        // {
        //     let entity = self.world.reserve_entity();
        //     let (objects, ref mut bound) = renderer.render_objects.next_static();
        //     objects.position = nalgebra::Vector3::new(0.0, 0.0, 10.0);
        //     objects.shape_type_material_ids.0 = ShapeType::Box;
        //     objects.shape_type_material_ids.1 = MaterialType::Checker;
        //     objects.shape_data1 = nalgebra::Vector4::new(20.0, 5.0, 1.0, 0.0);
        //     *bound = ObjectBound::Sphere(20.0);
        //     self.physics.collider_set.insert(
        //         ColliderBuilder::new(SharedShape::cuboid(
        //             objects.shape_data1.x,
        //             objects.shape_data1.y,
        //             objects.shape_data1.z,
        //         ))
        //         .translation(objects.position)
        //         .user_data(entity.to_bits() as u128)
        //         .build(),
        //     );
        //     self.world.spawn_at(entity, (Label("Wall"),));
        // }
        //
        // {
        //     let entity = self.world.reserve_entity();
        //     let (objects, ref mut bound) = renderer.render_objects.next_static();
        //     objects.position = nalgebra::Vector3::new(-20.0, 0.0, -5.0);
        //     objects.shape_type_material_ids.0 = ShapeType::Box;
        //     objects.shape_type_material_ids.1 = MaterialType::Checker;
        //     objects.shape_data1 = nalgebra::Vector4::new(1.0, 5.0, 15.0, 0.0);
        //     *bound = ObjectBound::Sphere(15.0);
        //     self.physics.collider_set.insert(
        //         ColliderBuilder::new(SharedShape::cuboid(
        //             objects.shape_data1.x,
        //             objects.shape_data1.y,
        //             objects.shape_data1.z,
        //         ))
        //         .translation(objects.position)
        //         .user_data(entity.to_bits() as u128)
        //         .build(),
        //     );
        //     self.world.spawn_at(entity, (Label("Wall"),));
        // }
        //
        // {
        //     let entity = self.world.reserve_entity();
        //     let (objects, ref mut bound) = renderer.render_objects.next_static();
        //     objects.position = nalgebra::Vector3::new(20.0, 0.0, -5.0);
        //     objects.shape_type_material_ids.0 = ShapeType::Box;
        //     objects.shape_type_material_ids.1 = MaterialType::Checker;
        //     objects.shape_data1 = nalgebra::Vector4::new(1.0, 5.0, 15.0, 0.0);
        //     *bound = ObjectBound::Sphere(15.0);
        //     self.physics.collider_set.insert(
        //         ColliderBuilder::new(SharedShape::cuboid(
        //             objects.shape_data1.x,
        //             objects.shape_data1.y,
        //             objects.shape_data1.z,
        //         ))
        //         .translation(objects.position)
        //         .user_data(entity.to_bits() as u128)
        //         .build(),
        //     );
        //     self.world.spawn_at(entity, (Label("Wall"),));
        // }

        {
            let player_rigid_body = self
                .physics
                .rigid_body_set
                .get(self.player_rigid_body_handle)
                .unwrap();
            renderer.camera.position = *player_rigid_body.translation();
        }

        window.set_is_cursor_grabbed(true);

        audio_context.global_sinks_map.remove("bgm");

        self.game_start_timer.reset(3.0);
        self.game_running = false;
    }

    fn update(
        &mut self,
        _window: &mut Window,
        renderer: &mut Renderer,
        input_manager: &InputManager,
        delta_time: f32,
        conrod_handle: &mut ConrodHandle,
        audio_context: &mut AudioContext,
        _control_flow: &mut ControlFlow,
        _database: &mut Database,
    ) -> SceneOp {
        renderer.camera.move_direction(input_manager.mouse_movement);

        let sec = self.game_timer.get_duration();

        let mut ui_cell = conrod_handle.get_ui_mut().set_widgets();
        {
            conrod_core::widget::Canvas::new()
                .color(conrod_core::color::TRANSPARENT)
                .set(self.ids.canvas, &mut ui_cell);

            conrod_core::widget::Canvas::new()
                .color(conrod_core::Color::Rgba(1.0, 1.0, 1.0, 0.3))
                .mid_top_of(self.ids.canvas)
                .wh(conrod_core::Dimensions::new(100.0, 30.0))
                .set(self.ids.canvas_duration, &mut ui_cell);

            conrod_core::widget::Text::new(&format!(
                "{:02}:{:02}",
                (sec / 60.0) as i32,
                (sec % 60.0) as i32
            ))
            .color(conrod_core::color::BLACK)
            .rgba(1.0, 1.0, 1.0, 1.0)
            .middle_of(self.ids.canvas_duration)
            .set(self.ids.duration_label, &mut ui_cell);
        }

        let mut scene_op = SceneOp::None;

        if !self.game_running {
            if self.game_start_timer.is_finished() {
                self.game_running = true;
            } else {
                self.game_start_timer.update(delta_time);
                conrod_core::widget::Text::new(&format!(
                    "{:.1}",
                    self.game_start_timer.get_duration()
                ))
                .align_middle_x_of(self.ids.canvas)
                .align_middle_y_of(self.ids.canvas)
                .set(self.ids.start_duration_label, &mut ui_cell);
            }
        } else {
            self.game_timer.update(delta_time);
            self.shoot_timer.update(delta_time);
            self.delta_shoot_time.update(delta_time);

            let mut missed = || {
                self.score.score -= 100;
                self.score.miss += 1;
            };

            let mut missed_secondary = false;

            for (id, (target, collider_handle)) in
                self.world.query_mut::<(&mut Target, &ColliderHandle)>()
            {
                if target.is_need_to_be_deleted(delta_time) {
                    self.entity_to_remove.push(id);
                    self.physics.collider_set.remove(
                        *collider_handle,
                        &mut self.physics.island_manager,
                        &mut self.physics.rigid_body_set,
                        false,
                    );

                    if !target.is_shooted() {
                        missed_secondary = true;
                    }
                }
            }

            for entity in self.entity_to_remove.iter() {
                self.world.despawn(*entity).unwrap();
            }
            self.entity_to_remove.clear();

            if missed_secondary {
                self.target_spawn_state = match self.target_spawn_state {
                    TargetSpawnState::Secondary => TargetSpawnState::Primary,
                    _ => unreachable!(),
                };
                missed();
                spawn_target(
                    &mut self.world,
                    &mut self.physics,
                    Vector3::new(0.0, 3.0, self.rng.sample(Uniform::new(-19.0, -13.0))),
                    Target::new(),
                );
            }

            self.shoot_animation.update(delta_time);
            renderer.rendering_info.fov_shootanim.y = lerp(
                0.0f32,
                -20.0f32.to_radians(),
                self.shoot_animation.get_value(),
            );

            self.physics.physics_pipeline.step(
                &self.physics.gravity,
                &self.physics.integration_parameters,
                &mut self.physics.island_manager,
                &mut self.physics.broad_phase,
                &mut self.physics.narrow_phase,
                &mut self.physics.rigid_body_set,
                &mut self.physics.collider_set,
                &mut self.physics.joint_set,
                &mut self.physics.ccd_solver,
                &(),
                &(),
            );
            self.physics.query_pipeline.update(
                &self.physics.island_manager,
                &self.physics.rigid_body_set,
                &self.physics.collider_set,
            );

            let _player_position = update_player_position(
                delta_time,
                input_manager,
                &mut renderer.camera,
                &mut self.physics,
                self.player_rigid_body_handle,
            );

            if input_manager.is_mouse_press(&MouseButton::Left) && self.shoot_timer.is_finished() {
                self.shoot_animation.trigger();
                self.shoot_timer.reset(0.4);

                let sink = rodio::Sink::try_new(&audio_context.output_stream_handle).unwrap();
                sink.append(
                    rodio::Decoder::new(BufReader::new(Cursor::new(AUDIO_FILE_SHOOT.to_vec())))
                        .unwrap(),
                );
                audio_context.push(Sink::Regular(sink));

                let ray = Ray::new(
                    nalgebra::Point::from(
                        renderer.camera.position
                            + renderer.camera.get_direction().into_inner() * 1.0,
                    ),
                    renderer.camera.get_direction().into_inner(),
                );
                const MAX_RAYCAST_DISTANCE: f32 = 1000.0;
                if let Some((handle, _distance)) = self.physics.query_pipeline.cast_ray(
                    &self.physics.collider_set,
                    &ray,
                    MAX_RAYCAST_DISTANCE,
                    true,
                    self.physics.interaction_groups,
                    None,
                ) {
                    let collider = self.physics.collider_set.get(handle).unwrap();
                    let entity = Entity::from_bits(collider.user_data as u64);

                    let mut need_to_spawn = false;
                    if let Ok(mut target) = self.world.get_mut::<Target>(entity) {
                        let sink =
                            rodio::Sink::try_new(&audio_context.output_stream_handle).unwrap();
                        sink.append(
                            rodio::Decoder::new(BufReader::new(Cursor::new(
                                AUDIO_FILE_SHOOTED.to_vec(),
                            )))
                            .unwrap(),
                        );
                        audio_context.push(Sink::Regular(sink));

                        if !target.is_shooted() {
                            let shoot_time = self.delta_shoot_time.get_duration();
                            self.delta_shoot_time.reset();

                            self.score.total_shoot_time += shoot_time;

                            need_to_spawn = true;
                            target.shooted();

                            self.target_spawn_state = match self.target_spawn_state {
                                TargetSpawnState::Primary => TargetSpawnState::Secondary,
                                TargetSpawnState::Secondary => TargetSpawnState::Primary,
                            };

                            self.score.score += ((300.0 * (3.0 - shoot_time)) as i32).max(0);
                            self.score.hit += 1;
                        } else {
                            missed();
                        }
                    } else {
                        missed();
                    }

                    if need_to_spawn {
                        match self.target_spawn_state {
                            TargetSpawnState::Primary => spawn_target(
                                &mut self.world,
                                &mut self.physics,
                                Vector3::new(0.0, 3.0, self.rng.sample(Uniform::new(-19.0, -13.0))),
                                Target::new(),
                            ),
                            TargetSpawnState::Secondary => {
                                let pos = nalgebra::Vector3::new(
                                    self.rng.sample(Uniform::new(-5.0, 5.0)),
                                    self.rng.sample(Uniform::new(0.5, 5.0)),
                                    self.rng.sample(Uniform::new(-19.0, -13.0)),
                                );
                                spawn_target(
                                    &mut self.world,
                                    &mut self.physics,
                                    pos,
                                    Target::new_with_delete_duration(
                                        self.secondary_delete_duration.clone(),
                                    ),
                                );
                            }
                        }
                    }
                } else {
                    missed();
                }
            }
        }

        drop(ui_cell);

        if sec <= 0.0 {
            scene_op = SceneOp::Push(
                Box::new(ClassicScoreScene::new(renderer, conrod_handle)),
                Some({
                    let mut m = HashMap::new();
                    self.score.write_message(&mut m);
                    m
                }),
            );
        }

        if input_manager.is_keyboard_press(&VirtualKeyCode::Escape) {
            scene_op = SceneOp::Push(Box::new(PauseScene::new(renderer, conrod_handle)), None);
        }

        scene_op
    }

    fn prerender(
        &mut self,
        renderer: &mut Renderer,
        _input_manager: &InputManager,
        _delta_time: f32,
        _conrod_handle: &mut ConrodHandle,
        _audio_context: &mut AudioContext,
    ) {
        for (_id, (collider_handle, target)) in self.world.query_mut::<(&ColliderHandle, &Target)>()
        {
            let collider = self.physics.collider_set.get_mut(*collider_handle).unwrap();
            let (objects, ref mut bound) = renderer.render_objects.next();
            objects.position = *collider.translation();
            objects.shape_type_material_ids.0 = ShapeType::Sphere;
            objects.shape_type_material_ids.1 = target.get_material();
            // let cam_to_obj = nalgebra::Unit::new_normalize(position.0 - renderer.camera.position);
            // let inner_cam_to_obj = cam_to_obj.into_inner() * -0.1;

            // objects.shape_data1.x = inner_cam_to_obj.x;
            // objects.shape_data1.y = inner_cam_to_obj.y;
            // objects.shape_data1.z = inner_cam_to_obj.z;
            objects.shape_data1.x = collider.shape().as_ball().unwrap().radius;

            *bound = ObjectBound::Sphere(0.5);
        }
    }

    fn deinit(
        &mut self,
        window: &mut Window,
        renderer: &mut Renderer,
        _conrod_handle: &mut ConrodHandle,
        _audio_context: &mut AudioContext,
        _database: &mut Database,
    ) {
        renderer.rendering_info.fov_shootanim.y = 0.0;
        renderer.render_objects.clear();
        window.set_is_cursor_grabbed(false);
    }
}
