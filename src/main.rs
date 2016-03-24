#[macro_use(value_t)]
extern crate clap;

extern crate opus;
extern crate serde;
extern crate serde_json;
extern crate byteorder;


include!(concat!(env!("OUT_DIR"), "/main.rs"));
