use super::{Coordinator, Spawn};

pub struct Topology {
    coordinator: Coordinator,
    nodes: Vec<Box<Spawn>>,
}

impl Topology {
    pub fn new(coordinator: Coordinator, nodes: Vec<Box<Spawn>>) -> Topology {
        Topology {
            coordinator: coordinator,
            nodes: nodes,
        }
    }

    pub fn run(self) {
        self.coordinator.spawn();

        for node in self.nodes.into_iter() {
            node.spawn();
        }
    }
}
