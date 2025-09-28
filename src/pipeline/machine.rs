use std::fmt::Debug;

use crate::{pipeline::{recipe::RecipeNew, IoPort, PipelineId}, ItemType};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineStatus {
    Working(u64),
    Full,
    LacksInput,
    Ready,
    CraftFinished,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CraftStatus {
    Incomplete,
    Complete,
}

#[derive(Clone, Debug)]
pub struct Machine {
    pub kind: MachineKind,
    recipe: RecipeNew,
    status: MachineStatus,
    input_ports: [Option<IoPort>; 4],
    output_ports: [Option<IoPort>; 4],
    outputs: [Option<PipelineId>; 4],
}

impl Machine {
    pub fn tick(&mut self) -> CraftStatus {
        match self.status {
            MachineStatus::Working(ref mut ticks_remaining) => {
                *ticks_remaining -= 1;

                if *ticks_remaining == 0 {
                    CraftStatus::Complete
                } else {
                    CraftStatus::Incomplete
                }
            },
            MachineStatus::Ready => {
                if self.recipe.ticks == 1 {
                    CraftStatus::Complete
                } else {
                    self.status = MachineStatus::Working(self.recipe.ticks - 1);
                    CraftStatus::Incomplete
                }
            },
            _ => CraftStatus::Incomplete,
        }
    }

    pub fn is_starter(&self) -> bool {
        match self.kind {
            MachineKind::Producer => true,
            _ => false,
        }
    }

    pub fn outputs(&self) -> Vec<ItemType> {
        self.recipe.outputs.iter().filter_map(|inner| *inner).collect()
    }

    pub fn inputs(&self) -> Vec<ItemType> {
        self.recipe.inputs.iter().filter_map(|inner| *inner).collect()
    }

    pub fn get_output(&mut self, item_type: ItemType) -> Result<MachineOutput, MachineBindError> {
        let mut idx: Option<usize> = None;
        for i in 0..4 {
            if let Some(port) = &self.output_ports[i] {
                if port.is_free() && port.item_type == item_type {
                    idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = idx {
            Ok(MachineOutput {
                id: &mut self.outputs[idx],
                port: &mut self.output_ports[idx],
                item_type: item_type,
            })
        } else {
            Err(MachineBindError::NoFreeOutputs)
        }
    }

    pub fn get_input(&mut self, item_type: ItemType) -> Result<MachineInput, MachineBindError> {
        let mut idx: Option<usize> = None;
        for i in 0..4 {
            if let Some(port) = &self.input_ports[i] {
                if port.is_free() && port.item_type == item_type {
                    idx = Some(i);
                    break;
                }
            }
        }

        if let Some(idx) = idx {
            Ok(MachineInput {
                port: &mut self.input_ports[idx],
            })
        } else {
            Err(MachineBindError::NoFreeOutputs)
        }
    }

    pub fn new_storage(item_type: ItemType) -> Self {
        Self {
            kind: MachineKind::Storage,
            recipe: RecipeNew::storage_recipe(),
            status: MachineStatus::LacksInput,
            input_ports: [Some(item_type.into()), None, None, None],
            output_ports: [None; 4],
            outputs: [None; 4],
        }
    }
}

impl From<RecipeNew> for Machine {
    fn from(value: RecipeNew) -> Self {
        let mut input_ports = value.inputs.iter().map(|input| input.map(|item_type| item_type.into())).take(4);
        let input_ports = [input_ports.next().flatten(), input_ports.next().flatten(), input_ports.next().flatten(), input_ports.next().flatten()];
        let mut output_ports = value.outputs.iter().map(|output| output.map(|item_type| item_type.into())).take(4);
        let output_ports = [output_ports.next().flatten(), output_ports.next().flatten(), output_ports.next().flatten(), output_ports.next().flatten()];

        Self {
            kind: value.machine_kind,
            recipe: value,
            status: MachineStatus::LacksInput,
            input_ports,
            output_ports,
            outputs: [None; 4],
        }
    }
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
    pub port: &'a mut Option<IoPort>,
    pub item_type: ItemType,
}

pub struct MachineInput<'a> {
    pub port: &'a mut Option<IoPort>,
}