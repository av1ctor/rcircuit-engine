use rand::{rngs::StdRng, Rng};
use serde::{Serialize, Deserialize};
use strum_macros::EnumCount;
use three_rs::math::{
    vector3::{Vector3, RIGHT, UP, FORWARD}, 
    quaternion::Quaternion, 
    euler::{EulerOrder, Euler}, 
    box3::Box3
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumCount, Default)]
pub enum ObjectType {
    #[default]
    UNKNOWN,
    PLAYER,
    ENEMY,
    KEY,
    TRAP,
    EXIT,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum ObjectState {
    #[default]
    IDLE,
    MOVING,
    RUNNING,
    JUMPING,
    DANCING,
    CROUCHING,
    SITTING,
    SHOOTING,
    PUNCHING,
    KICKING,
    CLIMBING,
    HURTING,
    EXPLODING,
    AGONIZING,
    DEAD,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum ObjectVisibility {
    #[default]
    HIDDEN,
    VISIBLE,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ObjectDir {
    pub rot: Vector3,
    pub q: Quaternion,
    pub angle: f32,
}

impl ObjectDir {
    pub fn from_vector(
        dir: &Vector3
    ) -> Self {
        let q = Quaternion::from_euler(
            &Euler::from_vector(dir.clone(), EulerOrder::XYZ)
        );
        Self {
            rot: dir.clone(),
            q,
            angle: q.angle(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub id: usize,
    pub typ: ObjectType,
    pub vis: ObjectVisibility,
    pub state: ObjectState,
    pub pos: Vector3,
    pub dir: ObjectDir,
    pub dim: Vector3,
    pub radius: f32,
}

impl Default for Object {
    fn default() -> Self {
        Self { 
            id: Default::default(), 
            typ: Default::default(), 
            pos: Default::default(), 
            dir: Default::default(),
            dim: Default::default(), 
            radius: Default::default(), 
            state: Default::default(), 
            vis: Default::default(),
        }
    }
}

impl Object {
    pub fn new(
        typ: ObjectType,
        pos: Vector3,
        dir: Quaternion,
        dim: Vector3,
        state: ObjectState,
        vis: ObjectVisibility,
        rng: &mut StdRng
    ) -> Self {
        Self { 
            id: rng.gen::<usize>(),
            pos,
            dir: ObjectDir {
                rot: Euler::from_quartenion(&dir, EulerOrder::XYZ).v,
                q: dir,
                angle: dir.angle(),
            },
            dim,
            radius: 1.0,
            typ,
            state,
            vis,
        }
    }

    fn rotate_on_axis(
        &mut self,
        axis: &Vector3,
        angle: f32
    ) {
        let q = self.dir.q.rotate_on_axis(axis, angle);
        self.dir = ObjectDir {
            rot: Euler::from_quartenion(&q, EulerOrder::XYZ).v,
            q,
            angle: q.angle(),
        };
    }

    pub fn rotate_x(
        &mut self,
        angle: f32
    ) {
        self.rotate_on_axis(&RIGHT, angle);
    }

    pub fn rotate_y(
        &mut self,
        angle: f32
    ) {
        self.rotate_on_axis(&UP, angle);
    }

    pub fn rotate_z(
        &mut self,
        angle: f32
    ) {
        self.rotate_on_axis(&FORWARD, angle);
    }
    
    pub fn calc_bb(
        &self 
    ) -> Box3 {
        let half_dim = self.dim.mul_scalar(0.5);
        Box3::new(
            self.pos.sub(&half_dim), 
            self.pos.add(&half_dim)
        )
    }
}