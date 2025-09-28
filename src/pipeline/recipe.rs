use crate::{pipeline::{machine::MachineKind, IoPort}, ItemType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RecipeNew {
    pub machine_kind: MachineKind,
    pub ticks: u64,
    pub inputs: [Option<ItemStack>; 4],
    pub outputs: [Option<ItemStack>; 4],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub amount: u64,
}

impl ItemStack {
    pub fn new(item_type: ItemType, amount: u64) -> Self {
        Self { item_type, amount }
    }
}

impl RecipeNew {
    pub fn producer_recipe(output: ItemStack, ticks: u64) -> Self {
        Self { machine_kind: MachineKind::Producer, ticks, inputs: [None; 4], outputs: [Some(output), None, None, None] }
    }

    pub fn transformer_recipe(input: ItemStack, output: ItemStack, ticks: u64) -> Self {
        Self { machine_kind: MachineKind::Transformer, ticks, inputs: [Some(input), None, None, None], outputs: [Some(output), None, None, None] }
    }

    pub fn combinator_recipe(inputs: (ItemStack, ItemStack), output: ItemStack, ticks: u64) -> Self {
        Self { machine_kind: MachineKind::Combinator, ticks, inputs: [Some(inputs.0), Some(inputs.1), None, None], outputs: [Some(output), None, None, None] }
    }

    pub fn separator_recipe(input: ItemStack, outputs: (ItemStack, ItemStack), ticks: u64) -> Self {
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
            RecipeNew::producer_recipe(ItemStack::new(ItemType::Input, 1), 10),
            RecipeNew::producer_recipe(ItemStack::new(ItemType::Output, 1), 10),
            RecipeNew::transformer_recipe(ItemStack::new(ItemType::Input, 5), ItemStack::new(ItemType::Storage, 1), 20),
            RecipeNew::transformer_recipe(ItemStack::new(ItemType::Output, 5), ItemStack::new(ItemType::Producer, 1), 20),
            RecipeNew::combinator_recipe((ItemStack::new(ItemType::Input, 5), ItemStack::new(ItemType::Output, 5)), ItemStack::new(ItemType::Transformer, 1), 60),
            RecipeNew::combinator_recipe((ItemStack::new(ItemType::Transformer, 1), ItemStack::new(ItemType::Input, 5)), ItemStack::new(ItemType::Combinator, 1), 60),
            RecipeNew::combinator_recipe((ItemStack::new(ItemType::Transformer, 1), ItemStack::new(ItemType::Output, 5)), ItemStack::new(ItemType::Separator, 1), 60),
        ] }
    }

    pub fn get_producer(&self, output: ItemType) -> Option<RecipeNew> {
        self.inner.iter().find_map(|e| { if e.machine_kind == MachineKind::Producer && e.outputs[0].map(|inner| inner.item_type) == Some(output) { Some(*e) } else { None }})
    }

    pub fn get_transformer(&self, output: ItemType) -> Option<RecipeNew> {
        self.inner.iter().find_map(|e| { if e.machine_kind == MachineKind::Transformer && e.outputs[0].map(|inner| inner.item_type) == Some(output) { Some(*e) } else { None }})
    }
    
    pub fn get_combinator(&self, output: ItemType) -> Option<RecipeNew> {
        self.inner.iter().find_map(|e| {
            if e.machine_kind == MachineKind::Combinator
            // && (e.inputs[0].map(|inner| inner.item_type), e.inputs[1].map(|inner| inner.item_type)) == (Some(inputs.0), Some(inputs.1))
            && e.outputs[0].map(|inner| inner.item_type) == Some(output) { Some(*e) } else { None }})
    }

    pub fn get_separator(&self, outputs: (ItemType, ItemType)) -> Option<RecipeNew> {
        self.inner.iter().find_map(|e| {
            if e.machine_kind == MachineKind::Combinator
            && (e.outputs[0].map(|inner| inner.item_type), e.outputs[1].map(|inner| inner.item_type)) == (Some(outputs.0), Some(outputs.1)) { Some(*e) } else { None }})
            // && e.inputs[0].map(|inner| inner.item_type) == Some(input) { Some(*e) } else { None }})
    }
}