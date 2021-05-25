#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(pub fn my_array)] //~ERROR 'gen_array' tried to declare a method 'my_array', but the return type syntax was wrong
struct Empty{}