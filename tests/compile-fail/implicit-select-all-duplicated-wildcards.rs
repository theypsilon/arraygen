#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
#[gen_array(fn my_array: f32, implicit_select_all: _, _)] //~ERROR 8:55: 8:56: gen_array method 'my_array' contains implicit_select_all clause with duplicated '_' type
struct Implicit {
    pub value: f32,
}
