use core::num;
use std::ops::Add;
use std::ptr::addr_eq;
use std::{any::type_name, io};

use std::collections::HashMap;

mod addresser;
mod logging;
mod data_bus;
mod memory;
mod cpu6502;
mod helper_functions;

use crate::addresser::{AddressingModes, ADDRESSER};
use crate::logging::{log_error, log_warning, log_info};




fn main() {
    println!("Hello, world!");
}
