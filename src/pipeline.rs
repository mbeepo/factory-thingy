use std::collections::HashMap;

use crate::{pipeline::machine::{ItemBuffer, Machine, MachineBindError}, ItemType};

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

    pub fn with_machine(machine: Machine) -> Self {
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

    pub fn with_machines(machines: Vec<Machine>) -> Self {
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

    pub fn push(&mut self, machine: Machine) -> PipelineId {
        let id = self.next_id();
        let segment = PipelineSegment::new(id, machine);
        self.inner.insert(id, segment);
        id
    }

    pub fn bind_output(&mut self, src_id: PipelineId, dest_id: PipelineId) -> Result<(), MachineBindError> {
        let [src, dest] = self.inner.get_disjoint_mut([&src_id, &dest_id]);
        let Some(input) = src else { Err(MachineBindError::InputDoesNotExist)? };
        let Some(output) = dest else { Err(MachineBindError::OutputDoesNotExist)? };

        for item_type in input.inner.outputs() {
            let output_data = input.inner.get_output(item_type)?;
            let input_data = output.inner.get_input(item_type)?;

            *output_data.id = Some(dest_id);
            output_data.port.expect("Invalid output port").status = PortStatus::Taken;
            input_data.port.expect("Invalid input port").status = PortStatus::Taken;
        }

        Ok(())
    }

    pub fn tick(&mut self) {
        let mut to_prune: Vec<PipelineId> = Vec::new();

        for &producer in &self.starters { 
            if let Some(producer) = self.inner.get_mut(&producer) {
                producer.inner.tick();
            } else {
                to_prune.push(producer);
            }
        }

        if to_prune.len() > 0 {
            self.starters = self.starters.iter().copied().filter(|p| !to_prune.contains(&p)).collect();
        }
    }

    fn next_id(&mut self) -> PipelineId {
        self.next_id.inc()
    }
}

#[derive(Debug)]
pub struct PipelineSegment {
    id: PipelineId,
    inner: Machine,
}

impl PipelineSegment {
    pub fn new(id: PipelineId, inner: Machine) -> Self {
        Self { id, inner: inner }
    }
}