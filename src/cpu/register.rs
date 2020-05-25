#[derive(Debug, Default)]
pub struct Registers {
    pub accumulator: u8,      // A
    pub index_x: u8,          // X
    pub index_y: u8,          // Y
    pub stack_pointer: u16,   // S
    pub status: Status,       // P
    pub program_counter: u16, // PC
}

#[derive(Debug)]
pub struct Status {
    pub negative: bool,       // N
    pub overflow: bool,       // V
    pub reserved: bool,       // R
    pub break_mode: bool,     // B
    pub decimal_mode: bool,   // D
    pub irq_prohibited: bool, // I
    pub zero: bool,           // Z
    pub carry: bool,          // C
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
