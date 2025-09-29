use eframe::egui;

use crate::pipeline::{machine::Machine, recipe::Recipes, Pipeline};
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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    
    let recipes = Recipes::init();

    // Our application state:
    let mut factory: Vec<pipeline::Pipeline> = Vec::with_capacity(4);
    {
        let mut pipeline1 = Pipeline::with_capacity(3);
        let producer1 = pipeline1.push(recipes.get_producer(ItemType::Output).unwrap().into());
        let producer2 = pipeline1.push(recipes.get_producer(ItemType::Input).unwrap().into());
        let producer3 = pipeline1.push(recipes.get_producer(ItemType::Input).unwrap().into());
        let combinator1 = pipeline1.push(recipes.get_combinator(ItemType::Transformer).unwrap().into());
        let combinator2 = pipeline1.push(recipes.get_combinator(ItemType::Combinator).unwrap().into());

        pipeline1.bind_output(producer1, combinator1).unwrap();
        pipeline1.bind_output(producer2, combinator1).unwrap();
        pipeline1.bind_output(combinator1, combinator2).unwrap();
        pipeline1.bind_output(producer3, combinator2).unwrap();

        pipeline1.set_mult(&producer1, 5).unwrap();
        pipeline1.set_mult(&producer2, 5).unwrap();
        pipeline1.set_mult(&producer3, 5).unwrap();
        pipeline1.set_mult(&combinator1, 5).unwrap();
        
        factory.push(pipeline1);
    }

    App::new().run();
    // eframe::run_simple_native("beepo app", options, move |ctx, _frame| {
    //     egui::CentralPanel::default().show(ctx, |ui| {
    //         ui.heading("Beepo Factory");
    //         ui.label("Under Construction :3");
    //     });
    // })
}