#[cfg(not(feature = "js"))]
use {
    serde::{Deserialize, Serialize},
    dodgy::{Agent, AvoidanceOptions},
    glam::Vec2,
    three_rs::math::Vector3,
    crate::{entities::Entity, engine::GameEntities}
};

#[cfg_attr(not(feature = "js"), derive(Clone, Serialize, Deserialize))]
pub struct CollisionAvoidanceManager {
}

impl CollisionAvoidanceManager {
    pub fn new(
    ) -> Self {
        Self {
        }
    }

    #[cfg(not(feature = "js"))]
    pub fn move_player(
        &mut self,
        player_num: usize,
        translation: Vector3,
        entities: &GameEntities,
        delta_time: f32
    ) -> Vector3 {
        let (player_nums, agents) = entities.players.iter().enumerate()
            .filter(|(_, p)| !(*p).is_dead() && (*p).is_visible())
            .map(|(num, p)|
                (
                    num,
                    Agent {
                        position: Vec2::new(p.data.pos.x, p.data.pos.z),
                        velocity: Vec2::new(p.vel.x, p.vel.z),
                        radius: p.data.dim.x / 2.0,
                        avoidance_responsibility: 1.0,
                        max_velocity: p.vel.length(),
                    }
                )
            )
            .unzip::<usize, Agent, Vec<_>, Vec<_>>();

        let npcs = entities.npcs.iter()
            .filter(|n| !n.is_dead() && n.is_visible())
            .map(|n|
                Agent {
                    position: Vec2::new(n.data.pos.x, n.data.pos.z),
                    velocity: Vec2::new(n.vel.x, n.vel.z),
                    radius: n.data.dim.x / 2.0,
                    avoidance_responsibility: 0.0,
                    max_velocity: n.vel.length(),
                }
            )
            .collect();

        let agent_num = player_nums.iter().enumerate()
            .find(|(_, num)| **num == player_num).unwrap().0;

        let neighbours = self.find_neigbours(agent_num, &agents, &npcs);
        if neighbours.len() == 0 {
            translation
        }
        else {
            let preferred_velocity = Vec2::new(translation.x, translation.z);

            let avoidance_velocity = agents[agent_num].compute_avoiding_velocity(
                &neighbours,
                &vec![],
                preferred_velocity,
                delta_time,
                &AvoidanceOptions {
                    obstacle_margin: 0.1,
                    time_horizon: 3.0,
                    obstacle_time_horizon: 1.0,
                }
            );
            
            Vector3::new(
                avoidance_velocity.x, 
                translation.y, 
                avoidance_velocity.y
            )
        }
    }

    #[cfg(not(feature = "js"))]
    fn find_neigbours<'a>(
        &'a self, 
        agent_num: usize, 
        agents: &'a Vec<Agent>, 
        npcs: &'a Vec<Agent>
    ) -> Vec<&Agent> {
        let mut neigbours = vec![];

        let agent = &agents[agent_num];
        let max_distance_sq = agent.radius * 10.0 * 2.0;
        
        for (i, ag) in agents.iter().enumerate() {
            if i != agent_num {
                let distance_sq = ag.position.distance_squared(agent.position);
                if distance_sq < max_distance_sq {
                    neigbours.push(ag);
                }
            }
        }

        let max_distance_sq = agent.radius * 3.0 * 2.0;
        for npc in npcs.iter() {
            let distance_sq = npc.position.distance_squared(agent.position);
            if distance_sq < max_distance_sq {
                neigbours.push(npc);
            }
        }
        
        neigbours
    }
}
