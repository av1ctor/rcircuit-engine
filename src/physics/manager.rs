use rapier3d::{
    control::KinematicCharacterController, 
    na::{Isometry, Translation3, UnitQuaternion}, 
    prelude::*
};
use serde::{Deserialize, Serialize};
use three_rs::math::{Quaternion, Vector3};
use super::character::{Character, CharacterRef};

#[derive(Clone, Serialize, Deserialize)]
pub struct PhysicsManager {
    query_pipeline: QueryPipeline,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    island_manager: IslandManager,
    impulse_joint_set: ImpulseJointSet,
    multibody_impulse_joint_set: MultibodyJointSet,
    character_controller: KinematicCharacterController,
    characters: Vec<Character>,
}

impl PhysicsManager {
    pub fn new(
    ) -> Self {
        Self {
            query_pipeline: QueryPipeline::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            island_manager: IslandManager::default(),
            impulse_joint_set: ImpulseJointSet::default(),
            multibody_impulse_joint_set: MultibodyJointSet::default(),
            character_controller: KinematicCharacterController{
                slide: false,
                autostep: None,
                snap_to_ground: None,
                ..KinematicCharacterController::default()
            },
            characters: vec![],
        }
    }

    pub fn add_character(
        &mut self,
        pos: &Vector3,
        rot: &Quaternion,
        height: f32,
        radius: f32,
        rb_typ: Option<RigidBodyType>
    ) -> CharacterRef {
        let char_ref = self.characters.len();

        let rb = if let Some(rb_typ) = rb_typ {
            match rb_typ {
                RigidBodyType::Dynamic => 
                    Some(RigidBodyBuilder::dynamic()
                        .build()),
                RigidBodyType::Fixed => 
                    Some(RigidBodyBuilder::fixed()
                        .build()),
                RigidBodyType::KinematicPositionBased => 
                    Some(RigidBodyBuilder::kinematic_position_based()
                        .build()),
                RigidBodyType::KinematicVelocityBased => 
                    Some(RigidBodyBuilder::kinematic_velocity_based()
                        .build()),
            }
        }
        else {
            None
        };
        
        let rigid_body_handle = if let Some(rb) = rb {
            Some(self.rigid_body_set.insert(rb))
        }
        else {
            None
        };

        let coll = ColliderBuilder::capsule_x(height / 2.0, radius)
            .position(Isometry::from_parts(
                    Translation3::new(pos.x, pos.y, pos.z), 
                    UnitQuaternion::from_quaternion(
                        rapier3d::na::Quaternion::new(rot.w, rot.x, rot.y, rot.z)
                    )
                )
            )
            .build();

        let collider_handle = if let Some(rigid_body_handle) = rigid_body_handle {
            self.collider_set.insert_with_parent(
                coll, rigid_body_handle, &mut self.rigid_body_set)
        }
        else {
            self.collider_set.insert(coll)
        };

        self.characters.push(Character {
            mass: 1.0,
            rigid_body_handle,
            collider_handle,
        });

        char_ref
    }

    pub fn remove_character(
        &mut self,
        char_ref: CharacterRef
    ) {
        let char = &self.characters[char_ref];
        
        if let Some(rigid_body_handle) = char.rigid_body_handle {
            self.rigid_body_set.remove(
                rigid_body_handle, 
                &mut self.island_manager, 
                &mut self.collider_set, 
                &mut self.impulse_joint_set, 
                &mut self.multibody_impulse_joint_set, 
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
        to: &Vector3
    ) -> Option<Vector3> {
        let char = &mut self.characters[char_ref];
        let collider = self.collider_set.get(char.collider_handle).unwrap();

        let pos = collider.position();
        let translation = vector![
            to.x - pos.translation.x,
            to.y - pos.translation.y,
            to.z - pos.translation.z,
        ];

        let filter = if let Some(rigid_body_handle) = char.rigid_body_handle {
            QueryFilter::new()
                .exclude_rigid_body(rigid_body_handle)
        }
        else {
            QueryFilter::new()
        };

        let mut collided = false;

        let mvt = self.character_controller.move_shape(
            1.0 / 60.0,
            &self.rigid_body_set,
            &self.collider_set,
            &self.query_pipeline,
            collider.shape(),
            collider.position(),
            translation,
            filter,
            |_| {collided = true}
        );

        if !collided {
            None
        }
        else {

            let collider = self.collider_set.get_mut(char.collider_handle).unwrap();

            if let Some(rb_handle) = collider.parent() {
                let rb = self.rigid_body_set.get_mut(rb_handle).unwrap();
                match rb.body_type() {
                    RigidBodyType::Dynamic => 
                        rb.set_translation(mvt.translation, true),
                    RigidBodyType::Fixed => 
                        (),
                    RigidBodyType::KinematicPositionBased => 
                        rb.set_next_kinematic_translation(mvt.translation),
                    RigidBodyType::KinematicVelocityBased => 
                        (),
                }
            }
            else {
                collider.set_translation(mvt.translation);
            }

            Some(Vector3::new(
                mvt.translation.x,
                mvt.translation.y,
                mvt.translation.z
            ))
        }
    }
}