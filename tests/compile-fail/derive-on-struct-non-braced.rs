#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
struct Empty1; //~ERROR 8:14: 8:15: derive 'Arraygen' should only be used with braced structs

#[derive(Arraygen)]
struct Empty2(i32, f32); //~ERROR 11:14: 11:24: derive 'Arraygen' should only be used with braced structs
