#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(fn my_array: f32, implicit_select_all: f32, f32)] //~ERROR 'gen_array' tried to declare a method 'my_array' with a 'implicit_select_all' clause, but it contained a duplicated return type 'f32'
struct Implicit {
    pub value: f32,
}
