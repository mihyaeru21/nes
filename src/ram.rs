use std::{cell::RefCell, rc::Rc};

pub type Ram = Rc<RefCell<Vec<u8>>>;
