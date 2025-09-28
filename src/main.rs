use eframe::egui;

use crate::pipeline::{machine::Machine, recipe::{CombinerRecipes, ProducerRecipes, TransformerRecipes}, Pipeline};

mod pipeline;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ItemType {
    Producer,
    Transformer,
    Combiner,
    Splitter,
    Storage,
    Input,
    Output,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum MachineType {
    Producer,
    Transformer,
    Combiner,
    Splitter,
    Storage,
}

#[derive(Clone, Copy, Debug, Default)]
struct CentralStorage {
    producers: u64,
    transformers: u64,
    combiners: u64,
    splitters: u64,
    storages: u64,
}


fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    let producer_recipes = ProducerRecipes::init();
    let transformer_recipes = TransformerRecipes::init();
    let combiner_recipes = CombinerRecipes::init();

    // Our application state:
    let mut central_storage = CentralStorage::default();
    let mut factory: Vec<pipeline::Pipeline> = Vec::with_capacity(4);
    {
        let mut pipeline1 = Pipeline::with_capacity(3);
        let producer = pipeline1.push(Machine::new(producer_recipes.get(ItemType::Output).unwrap().into()));
        let transformer = pipeline1.push(Machine::new(transformer_recipes.get(ItemType::Producer).unwrap().into()));
        let storage = pipeline1.push(Machine::new_storage(ItemType::Producer));

        pipeline1.bind_output(producer, transformer).unwrap();
        pipeline1.bind_output(transformer, storage).unwrap();
        
        
        factory.push(pipeline1);
    }


    eframe::run_simple_native("beepo app", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Beepo Factory");
            ui.columns_const(|[col1, col2]| {
                col1.vertical(|ui| {
                    ui.label("Producers: ");
                    ui.label("Transformers: ");
                    ui.label("Combiners: ");
                    ui.label("Splitters: ");
                    ui.label("Storages: ");
                });
                col2.vertical(|ui| {
                    ui.monospace(central_storage.producers.to_string());
                    ui.monospace(central_storage.transformers.to_string());
                    ui.monospace(central_storage.combiners.to_string());
                    ui.monospace(central_storage.splitters.to_string());
                    ui.monospace(central_storage.storages.to_string());
                });
            });
        });
    })
}