use crate::pipeline::{machine::{craft, push_outputs, ready_craft, tick_crafts, BufferType, InputBank, InputBufferText, InputBuffers, InputPort, MachineCoupling, MachineStatus, OutputBank, OutputBufferText, OutputBuffers, OutputPort, StatusText}, recipe::{Recipe, Recipes}};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

mod pipeline;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemType {
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
        .add_systems(FixedUpdate, (ready_craft, tick_crafts, craft, push_outputs, update_labels).chain())
        .insert_resource(Time::<Fixed>::from_seconds(0.1))
        .insert_resource(recipes)
        .run();
}

fn setup(mut commands: Commands, recipes: Res<Recipes>) {
    let producer1 = spawn_machine(&mut commands, recipes.get_producer(ItemType::Input).unwrap());
    let producer2 = spawn_machine(&mut commands, recipes.get_producer(ItemType::Output).unwrap());
    let producer3 = spawn_machine(&mut commands, recipes.get_producer(ItemType::Input).unwrap());
    let combinator1 = spawn_machine(&mut commands, recipes.get_combinator(ItemType::Transformer).unwrap());
    let combinator2 = spawn_machine(&mut commands, recipes.get_combinator(ItemType::Combinator).unwrap());

    bind_output(&mut commands, producer1, combinator1, ItemType::Input);
    bind_output(&mut commands, producer3, combinator2, ItemType::Input);
    bind_output(&mut commands, producer2, combinator1, ItemType::Output);
    bind_output(&mut commands, combinator1, combinator2, ItemType::Transformer);

    commands.spawn(Camera2d);

    const WIDTH: f32 = 200.0;
    const HEIGHT: f32 = 150.0;
    create_label(&mut commands, "Producer", producer1, Vec2::new(WIDTH*0.0, HEIGHT*0.0), Vec2::new(WIDTH, HEIGHT));
    create_label(&mut commands, "Producer", producer2, Vec2::new(WIDTH*0.0, HEIGHT*1.5), Vec2::new(WIDTH, HEIGHT));
    create_label(&mut commands, "Combinator", combinator1, Vec2::new(WIDTH*1.5, HEIGHT*0.75), Vec2::new(WIDTH, HEIGHT));
    create_label(&mut commands, "Producer", producer3, Vec2::new(WIDTH*1.5, HEIGHT*2.25), Vec2::new(WIDTH, HEIGHT));
    create_label(&mut commands, "Combinator", combinator2, Vec2::new(WIDTH*3.0, HEIGHT*1.5), Vec2::new(WIDTH, HEIGHT));
}

pub fn create_label(commands: &mut Commands, name: &str, entity: Entity, position: Vec2, size: Vec2) {
    commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(px(5)),
            row_gap: px(5),
            left: px(position.x),
            top: px(position.y),
            width: px(size.x),
            height: px(size.y),
            ..Default::default()
        },
        BackgroundColor(Color::BLACK),
        BorderRadius::all(px(5)),
    )).with_children(|builder| {
        builder.spawn((Text::new(name), TextFont {
            font_size: 12.0,
            ..default()
        }));
        let status_text = StatusText(builder.spawn((Text::new(""), TextFont {
            font_size: 12.0,
            ..default()
        })).id());
        let input_buffer_text = InputBufferText(builder.spawn((Text::new(""), TextFont {
            font_size: 12.0,
            ..default()
        })).id());
        let output_buffer_text = OutputBufferText(builder.spawn((Text::new(""), TextFont {
            font_size: 12.0,
            ..default()
        })).id());

        builder.commands().entity(entity).insert((status_text, input_buffer_text, output_buffer_text));
    });
}

pub fn update_labels(machine_query: Query<(&InputBufferText, &OutputBufferText, Option<&InputBuffers>, Option<&OutputBuffers>, &StatusText, &MachineStatus)>, mut label_query: Query<&mut Text>) {
    for (input_label, output_label, input_buf, output_buf, status_label, status) in machine_query {
        if let Some(input_buf) = input_buf {
            let mut input_label = label_query.get_mut(input_label.0).unwrap();
            let mut text = String::from("Input");
            for input in &input_buf.0 {
                text = format!("{}\n{:?} - {}/{}", text, input.item_type, input.buffer.current, input.buffer.max);
            }

            input_label.0 = text;
        }

        if let Some(output_buf) = output_buf {
            let mut output_label = label_query.get_mut(output_label.0).unwrap();
            let mut text = String::from("Output");
            for output in &output_buf.0 {
                text = format!("{}\n{:?} - {}/{}", text, output.item_type, output.buffer.current, output.buffer.max);
            }

            output_label.0 = text;
        }

        let mut status_label = label_query.get_mut(status_label.0).unwrap();
        status_label.0 = String::from(*status);
    }
}

pub fn spawn_machine(commands: &mut Commands, recipe: Recipe) -> Entity {
    let machine = commands.spawn((
        recipe.machine_kind,
        recipe,
        MachineStatus::Idle,
    )).id();

    let input_buffers = InputBuffers(recipe.inputs.iter().filter_map(|input| {
        if let Some(input) = input {
            Some(input.item_type.into())
        } else {
            None
        }
    }).collect());

    if input_buffers.0.len() > 0 {
        let input_bank = InputBank::with_capacity(input_buffers.0.len());
        commands.entity(machine).insert((input_buffers, input_bank));
    }

    let output_buffers = OutputBuffers(recipe.outputs.iter().filter_map(|output| {
        if let Some(output) = output {
            Some(output.item_type.into())
        } else {
            None
        }
    }).collect());

    if output_buffers.0.len() > 0 {
        let output_bank = OutputBank::with_capacity(output_buffers.0.len());
        commands.entity(machine).insert((output_buffers, output_bank));
    }

    machine
}

pub fn bind_output(commands: &mut Commands, src: Entity, dest: Entity, item_type: ItemType) {
    commands.spawn(MachineCoupling { output_port: OutputPort(src), input_port: InputPort(dest), buffer_type: BufferType(item_type) });
}