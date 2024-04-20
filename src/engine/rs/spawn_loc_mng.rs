use rand::{rngs::StdRng, Rng};
use three_rs::math::{
    vector3::Vector3,  Quaternion
};
use crate::map::NodeTransform;

pub struct SpawnLoc {
    pub node: NodeTransform,
    pub cnt: u32,
    pub index: usize,
}

pub struct SpawnLocManager {
    pub locs: Vec<SpawnLoc>,
    last_index: usize,
}

impl SpawnLocManager {
    pub fn new(
        nodes: Vec<NodeTransform>
    ) -> Self {
        Self {
            locs: 
                nodes.iter().enumerate()
                    .map(|(index, n)| SpawnLoc{
                        node: n.clone(),
                        cnt: 0,
                        index
                    })
                    .collect::<Vec<_>>(),
            last_index: usize::MAX,
        }   
    }

    pub fn next(
        &mut self,
        rng: &mut StdRng
    ) -> (Vector3, Quaternion) {
        let min_cnt = self.locs.iter()
            .map(|l| l.cnt)
            .min()
            .unwrap();
        let mut min_locs = self.locs.iter_mut()
            .filter(|l| l.cnt == min_cnt)
            .collect::<Vec<_>>();
        
        let (l, index) = loop {
            let l = rng.gen::<usize>() % min_locs.len();
            let index = min_locs[l].index;
            if index != self.last_index || min_locs.len() == 1 {
                break (l, index);
            }
        };

        self.last_index = index;
            
        let loc = &mut min_locs[l];
        loc.cnt += 1;

        (
            loc.node.translation, 
            loc.node.rotation, 
        )
    }
}