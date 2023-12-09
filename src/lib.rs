pub use binder::*;
pub use class::*;
pub use parcel::*;
#[cfg(feature = "service_manager")]
pub use service_manager::*;
pub use status::*;

mod sys {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/binder_sys.rs"));
}

mod binder;
mod class;
mod parcel;
#[cfg(feature = "service_manager")]
mod service_manager;
mod status;
#[cfg(test)]
mod test;
