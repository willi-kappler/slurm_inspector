// slurm_inspector V0.1 (2015.12.20), written by Willi Kappler
//
// Licensed under the MIT License

// External crates:
extern crate iron;
#[macro_use] extern crate log;
extern crate flexi_logger;

// Internal crates:
extern crate slurm_util;

// System modules:
use std::net::{SocketAddrV4, Ipv4Addr};
use std::sync::{Arc, Mutex};

// External modules:
use iron::prelude::{Iron, Request};
use flexi_logger::{detailed_format,init,LogConfig};

// Internal modules:
use slurm_util::configuration::setup_configuration;
use slurm_util::request_handler::handle_request;
use slurm_util::slurm_status::{SlurmStatus, check_slurm_status};

fn main() {
    // Parse command line arguments
    let config = setup_configuration();

    init(LogConfig { log_to_file: true, format: detailed_format, .. LogConfig::new() }, Some(config.log_level.clone()))
         .unwrap_or_else(|e| { panic!("Logger initialization failed with the following error: {}", e) });

    info!("configuration: port: {}, interval: {}, test mode: {}, log level: {}", config.port, config.interval, config.test_mode, config.log_level);

    // Create empty SlurmStatus object
    let initial_slurm_status = SlurmStatus{ node_info: Vec::new(), job_info: Vec::new(), last_update: String::new() };

    // Iron-persistence can't be used here since own thread can't access private filed "data" of struct "State"
    let local_slurm_status = Arc::new(Mutex::new(initial_slurm_status));

    // Start the background thread and read in real values
    check_slurm_status(&local_slurm_status, config.clone());

    // Need to clone this since each client request will be handled by iron in a separate thread
    let shared_slurm_status = local_slurm_status.clone();

    // Run iron web framework and wait for the user to load the page
    Iron::new( move |req: &mut Request| { handle_request(req, &shared_slurm_status) }
).http(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.port)).unwrap();
}
