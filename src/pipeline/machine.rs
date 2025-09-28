use std::fmt::Debug;

use crate::{pipeline::{recipe::{CombinatorRecipe, ProducerRecipe, Recipe, SeparatorRecipe, TransformerRecipe}, IoPort, PipelineId, PortStatus}, ItemType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachinePushError {
    NoSpace,
    InvalidInput,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MachineBindError {
    NoFreeOutputs,
    NoFreeInputs,
    InputDoesNotExist,
    OutputDoesNotExist,
    InvalidInput,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineKind {
    Producer,
    Transformer,
    Combinator,
    Separator,
    Storage,
}

#[derive(Clone, Debug)]
pub struct Machine {
    pub kind: MachineKind,
    recipe: Recipe,
    input_ports: [Option<IoPort>; 4],
    output_ports: [Option<IoPort>; 4],
    outputs: [Option<PipelineId>; 4],
}

impl Machine {
    pub fn tick(&mut self, ticks: u64) {
        todo!()
    }

    pub fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        todo!()
    }

    pub fn is_starter(&self) -> bool {
        todo!()
    }

    pub fn get_output(&mut self) -> Result<MachineOutput, MachineBindError> {
        todo!()
    }

    pub fn get_matching_input(&mut self, output_handle: &MachineOutput) -> Result<MachineInput, MachineBindError> {
        todo!()
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
pub struct Combinator {
    pub output: Option<PipelineId>,
    pub recipe: CombinatorRecipe,
    pub input_ports: (IoPort, IoPort),
    pub output_port: IoPort,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Separator {
    pub outputs: (Option<PipelineId>, Option<PipelineId>),
    pub recipe: SeparatorRecipe,
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

pub struct MachineOutput<'a> {
    pub id: &'a mut Option<PipelineId>,
    pub port: &'a mut IoPort,
    pub item_type: ItemType,
}

pub struct MachineInput<'a> {
    pub port: &'a mut IoPort,
}

pub trait MachineTrait: Debug {
    fn tick(&mut self, ticks: u64);
    fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError>;
    fn is_starter(&self) -> bool;
    fn get_output(&mut self) -> Result<MachineOutput, MachineBindError>;
    fn get_matching_input(&mut self, output_handle: &MachineOutput) -> Result<MachineInput, MachineBindError>;
}

impl Producer {
    pub fn new(recipe: ProducerRecipe) -> Self {
        let port = recipe.into_port();
        Self { output: None, recipe, output_port: port }
    }
}

impl MachineTrait for Producer {
    fn tick(&mut self, ticks: u64) {
        
    }

    fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Err(MachinePushError::InvalidInput)
    }

    fn is_starter(&self) -> bool {
        true
    }
    
    fn get_output(&mut self) -> Result<MachineOutput, MachineBindError> {
        if self.output_port.is_free() {
            Ok(MachineOutput { id: &mut self.output, port: &mut self.output_port, item_type: self.recipe.output })
        } else {
            return Err(MachineBindError::NoFreeOutputs)
        }
    }

    fn get_matching_input(&mut self, output_handle: &MachineOutput) -> Result<MachineInput, MachineBindError> {
        return Err(MachineBindError::NoFreeInputs)
    }
}

impl From<ProducerRecipe> for Producer {
    fn from(value: ProducerRecipe) -> Self {
        Self { recipe: value, output: None, output_port: value.output.into() }
    }
}

impl Transformer {
    pub fn new(recipe: TransformerRecipe) -> Self {
        let ports = recipe.into_ports();
        Self { output: None, recipe, input_port: ports.input, output_port: ports.output }
    }
}

impl MachineTrait for Transformer {
    fn tick(&mut self, ticks: u64) {
        
    }

    fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Ok(())
    }

    fn is_starter(&self) -> bool {
        false
    }

    fn get_output(&mut self) -> Result<MachineOutput, MachineBindError> {
        if self.output_port.is_free() {
            Ok(MachineOutput { id: &mut self.output, port: &mut self.output_port, item_type: self.recipe.output })
        } else {
            return Err(MachineBindError::NoFreeOutputs)
        }
    }

    fn get_matching_input(&mut self, output_handle: &MachineOutput) -> Result<MachineInput, MachineBindError> {
        if self.input_port.is_free() {
            if output_handle.item_type == self.input_port.item_type {
                Ok(MachineInput { port: &mut self.input_port })
            } else {
                Err(MachineBindError::InvalidInput)
            }
        } else {
            Err(MachineBindError::NoFreeInputs)
        }
    }
}

impl From<TransformerRecipe> for Transformer {
    fn from(value: TransformerRecipe) -> Self {
        Self { recipe: value, output: None, input_port: value.input.into(), output_port: value.output.into() }
    }
}

impl Combinator {
    pub fn new(recipe: CombinatorRecipe) -> Self {
        let ports = recipe.into_ports();
        Self { output: None, recipe, input_ports: ports.inputs, output_port: ports.output }
    }
}

impl MachineTrait for Combinator {
    fn tick(&mut self, ticks: u64) {
        
    }

    fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Ok(())
    }

    fn is_starter(&self) -> bool {
        false
    }

    fn get_output(&mut self) -> Result<MachineOutput, MachineBindError> {
        if self.output_port.is_free() {
            Ok(MachineOutput { id: &mut self.output, port: &mut self.output_port, item_type: self.recipe.output })
        } else { 
            return Err(MachineBindError::NoFreeOutputs)
        }
    }

    fn get_matching_input(&mut self, output_handle: &MachineOutput) -> Result<MachineInput, MachineBindError> {
        if self.input_ports.0.is_free() && self.input_ports.0.item_type == output_handle.item_type {
            return Ok(MachineInput { port: &mut self.input_ports.0 })
        }
        
        if self.input_ports.1.is_free() && self.input_ports.1.item_type == output_handle.item_type {
            return Ok(MachineInput { port: &mut self.input_ports.1 })
        }

        return Err(MachineBindError::NoFreeInputs)
    }
}

impl From<CombinatorRecipe> for Combinator {
    fn from(value: CombinatorRecipe) -> Self {
        Self { recipe: value, output: None, input_ports: (value.inputs.0.into(), value.inputs.1.into()), output_port: value.output.into() }
    }
}

impl Separator {
    pub fn new(recipe: SeparatorRecipe) -> Self {
        let ports = recipe.into_ports();
        Self { outputs: (None, None), recipe, input_port: ports.input, output_ports: ports.outputs }
    }
}

impl MachineTrait for Separator {
    fn tick(&mut self, ticks: u64) {
        
    }

    fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Ok(())
    }

    fn is_starter(&self) -> bool {
        false
    }

    fn get_output(&mut self) -> Result<MachineOutput, MachineBindError> {
        match self.outputs {
            (None, _) => Ok(MachineOutput { id: &mut self.outputs.0, port: &mut self.output_ports.0, item_type: self.recipe.outputs.0 }),
            (_, None) => Ok(MachineOutput { id: &mut self.outputs.1, port: &mut self.output_ports.1, item_type: self.recipe.outputs.1 }),
            _ => return Err(MachineBindError::NoFreeOutputs),
        }
    }

    fn get_matching_input(&mut self, output_handle: &MachineOutput) -> Result<MachineInput, MachineBindError> {
        if self.input_port.is_free() && self.input_port.item_type == output_handle.item_type{
            Ok(MachineInput { port: &mut self.input_port })
        } else {
            return Err(MachineBindError::NoFreeInputs)
        }
    }
}

impl From<SeparatorRecipe> for Separator {
    fn from(value: SeparatorRecipe) -> Self {
        Self { recipe: value, outputs: (None, None), input_port: value.input.into(), output_ports: (value.outputs.0.into(), value.outputs.1.into()) }
    }
}

impl Storage {
    pub fn new(item_type: ItemType) -> Self {
        Self { input_port: IoPort::with_capacity(item_type, 1000) }
    }
}

impl MachineTrait for Storage {
    fn tick(&mut self, ticks: u64) {
        
    }

    fn push(&mut self, item_type: ItemType, amount: u64) -> Result<(), MachinePushError> {
        Ok(())
    }

    fn is_starter(&self) -> bool {
        false
    }

    fn get_output(&mut self) -> Result<MachineOutput, MachineBindError> {
        return Err(MachineBindError::NoFreeOutputs)
    }

    fn get_matching_input(&mut self, output: &MachineOutput) -> Result<MachineInput, MachineBindError> {
        if self.input_port.is_free() && self.input_port.item_type == output.item_type{
            Ok(MachineInput { port: &mut self.input_port })
        } else {
            return Err(MachineBindError::NoFreeInputs)
        }
    }
}