// #[macro_use]
extern crate neon;
use neon::prelude::*;

extern crate audrey;
extern crate hound;
extern crate rnnoise_rust;

mod denoise;
mod rnnoise;

register_module!(mut cx, {
    cx.export_function("suppress", denoise::suppress)
});
