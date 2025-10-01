use crate::{pipeline::machine::ItemBuffer, ItemType};

pub mod recipe;
pub mod machine;

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub enum PortStatus {
//     Free,
//     Taken,
// }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct IoBuffer {
    pub buffer: ItemBuffer,
    pub item_type: ItemType,
}

impl IoBuffer {
    pub fn new(item_type: ItemType) -> Self {
        Self { buffer: ItemBuffer::new(), item_type }
    }

    pub fn with_capacity(item_type: ItemType, capacity: u64) -> Self {
        Self { buffer: ItemBuffer::with_capacity(capacity), item_type }
    }

    // pub fn is_free(&self) -> bool {
    //     self.status == PortStatus::Free
    // }
}

impl From<ItemType> for IoBuffer {
    fn from(value: ItemType) -> Self {
        Self::new(value)
    }
}

// #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
// pub struct PipelineId {
//     inner: usize,
// }

// impl PipelineId {
//     fn new() -> Self {
//             Self { inner: 0 }
//     }

//     fn inc(&mut self) -> Self {
//         let out = self.clone();
//         self.inner += 1;
//         out
//     }
// }

// #[derive(Debug)]
// pub struct Pipeline {
//     pub inner: HashMap<PipelineId, PipelineSegment>,
//     pub next_id: PipelineId,
// }

// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
// pub enum PipelineError {
//     InvalidId,
// }

// impl Pipeline {
//     pub fn new() -> Self {
//         Self { inner: HashMap::with_capacity(3), next_id: PipelineId::new() }
//     }

//     pub fn with_machine(machine: Machine) -> Self {
//         let mut next_id = PipelineId::new();
//         let mut inner = HashMap::with_capacity(3);
//         let id = next_id.inc();
//         inner.insert(id, PipelineSegment::new(id, machine));

//         Self { inner, next_id }
//     }

//     pub fn with_machines(machines: Vec<Machine>) -> Self {
//         let mut next_id = PipelineId::new();
//         let inner = HashMap::from_iter(machines.into_iter().map(|m| {
//             let id = next_id.inc();
//             (id, PipelineSegment::new(id, m))
//         }));

//         Self { inner, next_id }
//     }

//     pub fn with_capacity(capacity: usize) -> Self {
//         Self { inner: HashMap::with_capacity(capacity), next_id: PipelineId::new() }
//     }

//     pub fn push(&mut self, machine: Machine) -> PipelineId {
//         let id = self.next_id();
//         let segment = PipelineSegment::new(id, machine);
//         self.inner.insert(id, segment);
//         id
//     }

//     pub fn bind_output(&mut self, src_id: PipelineId, dest_id: PipelineId) -> Result<(), MachineBindError> {
//         let [src, dest] = self.inner.get_disjoint_mut([&src_id, &dest_id]);
//         let Some(input) = src else { Err(MachineBindError::InputDoesNotExist)? };
//         let Some(output) = dest else { Err(MachineBindError::OutputDoesNotExist)? };

//         for item_stack in input.inner.outputs() {
//             let output_data = input.inner.get_output(item_stack.item_type)?;
//             let input_data = output.inner.get_input(item_stack.item_type)?;

//             *output_data.id = Some(dest_id);
//             output_data.port.expect("Invalid output port").status = PortStatus::Taken;
//             input_data.port.expect("Invalid input port").status = PortStatus::Taken;
//         }

//         Ok(())
//     }

//     pub fn tick(&mut self) {
//         struct Output {
//             src: PipelineId,
//             dest: PipelineId,
//         }

//         let mut outputs: Vec<Output> = Vec::with_capacity(2);
//         for machine in self.inner.values_mut() {
//             let mut complete = CraftStatus::Incomplete;
//             for _ in 0..machine.inner.mult {
//                 complete |= machine.inner.tick();
//             }

//             if complete == CraftStatus::Complete {
//                 if let Some(output_id) = machine.inner.output_id {
//                     outputs.push(Output { src: machine.id, dest: output_id });
//                 }
//             }
//         }

//         for Output { src, dest} in outputs {
//             let [src, dest] = self.inner.get_disjoint_mut([&src, &dest]);

//             match (src, dest) {
//                 (Some(src), Some(dest)) => {
//                     for port in &mut src.inner.output_ports {
//                         if let Some(port) = port {
//                             let dest_port = dest.inner.input_ports.iter_mut().filter(|p| p.map(|p| p.item_type == port.item_type).unwrap_or(false)).next();
//                             if let Some(Some(dest_port)) = dest_port {
//                                 let change = dest_port.buffer.remaining().min(port.buffer.current);
//                                 dest_port.buffer.current += change;
//                                 port.buffer.current -= change;
//                             }
//                         }
//                     }
//                 },
//                 _ => panic!("They were here just a moment ago...")
//             }
//         }
//     }

//     pub fn set_mult(&mut self, machine: &PipelineId, mult: u64) -> Result<(), PipelineError> {
//         self.inner.get_mut(machine).ok_or(PipelineError::InvalidId)?.inner.mult = mult;

//         Ok(())
//     }

//     fn next_id(&mut self) -> PipelineId {
//         self.next_id.inc()
//     }
// }

// #[derive(Debug)]
// pub struct PipelineSegment {
//     id: PipelineId,
//     inner: Machine,
// }

// impl PipelineSegment {
//     pub fn new(id: PipelineId, inner: Machine) -> Self {
//         Self { id, inner: inner }
//     }
// }