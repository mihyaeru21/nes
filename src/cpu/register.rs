#[derive(Debug, Default)]
pub struct Registers {
    pub accumulator: u8,
    pub index_x: u8,
    pub index_y: u8,
    pub stack_pointer: u16,
    pub status: Status,
    pub program_counter: u16,
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

impl Default for Status {
    fn default() -> Self {
        Self {
            negative: false,
            overflow: false,
            reserved: true, // 常にセットされている
            break_mode: false,
            decimal_mode: false,
            irq_prohibited: false,
            zero: false,
            carry: false,
        }
    }
}
