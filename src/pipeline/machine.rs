use crate::{pipeline::{recipe::{CombinerRecipe, ProducerRecipe, Recipe, SplitterRecipe, TransformerRecipe}, IoPort, PipelineId, PortStatus}, ItemType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachinePushError {
    NoSpace,
    InvalidInput,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachineLinkError {
    NoFreeOutputs,
    NoFreeInputs,
    InputDoesNotExist,
    OutputDoesNotExist,
}

#[derive(Clone, Debug)]
pub enum Machine {
    Producer(Producer),
    Transformer(Transformer),
    Combiner(Combiner),
    Splitter(Splitter),
    Storage(Storage),
}

impl Machine {
    pub fn new(recipe: Recipe) -> Self {
        match recipe {
            Recipe::Producer(r) => Self::Producer(Producer::new(r)),
            Recipe::Transformer(r) => Self::Transformer(Transformer::new(r)),
            Recipe::Combiner(r) => Self::Combiner(Combiner::new(r)),
            Recipe::Splitter(r) => Self::Splitter(Splitter::new(r)),
        }
    }

    pub fn new_storage(item_type: ItemType) -> Self {
        Self::Storage(Storage::new(item_type))
    }
    
    pub fn tick(&mut self, ticks: u64) {
        match self {
            Self::Producer(inner) => inner.tick(ticks),
            Self::Transformer(inner) => inner.tick(ticks),
            Self::Combiner(inner) => inner.tick(ticks),
            Self::Splitter(inner) => inner.tick(ticks),
            Self::Storage(_) => {},
        }
    }

    pub fn push_input(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        match self {
            Self::Producer(inner) => inner.push(item_type, amount),
            Self::Transformer(inner) => inner.push(item_type, amount),
            Self::Combiner(inner) => inner.push(item_type, amount),
            Self::Splitter(inner) => inner.push(item_type, amount),
            Self::Storage(inner) => inner.push(item_type, amount),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Producer {
    pub output: Option<PipelineId>,
    pub recipe: ProducerRecipe,
    pub output_port: IoPort,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Transformer {
    pub output: Option<PipelineId>,
    pub recipe: TransformerRecipe,
    pub input_port: IoPort,
    pub output_port: IoPort
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Combiner {
    pub output: Option<PipelineId>,
    pub recipe: CombinerRecipe,
    pub input_ports: (IoPort, IoPort),
    pub output_port: IoPort,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Splitter {
    pub outputs: (Option<PipelineId>, Option<PipelineId>),
    pub recipe: SplitterRecipe,
    pub input_port: IoPort,
    pub output_ports: (IoPort, IoPort)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Storage {
    pub input_port: IoPort,
}

const DEFAULT_BUFFER_SIZE: u64 = 50;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemBuffer {
    current: u64,
    max: u64,
}

impl ItemBuffer {
    pub fn new() -> Self {
        Self { current: 0, max: DEFAULT_BUFFER_SIZE }
    }

    pub fn with_capacity(max: u64) -> Self {
        Self { current: 0, max }
    }
}

impl Producer {
    pub fn new(recipe: ProducerRecipe) -> Self {
        let port = recipe.into_port();
        Self { output: None, recipe, output_port: port }
    }

    pub fn tick(&mut self, ticks: u64) {
        
    }

    pub fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Err(MachinePushError::InvalidInput)
    }
}

impl Transformer {
    pub fn new(recipe: TransformerRecipe) -> Self {
        let ports = recipe.into_ports();
        Self { output: None, recipe, input_port: ports.input, output_port: ports.output }
    }

    pub fn tick(&mut self, ticks: u64) {
        
    }

    pub fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Ok(())
    }
}

impl Combiner {
    pub fn new(recipe: CombinerRecipe) -> Self {
        let ports = recipe.into_ports();
        Self { output: None, recipe, input_ports: ports.inputs, output_port: ports.output }
    }

    pub fn tick(&mut self, ticks: u64) {
        
    }

    pub fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Ok(())
    }
}

impl Splitter {
    pub fn new(recipe: SplitterRecipe) -> Self {
        let ports = recipe.into_ports();
        Self { outputs: (None, None), recipe, input_port: ports.input, output_ports: ports.outputs }
    }

    pub fn tick(&mut self, ticks: u64) {
        
    }

    pub fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Ok(())
    }
}

impl Storage {
    pub fn new(item_type: ItemType) -> Self {
        Self { input_port: IoPort::with_capacity(item_type, 1000) }
    }

    pub fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Ok(())
    }
}