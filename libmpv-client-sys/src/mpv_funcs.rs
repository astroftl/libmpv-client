#![cfg(not(feature = "dyn-sym"))]

use crate::*;

include!(concat!(env!("OUT_DIR"), "/func_bindings.rs"));