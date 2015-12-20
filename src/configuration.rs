//! Handles the configuration for slurm_inspector
//! Parses command line arguments via clap and sets default values

// External modules:
use clap::App;


/// slurm_inspector configuration (from command line arguments)
#[derive(Debug, Clone, PartialEq)]
pub struct Configuration {
    /// Iron webframework port, default: 4545
    pub port: u16,
    /// Update time intervall for SLURM status thread in seconds, default: 60 sec.
    pub interval: u64,
    /// If test mode is enabled, create some test data and do not call external commands ("sinfo", "squeue")
    pub test_mode: bool,
    /// Set the log level for flexi_logger: error, info or debug
    pub log_level: String
}

/// This will parse the command line arguments and create a new configuration object
/// If the arguments are missing or there is a parse error, then the default values are used
pub fn setup_configuration() -> Configuration {
    let matches = App::new("slurm_inspector")
        .version("0.1")
        .author("Willi Kappler")
        .about("Web GUI for the slurm job management system")
        .args_from_usage(
            "-p --port=[PORT] 'Sets the port for the web GUI (default: 4545)'
             -i --interval=[INTERVAL] 'Sets the update interval (in sec.) for the web page (default: 60 sec.)'
             --test 'create test values, does not call sinfo or squeue'
             --loglevel=[LOGLEVEL] 'specify log level: error, info or debug'"
        )
        .get_matches();

        let port = value_t!(matches.value_of("PORT"), u16).unwrap_or(4545);
        let interval = value_t!(matches.value_of("INTERVAL"), u64).unwrap_or(60);
        let test_mode = matches.is_present("test");
        let log_level = match matches.value_of("LOGLEVEL") {
            Some("error") => "error",
            Some("info") => "info",
            Some("debug") => "debug",
            _ => "info"
        };

        Configuration {
            port: port,
            interval: interval,
            test_mode: test_mode,
            log_level: log_level.to_string()
        }
}

#[test]
fn test_setup_configuration() {
    assert_eq!(setup_configuration(), Configuration{ port: 4545, interval: 60, test_mode: false, log_level: "info".to_string() });
}
