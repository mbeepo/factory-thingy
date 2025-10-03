use std::{fmt::Debug, ops::{BitOr, BitOrAssign}};

use bevy::ecs::{component::Component, entity::UniqueEntityVec};
use bevy::prelude::*;

use crate::{pipeline::{recipe::Recipe, IoBuffer}, ItemType};

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

impl From<MachineStatus> for String {
    fn from(value: MachineStatus) -> Self {
        match value {
            MachineStatus::Working(Working { ticks_remaining, amount }) => format!("Crafting x{amount}: {ticks_remaining} left"),
            MachineStatus::Full => String::from("Full"),
            MachineStatus::Idle => String::from("Idle"),
            MachineStatus::LacksInput => String::from("Waiting for input"),
            _ => String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Working {
    ticks_remaining: u64,
    amount: u64,
}

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = InputPort)]
/// Connects a MachineCoupling to a Machine
pub struct InputBank(Vec<Entity>);

#[derive(Component, Clone, Copy, Debug)]
#[relationship(relationship_target = InputBank)]
/// Connects a MachineCoupling to an InputBank
pub struct InputPort(pub Entity);

#[derive(Component, Clone, Debug)]
#[relationship_target(relationship = OutputPort)]
/// Connects a Machine to an MachineCoupling
pub struct OutputBank(Vec<Entity>);

#[derive(Component, Clone, Debug)]
#[relationship(relationship_target = OutputBank)]
/// Connects an OutputBank to a MachineCoupling
pub struct OutputPort(pub Entity);

#[derive(Bundle, Clone, Debug)]
pub struct MachineCoupling {
    pub input_port: InputPort,
    pub output_port: OutputPort,
    pub buffer_type: BufferType,
}

#[derive(Component, Clone, Debug)]
pub struct BufferType(pub ItemType);

impl From<ItemType> for BufferType {
    fn from(value: ItemType) -> Self {
        Self(value)
    }
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

#[derive(Component, Clone, Debug)]
pub struct InputBufferText(pub Entity);

#[derive(Component, Clone, Debug)]
pub struct OutputBufferText(pub Entity);

#[derive(Component, Clone, Debug)]
pub struct StatusText(pub Entity);

pub fn tick_crafts(mut machine_query: Query<(&mut MachineStatus)>) {
    for entity in &mut machine_query {
        let (mut status) = entity;
        match *status {
            MachineStatus::Working(Working { ref mut ticks_remaining, amount}) => {
                *ticks_remaining -= 1;

                if *ticks_remaining == 0 {
                    *status = MachineStatus::CraftsFinished(amount);
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

pub fn ready_craft(mut machine_query: Query<(Option<&mut InputBuffers>, &OutputBuffers, &mut MachineStatus, &Recipe, Option<&Mult>)>, ) {
    for (mut inputs, outputs, mut status, recipe, mult) in &mut machine_query.iter_mut().filter(|(_, _, status, _, _)| **status == MachineStatus::Idle) {
        let mut possible_crafts = mult.unwrap_or(&Mult(1)).0;

        if let Some(inputs) = &mut inputs {
            for input in recipe.inputs.iter().filter_map(|i| *i) {
                let buffered = inputs.0.iter().fold(0, |acc, port| {
                    if port.item_type == input.item_type {
                        acc + port.buffer.current
                    } else {
                        acc
                    }
                });

                possible_crafts = possible_crafts.min(buffered / input.amount);
                if possible_crafts == 0 { break; }
            }
        }

        for output in recipe.outputs.iter().filter_map(|o| *o) {
            let bufferable = outputs.0.iter().fold(0, |acc, port| {
                if port.item_type == output.item_type {
                    acc + port.buffer.remaining()
                } else {
                    acc
                }
            });

            possible_crafts = possible_crafts.min(bufferable / output.amount);
            if possible_crafts == 0 { break; }
        }

        if let Some(inputs) = &mut inputs {
            for input in recipe.inputs.iter().filter_map(|o| *o) {
                let mut taken = input.amount * possible_crafts;

                for input in inputs.0.iter_mut().filter(|i| i.item_type == input.item_type) {
                    let takeable = taken.min(input.buffer.current);
                    input.buffer.current -= takeable;
                    taken -= takeable;
                    if taken == 0 { break; }
                }
            }
        }

        if possible_crafts > 0 {
            *status = MachineStatus::Working(Working { ticks_remaining: recipe.ticks, amount: possible_crafts });
        }
    }
}

pub fn push_outputs(mut src_query: Query<(&mut OutputBuffers, &OutputBank)>, coupling_query: Query<(&InputPort, &BufferType)>, mut dest_query: Query<&mut InputBuffers>) {
    for (mut output_buf, output_bank) in &mut src_query.iter_mut().filter(|(_, output_bank)| output_bank.len() > 0) {
        let couplings: Vec<(&InputPort, &BufferType)> = output_bank.iter().filter_map(|entity| coupling_query.get(entity).ok()).collect();
        for buf in output_buf.0.iter_mut().filter(|buf| buf.buffer.current > 0) {
            let Some(coupling) = couplings.iter().find(|(_, BufferType(item_type))| *item_type == buf.item_type) else { continue };
            let mut dest_buf = dest_query.get_mut(coupling.0.0).unwrap();
            let Some(dest_buf) = dest_buf.0.iter_mut().find(|b| b.item_type == buf.item_type) else { continue };
            let pushable = dest_buf.buffer.remaining().min(buf.buffer.current);
            
            buf.buffer.current -= pushable;
            dest_buf.buffer.current += pushable;
        }
    }
}
const DEFAULT_BUFFER_SIZE: u64 = 50;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemBuffer {
    pub current: u64,
    pub max: u64,
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