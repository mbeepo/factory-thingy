use std::collections::HashMap;

use crate::{pipeline::machine::{ItemBuffer, MachineKind, MachineBindError, MachineTrait}, CentralStorage, ItemType};

pub mod recipe;
pub mod machine;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortStatus {
    Free,
    Taken,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct IoPort {
    pub status: PortStatus,
    pub buffer: ItemBuffer,
    pub item_type: ItemType,
}

impl IoPort {
    pub fn new(item_type: ItemType) -> Self {
        Self { status: PortStatus::Free, buffer: ItemBuffer::new(), item_type }
    }

    pub fn with_capacity(item_type: ItemType, capacity: u64) -> Self {
        Self { status: PortStatus::Free, buffer: ItemBuffer::with_capacity(capacity), item_type }
    }

    pub fn is_free(&self) -> bool {
        self.status == PortStatus::Free
    }
}

impl From<ItemType> for IoPort {
    fn from(value: ItemType) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct PipelineId {
    inner: usize,
}

impl PipelineId {
    fn new() -> Self {
            Self { inner: 0 }
    }

    fn inc(&mut self) -> Self {
        let out = self.clone();
        self.inner += 1;
        out
    }
}

#[derive(Debug)]
pub struct Pipeline {
    pub inner: HashMap<PipelineId, PipelineSegment>,
    pub next_id: PipelineId,
    starters: Vec<PipelineId>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { inner: HashMap::with_capacity(3), next_id: PipelineId::new(), starters: Vec::with_capacity(1) }
    }

    pub fn with_machine(machine: Box<dyn MachineTrait>) -> Self {
        let starter = machine.is_starter();
        let mut next_id = PipelineId::new();
        let mut inner = HashMap::with_capacity(3);
        let id = next_id.inc();
        inner.insert(id, PipelineSegment::new(id, machine));

        if starter {
            Self { inner, next_id, starters: vec![id] }
        } else {
            Self { inner, next_id, starters: Vec::with_capacity(1) }
        }
    }

    pub fn with_machines(machines: Vec<Box<dyn MachineTrait>>) -> Self {
        let mut next_id = PipelineId::new();
        let mut starters: Vec<PipelineId> = Vec::with_capacity(1);
        let inner = HashMap::from_iter(machines.into_iter().map(|m| {
            let id = next_id.inc();
            if m.is_starter() { starters.push(id) };
            (id, PipelineSegment::new(id, m))
        }));

        Self { inner, next_id, starters }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: HashMap::with_capacity(capacity), next_id: PipelineId::new(), starters: Vec::with_capacity(1) }
    }

    pub fn push(&mut self, machine: Box<dyn MachineTrait>) -> PipelineId {
        let id = self.next_id();
        let segment = PipelineSegment::new(id, machine);
        self.inner.insert(id, segment);
        id
    }

    pub fn bind_output(&mut self, input_id: PipelineId, output_id: PipelineId) -> Result<(), MachineBindError> {

        let [input, output] = self.inner.get_disjoint_mut([&input_id, &output_id]);
        println!("{:?}\n{:?}", input, output);
        let Some(input) = input else { Err(MachineBindError::InputDoesNotExist)? };
        let Some(output) = output else { Err(MachineBindError::OutputDoesNotExist)? };

        let output_data = input.inner.get_output()?;
        let input_data = output.inner.get_matching_input(&output_data);

        Ok(())
    }

    pub fn tick(&mut self, ticks: u64) -> CentralStorage {
        let mut to_prune: Vec<PipelineId> = Vec::new();
        let mut central_storage = CentralStorage::default();

        for &producer in &self.starters { 
            if let Some(producer) = self.inner.get_mut(&producer) {
                producer.inner.tick(ticks);
            } else {
                to_prune.push(producer);
            }
        }

        if to_prune.len() > 0 {
            self.starters = self.starters.iter().copied().filter(|p| !to_prune.contains(&p)).collect();
        }

        central_storage
    }

    fn next_id(&mut self) -> PipelineId {
        self.next_id.inc()
    }
}

#[derive(Debug)]
pub struct PipelineSegment {
    id: PipelineId,
    inner: Box<dyn MachineTrait>,
}

impl PipelineSegment {
    pub fn new(id: PipelineId, inner: Box<dyn MachineTrait>) -> Self {
        Self { id, inner: inner }
    }
}