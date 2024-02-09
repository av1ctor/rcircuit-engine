use std::f32::consts::PI;
use rapier3d::{
    control::{CharacterLength, KinematicCharacterController}, 
    na::{Isometry, Translation3, UnitQuaternion}, 
    prelude::*
};
use serde::{Deserialize, Serialize};
use three_rs::math::{Quaternion, Vector3};
use super::character::{Character, CharacterRef};

#[derive(Serialize, Deserialize)]
pub struct PhysicsManager {
    #[serde(skip)]
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    integration_parameters: IntegrationParameters,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    query_pipeline: QueryPipeline,
    ccd_solver: CCDSolver,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    gravity: Vector3,
    character_controller: KinematicCharacterController,
    characters: Vec<Character>,
    num_dyn_chars: usize,
}

impl Clone for PhysicsManager {
    fn clone(&self) -> Self {
        Self { 
            physics_pipeline: PhysicsPipeline::new(), 
            island_manager: self.island_manager.clone(), 
            integration_parameters: self.integration_parameters.clone(), 
            broad_phase: self.broad_phase.clone(), 
            narrow_phase: self.narrow_phase.clone(), 
            query_pipeline: self.query_pipeline.clone(), 
            ccd_solver: self.ccd_solver.clone(), 
            rigid_body_set: self.rigid_body_set.clone(), 
            collider_set: self.collider_set.clone(), 
            impulse_joint_set: self.impulse_joint_set.clone(), 
            multibody_joint_set: self.multibody_joint_set.clone(), 
            gravity: self.gravity.clone(), 
            character_controller: self.character_controller.clone(), 
            characters: self.characters.clone(),
            num_dyn_chars: self.num_dyn_chars,
        }
    }
}

impl PhysicsManager {
    pub fn new(
        gravity: Vector3
    ) -> Self {
        Self {
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::default(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            query_pipeline: QueryPipeline::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            impulse_joint_set: ImpulseJointSet::default(),
            multibody_joint_set: MultibodyJointSet::default(),
            gravity,
            character_controller: KinematicCharacterController{
                slide: false,
                autostep: None,
                snap_to_ground: None,
                max_slope_climb_angle: 0.0,
                min_slope_slide_angle: PI / 2.0,
                offset: CharacterLength::Absolute(0.01),
                ..KinematicCharacterController::default()
            },
            characters: vec![],
            num_dyn_chars: 0,
        }
    }

    pub fn step(
        &mut self
    ) {
        self.physics_pipeline.step(
            &vector![self.gravity.x, self.gravity.y, self.gravity.z],
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
          );
    }

    pub fn add_character(
        &mut self,
        id: usize,
        pos: &Vector3,
        rot: &Quaternion,
        height: f32,
        radius: f32,
        mass: f32,
        enabled: bool,
        rb_typ: Option<RigidBodyType>
    ) -> CharacterRef {
        let char_ref = self.characters.len();

        let position = Isometry::from_parts(
            Translation3::new(pos.x, pos.y, pos.z), 
            UnitQuaternion::from_quaternion(
                rapier3d::na::Quaternion::new(rot.w, rot.x, rot.y, rot.z)
            )
        );

        let rb = match rb_typ {
            Some(RigidBodyType::Dynamic) => {
                if enabled {
                    self.num_dyn_chars += 1
                };
                
                Some(RigidBodyBuilder::dynamic()
                    .position(position)
                    .enabled(enabled)
                    .build())
            },
            Some(RigidBodyType::Fixed) => {
                Some(RigidBodyBuilder::fixed()
                    .position(position)
                    .enabled(enabled)
                    .build())
            },
            Some(RigidBodyType::KinematicPositionBased) => {
                Some(RigidBodyBuilder::kinematic_position_based()
                    .position(position)
                    .enabled(enabled)
                    //.enabled_translations(false, true, false)
                    .build())
            },
            _ =>
                None
        };
        
        let rigid_body_handle = if let Some(rb) = rb {
            Some(self.rigid_body_set.insert(rb))
        }
        else {
            None
        };

        let mut collider = ColliderBuilder::capsule_y(height / 2.0 - radius, radius)
            .enabled(enabled)
            .active_collision_types(
                ActiveCollisionTypes::default() | 
                ActiveCollisionTypes::KINEMATIC_FIXED |
                ActiveCollisionTypes::KINEMATIC_KINEMATIC
            )
            .user_data(id as _)
            .build();

        let collider_handle = if let Some(rigid_body_handle) = rigid_body_handle {
            self.collider_set.insert_with_parent(
                collider, rigid_body_handle, &mut self.rigid_body_set)
        }
        else {
            collider.set_position(position);
            self.collider_set.insert(collider)
        };

        self.characters.push(Character {
            mass,
            rigid_body_handle,
            collider_handle,
        });

        char_ref
    }

    pub fn toggle_character(
        &mut self,
        char_ref: CharacterRef,
        enabled: bool
    ) {
        let char = &self.characters[char_ref];
        if let Some(rigid_body_handle) = char.rigid_body_handle {
            let rb = self.rigid_body_set.get(rigid_body_handle).unwrap();
            if rb.is_dynamic() {
                if !rb.is_enabled() && enabled {
                    self.num_dyn_chars += 1;
                }
                else if rb.is_enabled() && !enabled {
                    self.num_dyn_chars -= 1;
                }
            };

            self.rigid_body_set.get_mut(rigid_body_handle).unwrap()
                .set_enabled(enabled);    
        }
        self.collider_set.get_mut(char.collider_handle).unwrap()
            .set_enabled(enabled);
    }

    pub fn remove_character(
        &mut self,
        char_ref: CharacterRef
    ) {
        let char = &self.characters[char_ref];
        
        if let Some(rigid_body_handle) = char.rigid_body_handle {
            let rb = self.rigid_body_set.get(rigid_body_handle).unwrap();
            if rb.is_dynamic() && rb.is_enabled() {
                self.num_dyn_chars -= 1;
            };

            self.rigid_body_set.remove(
                rigid_body_handle, 
                &mut self.island_manager, 
                &mut self.collider_set, 
                &mut self.impulse_joint_set, 
                &mut self.multibody_joint_set, 
                true
            );
        }
        else {
            self.collider_set.remove(
                char.collider_handle, 
                &mut self.island_manager, 
                &mut self.rigid_body_set, false
            );
        }

        self.characters.remove(char_ref);
    }

    pub fn move_character(
        &mut self,
        char_ref: CharacterRef,
        translation: &Vector3
    ) -> Option<Vector3> {
        let char = &self.characters[char_ref];

        let filter = if let Some(rigid_body_handle) = char.rigid_body_handle {
            QueryFilter::new()
                .exclude_rigid_body(rigid_body_handle)
                .predicate(&|_, c| c.is_enabled())
        }
        else {
            QueryFilter::new()
                .predicate(&|_, c| c.is_enabled())
        };

        let mut collisions = vec![];

        let collider = self.collider_set.get(char.collider_handle).unwrap();

        let mvt = self.character_controller.move_shape(
            self.integration_parameters.dt,
            &self.rigid_body_set,
            &self.collider_set,
            &self.query_pipeline,
            collider.shape(),
            collider.position(),
            vector![
                translation.x,
                translation.y,
                translation.z,
            ],
            filter,
            |c| {collisions.push(c)}
        );

        if collisions.len() > 0 && self.num_dyn_chars > 0 {
            for collision in &collisions {
                self.character_controller.solve_character_collision_impulses(
                    self.integration_parameters.dt,
                    &mut self.rigid_body_set,
                    &self.collider_set,
                    &self.query_pipeline,
                    collider.shape(),
                    char.mass,
                    collision,
                    filter,
                );
            }

            //FIXME: update the game object position and rotation
        }

        if let Some(rb_handle) = collider.parent() {
            let rb = self.rigid_body_set.get_mut(rb_handle).unwrap();
            match rb.body_type() {
                RigidBodyType::KinematicPositionBased => {
                    rb.set_translation(rb.translation() + mvt.translation, true);
                },
                _ => {
                },
            }
        
        }
        else {
            let collider = self.collider_set.get_mut(char.collider_handle).unwrap();
            collider.set_translation(collider.translation() + mvt.translation);
        }

        if collisions.len() == 0 {
            None
        }            
        else {
            Some(Vector3::new(
                mvt.translation.x,
                mvt.translation.y,
                mvt.translation.z
            ))
        }
    }
}