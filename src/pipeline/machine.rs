use std::{fmt::Debug, ops::{BitOr, BitOrAssign}};

use bevy::ecs::component::Component;
use bevy::prelude::*;

use crate::pipeline::{recipe::Recipe, IoBuffer};

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub enum MachinePushError {
//     NoSpace,
//     InvalidInput,
// }

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub enum MachineBindError {
//     NoFreeOutputs,
//     NoFreeInputs,
//     InputDoesNotExist,
//     OutputDoesNotExist,
//     InvalidInput,
// }

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineKind {
    Producer,
    Transformer,
    Combinator,
    Separator,
    Storage,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum MachineStatus {
    Working(Working),
    Full,
    LacksInput,
    CraftsFinished(u64),
    Idle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Working {
    ticks_remaining: u64,
    amount: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CraftStatus {
    Incomplete,
    Complete,
}

impl BitOr for CraftStatus {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        if self == CraftStatus::Complete || rhs == CraftStatus::Complete {
            CraftStatus::Complete
        } else {
            CraftStatus::Incomplete
        }
    }
}

impl BitOrAssign for CraftStatus {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = InputPort)]
/// Connects a MachineCoupling to a Machine
/// 
/// Entries relate 1:1 with those in the connected Machine's InputBuffers
pub struct InputBank(Vec<Entity>);

#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = InputBank)]
/// Connects a MachineCoupling to an InputBank
pub struct InputPort(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = OutputPort)]
/// Connects a Machine to an MachineCoupling
/// 
/// Entries relate 1:1 with those in the connected Machine's OutputBuffers
pub struct OutputBank(Vec<Entity>);

#[derive(Component, Clone, Debug)]
#[relationship(relationship_target = OutputBank)]
/// Connects an OutputBank to a MachineCoupling
pub struct OutputPort(pub Entity);

#[derive(Bundle, Clone, Debug)]
pub struct MachineCoupling {
    pub input_port: InputPort,
    pub output_port: OutputPort,
}

#[derive(Component, Clone, Debug)]
pub struct InputBuffers(pub Vec<IoBuffer>);

#[derive(Component, Clone, Debug)]
pub struct OutputBuffers(pub Vec<IoBuffer>);

#[derive(Component, Clone, Copy, Debug)]
pub struct Mult(pub u64);

impl InputBank {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

impl OutputBank {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
}

#[derive(Bundle, Clone, Debug)]
pub struct Machine {
    pub kind: MachineKind,
    pub input_bank: InputBank,
    pub input_buffers: InputBuffers,
    pub output_bank: OutputBank,
    pub output_buffers: OutputBuffers,
    pub recipe: Recipe,
    pub status: MachineStatus,
}

#[derive(Bundle, Clone, Debug)]
pub struct Producer {
    pub output_bank: OutputBank,
    pub output_buffers: OutputBuffers,
    pub recipe: Recipe,
    pub status: MachineStatus,
}

// #[derive(Clone, Debug)]
// pub struct Machine {
//     pub kind: MachineKind,
//     pub output_id: Option<PipelineId>,
//     recipe: Recipe,
//     status: MachineStatus,
//     pub input_ports: [Option<IoBuffer>; 4],
//     pub output_ports: [Option<IoBuffer>; 4],
//     pub mult: u64,
// }

pub fn tick_crafts(mut machine_query: Query<(&Recipe, &mut MachineStatus, Option<&Mult>)>) {
    for entity in &mut machine_query {
        let (recipe, mut status, maybe_mult) = entity;
        match *status {
            MachineStatus::Working(Working { ref mut ticks_remaining, amount}) => {
                if let Some(mult) = maybe_mult {
                    if mult.0 > *ticks_remaining {
                        *ticks_remaining = (mult.0 - *ticks_remaining) / recipe.ticks.max(1);
                        *status = MachineStatus::CraftsFinished(amount);
                    } else {
                        *ticks_remaining -= mult.0;
                    }
                } else {
                    *ticks_remaining -= 1;

                    if *ticks_remaining == 0 {
                        *status = MachineStatus::CraftsFinished(1);
                    }
                }
            },
            _ => {},
        }
    }
}

pub fn craft(mut machine_query: Query<(&mut OutputBuffers, &mut MachineStatus, &Recipe)>) {
    for (mut buffers, mut status, recipe, num_crafts) in machine_query.iter_mut().filter_map(|(buffers, status, recipe)| {
        if let MachineStatus::CraftsFinished(num_crafts) = *status {
            Some((buffers, status, recipe, num_crafts))
        } else {
            None
        }
    }) {
        for item_stack in recipe.outputs.iter().filter_map(|o| *o) {
            let buffer = buffers.0.iter_mut().find(|b| b.item_type == item_stack.item_type).expect(format!("No buffer for recipe output: {:?}", item_stack.item_type).as_ref());
            // Buffers were checked for space during ready phase, just let it overflow here
            buffer.buffer.current += item_stack.amount * num_crafts;
            println!("Crafted {:?} x{}", item_stack.item_type, item_stack.amount * num_crafts);
        }
        *status = MachineStatus::Idle;
    }
}

pub fn ready_craft(mut machine_query: Query<(&mut InputBuffers, &OutputBuffers, &mut MachineStatus, &Recipe, Option<&Mult>)>, ) {
    for (mut inputs, outputs, mut status, recipe, mult) in &mut machine_query {
        if *status != MachineStatus::Idle {
            continue;
        }
        let mut possible_crafts = mult.unwrap_or(&Mult(1)).0;

        for input in recipe.inputs.iter().filter_map(|i| *i) {
            let buffered = inputs.0.iter().fold(0, |acc, port| {
                if port.item_type == input.item_type {
                    acc + port.buffer.current
                } else {
                    acc
                }
            });

            let bungus = buffered / input.amount;
            if bungus == 0 {
                possible_crafts = 0;
                break;
            }

            possible_crafts = possible_crafts.min(bungus);
        }

        for output in recipe.outputs.iter().filter_map(|o| *o) {
            let bufferable = outputs.0.iter().fold(0, |acc, port| {
                if port.item_type == output.item_type {
                    acc + port.buffer.remaining()
                } else {
                    acc
                }
            });

            let bungus = bufferable / output.amount;
            if bungus == 0 {
                possible_crafts = 0;
                break;
            }

            possible_crafts = possible_crafts.min(bungus);
            println!("Crafting {:?} x{}", output.item_type, output.amount * possible_crafts);
        }

        for output in recipe.outputs.iter().filter_map(|o| *o) {
            let mut taken = output.amount * possible_crafts;

            for input in inputs.0.iter_mut().filter(|i| i.item_type == output.item_type) {
                let takeable = taken.min(input.buffer.current);
                input.buffer.current -= takeable;
                taken -= takeable;
                if taken == 0 { break; }
            }
        }

        *status = MachineStatus::Working(Working { ticks_remaining: recipe.ticks, amount: possible_crafts });
    }
}

pub fn push_outputs(mut src_query: Query<(&mut OutputBuffers, &OutputBank)>, coupling_query: Query<(&OutputPort, &InputPort)>, mut dest_query: Query<&mut InputBuffers>) {
    for (mut output_buf, output_bank) in &mut src_query.iter_mut().filter(|(_, output_bank)| output_bank.len() > 0) {
        for (idx, buf) in output_buf.0.iter_mut().filter(|buf| buf.buffer.current > 0).enumerate() {
            let coupling = coupling_query.get(output_bank.0[idx]).unwrap();
            let dest_buf = &mut dest_query.get_mut(coupling.1.0).unwrap().0[idx];
            let pushable = dest_buf.buffer.remaining().min(buf.buffer.current);
            
            buf.buffer.current -= pushable;
            dest_buf.buffer.current += pushable;
        }
    }
}

// impl Machine {
//     pub fn tick(&mut self) -> CraftStatus {
//         match self.status {
//             MachineStatus::Working(ref mut ticks_remaining) => {
//                 *ticks_remaining -= 1;

//                 if *ticks_remaining == 0 {
//                     self.try_craft();
//                     CraftStatus::Complete
//                 } else {
//                     CraftStatus::Incomplete
//                 }
//             },
//             MachineStatus::Ready => {
//                 if self.recipe.ticks == 1 {
//                     self.try_craft();
//                     CraftStatus::Complete
//                 } else if self.recipe.ticks == 0 {
//                     CraftStatus::Complete  
//                 } else {
//                     self.status = MachineStatus::Working(self.recipe.ticks - 1);
//                     CraftStatus::Incomplete
//                 }
//             },
//             MachineStatus::LacksInput => {
//                 self.try_ready();
//                 CraftStatus::Incomplete
//             },
//             _ => CraftStatus::Incomplete,
//         }
//     }

//     pub fn try_craft(&mut self) {
//         if self.status != MachineStatus::Working(0) {
//             return;
//         }

//         let mut ready = [false, false, false, false];
//         for i in 0..4 {
//             if let Some(item_stack) = self.recipe.outputs[i] {
//                 if let Some(port) = self.output_ports[i] {
//                     if item_stack.item_type != port.item_type {
//                         panic!("Invalid output");
//                     }

//                     if port.buffer.remaining() >= item_stack.amount {
//                         ready[i] = true;
//                     }
//                 } else {
//                     panic!("Missing input port");
//                 }
//             } else {
//                 ready[i] = true;
//             }
//         }

//         if ready.iter().all(|b| *b) {
//             for i in 0..4 {
//                 if let Some(item_stack) = self.recipe.outputs[i] {
//                     if let Some(port) = &mut self.output_ports[i] {
//                         port.buffer.current += item_stack.amount;
//                         println!("Crafted {:?} x{}", item_stack.item_type, item_stack.amount);
//                     }
//                 }
//             }

//             self.try_ready();
//         }
//     }

//     /// Attempts to ready a craft by taking items from input ports and setting machine status
//     pub fn try_ready(&mut self) {
//         let mut ready = [false, false, false, false];
//         for i in 0..4 {
//             if let Some(item_stack) = self.recipe.inputs[i] {
//                 if let Some(port) = self.input_ports[i] {
//                     if item_stack.item_type != port.item_type {
//                         panic!("Invalid input");
//                     }

//                     if port.buffer.current >= item_stack.amount {
//                         ready[i] = true;
//                     }
//                 } else {
//                     panic!("Missing input port");
//                 }
//             } else {
//                 ready[i] = true;
//             }
//         }

//         if ready.iter().all(|b| *b) {
//             for i in 0..4 {
//                 if let Some(item_stack) = self.recipe.outputs[i] {
//                     if let Some(port) = &mut self.input_ports[i] {
//                         port.buffer.current -= item_stack.amount;
//                     }
//                 }
//             }

//             self.status = MachineStatus::Ready;
//         } else {
//             self.status = MachineStatus::LacksInput;
//         }
//     }

//     pub fn is_starter(&self) -> bool {
//         match self.kind {
//             MachineKind::Producer => true,
//             _ => false,
//         }
//     }

//     pub fn outputs(&self) -> Vec<ItemStack> {
//         self.recipe.outputs.iter().filter_map(|inner| *inner).collect()
//     }

//     pub fn inputs(&self) -> Vec<ItemStack> {
//         self.recipe.inputs.iter().filter_map(|inner| *inner).collect()
//     }

//     pub fn get_output(&mut self, item_type: ItemType) -> Result<MachineOutput, MachineBindError> {
//         let mut idx: Option<usize> = None;
//         for i in 0..4 {
//             if let Some(port) = &self.output_ports[i] {
//                 if port.is_free() && port.item_type == item_type {
//                     idx = Some(i);
//                     break;
//                 }
//             }
//         }

//         if let Some(idx) = idx {
//             Ok(MachineOutput {
//                 id: &mut self.output_id,
//                 port: &mut self.output_ports[idx],
//                 item_type: item_type,
//             })
//         } else {
//             Err(MachineBindError::NoFreeOutputs)
//         }
//     }

//     pub fn get_input(&mut self, item_type: ItemType) -> Result<MachineInput, MachineBindError> {
//         let mut idx: Option<usize> = None;
//         for i in 0..4 {
//             if let Some(port) = &self.input_ports[i] {
//                 if port.is_free() && port.item_type == item_type {
//                     idx = Some(i);
//                     break;
//                 }
//             }
//         }

//         if let Some(idx) = idx {
//             Ok(MachineInput {
//                 port: &mut self.input_ports[idx],
//             })
//         } else {
//             Err(MachineBindError::NoFreeOutputs)
//         }
//     }

//     pub fn new_storage(item_type: ItemType) -> Self {
//         Self {
//             kind: MachineKind::Storage,
//             output_id: None,
//             recipe: Recipe::storage_recipe(),
//             status: MachineStatus::LacksInput,
//             input_ports: [Some(item_type.into()), None, None, None],
//             output_ports: [None; 4],
//             mult: 1,
//         }
//     }
// }

// impl From<Recipe> for Machine {
//     fn from(value: Recipe) -> Self {
//         let mut input_ports = value.inputs.iter().map(|input| input.map(|item_stack| item_stack.item_type.into())).take(4);
//         let input_ports = [input_ports.next().flatten(), input_ports.next().flatten(), input_ports.next().flatten(), input_ports.next().flatten()];
//         let mut output_ports = value.outputs.iter().map(|output| output.map(|item_stack| item_stack.item_type.into())).take(4);
//         let output_ports = [output_ports.next().flatten(), output_ports.next().flatten(), output_ports.next().flatten(), output_ports.next().flatten()];

//         Self {
//             kind: value.machine_kind,
//             output_id: None,
//             recipe: value,
//             status: MachineStatus::LacksInput,
//             input_ports,
//             output_ports,
//             mult: 1,
//         }
//     }
// }

const DEFAULT_BUFFER_SIZE: u64 = 50;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemBuffer {
    pub current: u64,
    max: u64,
}

impl ItemBuffer {
    pub fn new() -> Self {
        Self { current: 0, max: DEFAULT_BUFFER_SIZE }
    }

    pub fn with_capacity(max: u64) -> Self {
        Self { current: 0, max }
    }

    pub fn remaining(&self) -> u64 {
        self.max - self.current
    }
}

// pub struct MachineOutput<'a> {
//     pub id: &'a mut Option<PipelineId>,
//     pub port: &'a mut Option<IoBuffer>,
//     pub item_type: ItemType,
// }

// pub struct MachineInput<'a> {
//     pub port: &'a mut Option<IoBuffer>,
// }