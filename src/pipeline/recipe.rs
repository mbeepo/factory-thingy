use crate::{pipeline::{machine::MachineKind, IoPort}, ItemType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RecipeNew {
    pub machine_kind: MachineKind,
    pub ticks: u64,
    pub inputs: [Option<ItemType>; 4],
    pub outputs: [Option<ItemType>; 4],
}

impl RecipeNew {
    pub fn producer_recipe(output: ItemType, ticks: u64) -> Self {
        Self { machine_kind: MachineKind::Producer, ticks, inputs: [None; 4], outputs: [Some(output), None, None, None] }
    }

    pub fn transformer_recipe(input: ItemType, output: ItemType, ticks: u64) -> Self {
        Self { machine_kind: MachineKind::Transformer, ticks, inputs: [Some(input), None, None, None], outputs: [Some(output), None, None, None] }
    }

    pub fn combinator_recipe(inputs: (ItemType, ItemType), output: ItemType, ticks: u64) -> Self {
        Self { machine_kind: MachineKind::Combinator, ticks, inputs: [Some(inputs.0), Some(inputs.1), None, None], outputs: [Some(output), None, None, None] }
    }

    pub fn separator_recipe(input: ItemType, outputs: (ItemType, ItemType), ticks: u64) -> Self {
        Self { machine_kind: MachineKind::Separator, ticks, inputs: [Some(input), None, None, None], outputs: [Some(outputs.0), Some(outputs.1), None, None] }
    }

    pub fn storage_recipe() -> Self {
        Self { machine_kind: MachineKind::Storage, ticks: 0, inputs: [None; 4], outputs: [None; 4] }
    }
}

#[derive(Clone, Debug)]
pub struct Recipes {
    pub inner: Vec<RecipeNew>,
}

impl Recipes {
    pub fn init() -> Self {
        Self { inner: vec![
            RecipeNew::producer_recipe(ItemType::Input, 10),
            RecipeNew::producer_recipe(ItemType::Output, 10),
            RecipeNew::transformer_recipe(ItemType::Input, ItemType::Storage, 20),
            RecipeNew::transformer_recipe(ItemType::Output, ItemType::Producer, 20),
            RecipeNew::combinator_recipe((ItemType::Input, ItemType::Output), ItemType::Transformer, 60),
            RecipeNew::combinator_recipe((ItemType::Transformer, ItemType::Input), ItemType::Combinator, 60),
            RecipeNew::combinator_recipe((ItemType::Transformer, ItemType::Output), ItemType::Separator, 60),
        ] }
    }

    pub fn get_producer(&self, output: ItemType) -> Option<RecipeNew> {
        self.inner.iter().find_map(|e| { if e.machine_kind == MachineKind::Producer && e.outputs[0] == Some(output) { Some(*e) } else { None }})
    }

    pub fn get_transformer(&self, output: ItemType) -> Option<RecipeNew> {
        self.inner.iter().find_map(|e| { if e.machine_kind == MachineKind::Transformer && e.outputs[0] == Some(output) { Some(*e) } else { None }})
    }
    
    pub fn get_combinator(&self, inputs: (ItemType, ItemType), output: ItemType) -> Option<RecipeNew> {
        self.inner.iter().find_map(|e| {
            if e.machine_kind == MachineKind::Combinator
            && (e.inputs[0], e.inputs[1]) == (Some(inputs.0), Some(inputs.1))
            && e.outputs[0] == Some(output) { Some(*e) } else { None }})
    }

    pub fn get_separator(&self, input: ItemType, outputs: (ItemType, ItemType)) -> Option<RecipeNew> {
        self.inner.iter().find_map(|e| {
            if e.machine_kind == MachineKind::Combinator
            && (e.outputs[0], e.outputs[1]) == (Some(outputs.0), Some(outputs.1))
            && e.inputs[0] == Some(input) { Some(*e) } else { None }})
    }
}