use std::collections::HashMap;

use crate::{pipeline::{machine::{ItemBuffer, Machine, MachineLinkError}, recipe::{CombinerRecipe, ProducerRecipe, SplitterRecipe, TransformerRecipe}}, CentralStorage, ItemType};

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

#[derive(Clone, Debug)]
pub struct Pipeline {
    pub inner: HashMap<PipelineId, PipelineSegment>,
    pub next_id: PipelineId,
    producers: Vec<PipelineId>,
}

impl Pipeline {
    pub fn new() -> Self {
        Self { inner: HashMap::with_capacity(3), next_id: PipelineId::new(), producers: Vec::with_capacity(1) }
    }

    pub fn with_machine(start: Machine) -> Self {
        let producer = match start {
            Machine::Producer(_) => true,
            _ => false,
        };
        let mut next_id = PipelineId::new();
        let mut inner = HashMap::with_capacity(3);
        let id = next_id.inc();
        inner.insert(id, PipelineSegment::new(id, start));

        if producer {
            Self { inner, next_id, producers: vec![id] }
        } else {
            Self { inner, next_id, producers: Vec::with_capacity(1) }
        }
    }

    pub fn with_machines(start: Vec<Machine>) -> Self {
        let mut next_id = PipelineId::new();
        let mut producers: Vec<PipelineId> = Vec::with_capacity(1);
        let inner = HashMap::from_iter(start.into_iter().map(|m| {
            let id = next_id.inc();
            if let Machine::Producer(_) = m { producers.push(id) };
            (id, PipelineSegment::new(id, m))
        }));

        Self { inner, next_id, producers }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { inner: HashMap::with_capacity(capacity), next_id: PipelineId::new(), producers: Vec::with_capacity(1) }
    }

    pub fn push(&mut self, machine: Machine) -> PipelineId {
        let id = self.next_id();
        let segment = PipelineSegment::new(id, machine);
        self.inner.insert(id, segment);
        id
    }

    pub fn bind_output(&mut self, input_id: PipelineId, output_id: PipelineId) -> Result<(), MachineLinkError> {
        struct Output<'a> {
            id: &'a mut Option<PipelineId>,
            port: &'a mut IoPort,
            item_type: ItemType,
        }

        let [input, output] = self.inner.get_disjoint_mut([&input_id, &output_id]);
        println!("{:?}\n{:?}", input, output);
        let Some(input) = input else { Err(MachineLinkError::InputDoesNotExist)? };
        let Some(output) = output else { Err(MachineLinkError::OutputDoesNotExist)? };

        let output_data = match &mut *input.inner {
            Machine::Producer(inner) => {
                if inner.output_port.is_free() {
                    Output { id: &mut inner.output, port: &mut inner.output_port, item_type: inner.recipe.output }
                } else {
                    return Err(MachineLinkError::NoFreeOutputs)
                }
            },
            Machine::Transformer(inner) => {
                if inner.output_port.is_free() {
                    Output { id: &mut inner.output, port: &mut inner.output_port, item_type: inner.recipe.output }
                } else {
                    return Err(MachineLinkError::NoFreeOutputs)
                }
            },
            Machine::Combiner(inner) => {
                if inner.output_port.is_free() {
                    Output { id: &mut inner.output, port: &mut inner.output_port, item_type: inner.recipe.output }
                } else { 
                    return Err(MachineLinkError::NoFreeOutputs)
                }
            },
            Machine::Splitter(inner) => {
                match inner.outputs {
                    (None, _) => Output { id: &mut inner.outputs.0, port: &mut inner.output_ports.0, item_type: inner.recipe.outputs.0 },
                    (_, None) => Output { id: &mut inner.outputs.1, port: &mut inner.output_ports.1, item_type: inner.recipe.outputs.1 },
                    _ => return Err(MachineLinkError::NoFreeOutputs),
                }
            },
            Machine::Storage(_) => return Err(MachineLinkError::NoFreeOutputs),
        };

        match &mut *output.inner {
            Machine::Producer(_) => return Err(MachineLinkError::NoFreeInputs),
            Machine::Transformer(inner) => {
                if inner.input_port.is_free() {
                    *output_data.id = Some(output_id);
                    output_data.port.status = PortStatus::Taken;
                    inner.input_port.status = PortStatus::Taken;
                } else {
                    return Err(MachineLinkError::NoFreeInputs)
                }
            },
            Machine::Combiner(inner) => {
                if inner.input_ports.0.item_type == output_data.item_type {
                    *output_data.id = Some(output_id);
                    output_data.port.status = PortStatus::Taken;
                    inner.input_ports.0.status = PortStatus::Taken;
                } else if inner.input_ports.1.item_type == output_data.item_type {
                    *output_data.id = Some(output_id);
                    output_data.port.status = PortStatus::Taken;
                    inner.input_ports.1.status = PortStatus::Taken;
                } else {
                    return Err(MachineLinkError::NoFreeInputs)
                }
            },
            Machine::Splitter(inner) => todo!(),
            Machine::Storage(inner) => {
                if inner.input_port.is_free() {
                    *output_data.id = Some(output_id);
                    output_data.port.status = PortStatus::Taken;
                    inner.input_port.status = PortStatus::Taken;
                } else {
                    return Err(MachineLinkError::NoFreeInputs)
                }
            }
        }

        Ok(())
    }

    pub fn tick(&mut self, ticks: u64) -> CentralStorage {
        let mut to_prune: Vec<PipelineId> = Vec::new();
        let mut central_storage = CentralStorage::default();

        for &producer in &self.producers { 
            if let Some(producer) = self.inner.get_mut(&producer) {
                producer.inner.tick(ticks);
            } else {
                to_prune.push(producer);
            }
        }

        if to_prune.len() > 0 {
            self.producers = self.producers.iter().copied().filter(|p| !to_prune.contains(&p)).collect();
        }

        central_storage
    }

    fn next_id(&mut self) -> PipelineId {
        self.next_id.inc()
    }
}

#[derive(Clone, Debug)]
pub struct PipelineSegment {
    id: PipelineId,
    inner: Box<Machine>,
}

impl PipelineSegment {
    pub fn new(id: PipelineId, inner: Machine) -> Self {
        Self { id, inner: Box::new(inner) }
    }
}