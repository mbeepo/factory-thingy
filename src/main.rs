use crate::pipeline::{machine::{craft, push_outputs, ready_craft, tick_crafts, InputBank, InputBuffers, InputPort, Machine, MachineKind, MachineStatus, Mult, OutputBank, OutputBuffers, OutputPort, Producer}, recipe::Recipes, IoBuffer};
use bevy::prelude::*;

mod pipeline;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ItemType {
    Producer,
    Transformer,
    Combinator,
    Separator,
    Storage,
    Input,
    Output,
}


// fn main() -> eframe::Result {
fn main() {
    let recipes = Recipes::init();

    // Our application state:
    // let mut factory: Vec<pipeline::Pipeline> = Vec::with_capacity(4);
    // {
    //     let mut pipeline1 = Pipeline::with_capacity(3);
    //     let producer1 = pipeline1.push(recipes.get_producer(ItemType::Output).unwrap().into());
    //     let producer2 = pipeline1.push(recipes.get_producer(ItemType::Input).unwrap().into());
    //     let producer3 = pipeline1.push(recipes.get_producer(ItemType::Input).unwrap().into());
    //     let combinator1 = pipeline1.push(recipes.get_combinator(ItemType::Transformer).unwrap().into());
    //     let combinator2 = pipeline1.push(recipes.get_combinator(ItemType::Combinator).unwrap().into());

    //     pipeline1.bind_output(producer1, combinator1).unwrap();
    //     pipeline1.bind_output(producer2, combinator1).unwrap();
    //     pipeline1.bind_output(combinator1, combinator2).unwrap();
    //     pipeline1.bind_output(producer3, combinator2).unwrap();

    //     pipeline1.set_mult(&producer1, 5).unwrap();
    //     pipeline1.set_mult(&producer2, 5).unwrap();
    //     pipeline1.set_mult(&producer3, 5).unwrap();
    //     pipeline1.set_mult(&combinator1, 5).unwrap();
        
    //     factory.push(pipeline1);
    // }

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (ready_craft, tick_crafts, craft, push_outputs).chain())
        .insert_resource(Time::<Fixed>::from_seconds(0.1))
        .insert_resource(recipes)
        .run();
}

fn setup(mut commands: Commands, recipes: Res<Recipes>) {
    let producer1 = commands.spawn((Producer {
        output_bank: OutputBank::with_capacity(1),
        output_buffers: OutputBuffers(vec![IoBuffer::new(ItemType::Input)]),
        recipe: recipes.get_producer(ItemType::Input).unwrap(),
        status: MachineStatus::Idle,
    }, Mult(5), MachineKind::Producer)).id();
    let producer2 = commands.spawn((Producer {
        output_bank: OutputBank::with_capacity(1),
        output_buffers: OutputBuffers(vec![ItemType::Input.into()]),
        recipe: recipes.get_producer(ItemType::Input).unwrap(),
        status: MachineStatus::Idle,
    }, Mult(5), MachineKind::Producer)).id();
    let producer3 = commands.spawn((Producer {
        output_bank: OutputBank::with_capacity(1),
        output_buffers: OutputBuffers(vec![ItemType::Output.into()]),
        recipe: recipes.get_producer(ItemType::Output).unwrap(),
        status: MachineStatus::Idle,
    }, Mult(5), MachineKind::Producer)).id();
    
    let combinator1 = commands.spawn(Machine {
        kind: MachineKind::Combinator,
        input_bank: InputBank::with_capacity(2),
        input_buffers: InputBuffers(vec![ItemType::Input.into(), ItemType::Output.into()]),
        output_bank: OutputBank::with_capacity(1),
        output_buffers: OutputBuffers(vec![ItemType::Transformer.into()]),
        recipe: recipes.get_combinator(ItemType::Transformer).unwrap(),
        status: MachineStatus::Idle,
    }).id();
    let combinator2 = commands.spawn(Machine {
        kind: MachineKind::Combinator,
        input_bank: InputBank::with_capacity(2),
        input_buffers: InputBuffers(vec![ItemType::Transformer.into(), ItemType::Input.into()]),
        output_bank: OutputBank::with_capacity(1),
        output_buffers: OutputBuffers(vec![ItemType::Combinator.into()]),
        recipe: recipes.get_combinator(ItemType::Combinator).unwrap(),
        status: MachineStatus::Idle,
    }).id();

    commands.spawn((OutputPort(producer1), InputPort(combinator1)));
    commands.spawn((OutputPort(producer2), InputPort(combinator2)));
    commands.spawn((OutputPort(producer3), InputPort(combinator1)));
    commands.spawn((OutputPort(combinator1), InputPort(combinator2)));
}