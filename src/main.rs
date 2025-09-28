use eframe::egui;

use crate::pipeline::{machine::Machine, recipe::Recipes, Pipeline};

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


fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    
    let recipes = Recipes::init();

    // Our application state:
    let mut factory: Vec<pipeline::Pipeline> = Vec::with_capacity(4);
    {
        let mut pipeline1 = Pipeline::with_capacity(3);
        let producer = pipeline1.push(recipes.get_producer(ItemType::Output).unwrap().into());
        let transformer = pipeline1.push(recipes.get_transformer(ItemType::Producer).unwrap().into());
        let storage = pipeline1.push(Machine::new_storage(ItemType::Producer));

        pipeline1.bind_output(producer, transformer).unwrap();
        pipeline1.bind_output(transformer, storage).unwrap();
        
        factory.push(pipeline1);
    }

    for i in 0..100 {
        println!("{i}");
        factory[0].tick();
    }

    eframe::run_simple_native("beepo app", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Beepo Factory");
            ui.label("Under Construction :3");
        });
    })
}