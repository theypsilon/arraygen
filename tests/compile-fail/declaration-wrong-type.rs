#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array)] //~ERROR 8:12: 8:29: expected `:`
struct Empty{}