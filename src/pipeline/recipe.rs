use crate::{pipeline::IoPort, ItemType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Recipe {
    Producer(ProducerRecipe),
    Transformer(TransformerRecipe),
    Combiner(CombinerRecipe),
    Splitter(SplitterRecipe),
}

impl From<ProducerRecipe> for Recipe {
    fn from(value: ProducerRecipe) -> Self {
        Self::Producer(value)
    }
}

impl From<TransformerRecipe> for Recipe {
    fn from(value: TransformerRecipe) -> Self {
        Self::Transformer(value)
    }
}

impl From<CombinatorRecipe> for Recipe {
    fn from(value: CombinatorRecipe) -> Self {
        Self::Combiner(value)
    }
}

impl From<SeparatorRecipe> for Recipe {
    fn from(value: SeparatorRecipe) -> Self {
        Self::Splitter(value)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProducerRecipe {
    pub ticks: u64,
    pub output: ItemType,
}

impl ProducerRecipe {
    pub fn into_port(&self) -> IoPort {
        self.output.into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TransformerRecipe {
    pub ticks: u64,
    pub input: ItemType,
    pub output: ItemType,
}

pub struct TransformerPorts {
    pub input: IoPort,
    pub output: IoPort,
}

impl TransformerRecipe {
    pub fn into_ports(&self) -> TransformerPorts {
        TransformerPorts { input: self.input.into(), output: self.output.into() }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CombinatorRecipe {
    pub ticks: u64,
    pub inputs: (ItemType, ItemType),
    pub output: ItemType,
}

pub struct CombinerPorts {
    pub inputs: (IoPort, IoPort),
    pub output: IoPort,
}

impl CombinatorRecipe {
    pub fn into_ports(&self) -> CombinerPorts {
        CombinerPorts { inputs: (self.inputs.0.into(), self.inputs.1.into()), output: self.output.into() }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SeparatorRecipe {
    pub ticks: u64,
    pub input: ItemType,
    pub outputs: (ItemType, ItemType),
}

pub struct SplitterPorts {
    pub input: IoPort,
    pub outputs: (IoPort, IoPort),
}

impl SeparatorRecipe {
    pub fn into_ports(&self) -> SplitterPorts {
        SplitterPorts { input: self.input.into(), outputs: (self.outputs.0.into(), self.outputs.1.into()) }
    }
}

pub struct ProducerRecipes {
    recipes: Vec<ProducerRecipe>,
}

pub struct TransformerRecipes {
    recipes: Vec<TransformerRecipe>,
}

pub struct CombinerRecipes {
    recipes: Vec<CombinatorRecipe>,
}

pub struct SplitterRecipes {
    recipes: Vec<SeparatorRecipe>,
}

impl ProducerRecipes {
    pub fn init() -> Self {
        Self { recipes: vec![
            ProducerRecipe { ticks: 10, output: ItemType::Input },
            ProducerRecipe { ticks: 10, output: ItemType::Output },
        ] }
    }

    pub fn get(&self, output: ItemType) -> Option<ProducerRecipe> {
        self.recipes.iter().find_map(|e| { if e.output == output { Some(*e) } else { None }})
    }
}

impl TransformerRecipes {
    pub fn init() -> Self {
        Self { recipes: vec![
            TransformerRecipe { ticks: 20, input: ItemType::Input, output: ItemType::Storage },
            TransformerRecipe { ticks: 20, input: ItemType::Output, output: ItemType::Producer },
        ] }
    }

    pub fn get(&self, output: ItemType) -> Option<TransformerRecipe> {
        self.recipes.iter().find_map(|e| { if e.output == output { Some(*e) } else { None }})
    }
}

impl CombinerRecipes {
    pub fn init() -> Self {
        Self { recipes: vec![
            CombinatorRecipe { ticks: 60, inputs: (ItemType::Input, ItemType::Output), output: ItemType::Transformer },
            CombinatorRecipe { ticks: 60, inputs: (ItemType::Transformer, ItemType::Input), output: ItemType::Combinator },
            CombinatorRecipe { ticks: 60, inputs: (ItemType::Transformer, ItemType::Output), output: ItemType::Separator },
        ] }
    }

    pub fn get(&self, output: ItemType) -> Option<CombinatorRecipe> {
        self.recipes.iter().find_map(|e| { if e.output == output { Some(*e) } else { None }})
    }
}