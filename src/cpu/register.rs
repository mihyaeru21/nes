#[derive(Debug)]
pub struct Registers {
    accumulator: u8,
    index_x: u8,
    index_y: u8,
    stack_pointer: u16,
    status: Status,
    program_counter: u16,
}

#[derive(Debug)]
pub struct Status {
    pub negative: bool,
    pub overflow: bool,
    pub reserved: bool,
    pub break_mode: bool,
    pub decimal_mode: bool,
    pub irq_prohibited: bool,
    pub zero: bool,
    pub carry: bool,
}
