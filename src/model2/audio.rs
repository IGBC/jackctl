use crate::model2::port::PortGroup;

#[derive(Default)]
pub struct AudioGroups {
    inputs: PortGroup,
    outputs: PortGroup,
}
