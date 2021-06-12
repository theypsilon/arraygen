#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(fn my_array: f32, implicit_select_all: f32, f32)] //~ERROR 8:57: 8:60: gen_array method 'my_array' contains implicit_select_all clause with duplicated 'f32' type
struct Implicit {
    pub value: f32,
}
