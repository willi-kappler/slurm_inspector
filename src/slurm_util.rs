//! Utility function used by slurm_inspector
//! Just contains references to external and internal modules

#[macro_use] extern crate clap;
#[macro_use] extern crate log;
extern crate flexi_logger;
extern crate iron;
extern crate time;

pub mod sinfo_util;
pub mod squeue_util;
pub mod configuration;
pub mod request_handler;
pub mod slurm_status;
