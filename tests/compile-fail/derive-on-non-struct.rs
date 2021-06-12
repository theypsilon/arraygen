#![no_main]

extern crate arraygen;

use arraygen::Arraygen;

#[derive(Arraygen)]
enum Empty1{} //~ERROR 8:1: 8:5: derive 'Arraygen' should only be used with braced structs

#[derive(Arraygen)]
union Empty2{a: i32} //~ERROR 11:1: 11:6: derive 'Arraygen' should only be used with braced structs