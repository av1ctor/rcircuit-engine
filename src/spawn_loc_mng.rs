use rand::{rngs::StdRng, Rng};
use three_rs::math::{
    vector3::Vector3,  Quaternion
};
use crate::map::NodeTransform;

pub struct SpawnLocManager {
    pub locs: Vec<(NodeTransform, usize)>,
}

impl SpawnLocManager {
    pub fn new(
        nodes: Vec<NodeTransform>
    ) -> Self {
        Self {
            locs: 
                nodes.iter()
                    .map(|n| (n.clone(), 0usize))
                    .collect::<Vec<_>>(),
        }   
    }

    pub fn next(
        &mut self,
        idx: usize,
        rng: &mut StdRng
    ) -> (Vector3, Quaternion) {
        let min_cnt = self.locs.iter()
                .map(|l| l.1)
                .filter(|cnt| *cnt <= idx)
                .reduce(|min, cnt| if cnt <= min {cnt} else {min})
                .unwrap();
        let mut min_locs = self.locs.iter_mut()
            .filter(|l| l.1 <= min_cnt)
            .collect::<Vec<_>>();
        
        let l = rng.gen::<usize>() % min_locs.len();
            
        min_locs[l].1 += 1;

        (
            min_locs[l].0.translation, 
            min_locs[l].0.rotation, 
        )
    }
}