#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub my_array: i32)] //~ERROR 'gen_array' was used with the wrong syntax
struct Empty{}