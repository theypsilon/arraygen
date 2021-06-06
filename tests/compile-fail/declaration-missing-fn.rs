#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub my_array: i32)] //~ERROR 8:17: 8:25: expected `fn`
struct Empty{}