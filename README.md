# About

slurm_inspector V0.1 (2015.12.20), written by Willi Kappler

Licensed under the MIT License

This tool monitors the SLURM job management system (SLURM = Simple Linux Utility for Resource Management)
(see [https://computing.llnl.gov/linux/slurm/slurm.html](https://computing.llnl.gov/linux/slurm/slurm.html))
The result is presented on a simple web page (using default values for ex. http://localhost:4545).
slurm_inspector is completely written in [Rust](https://www.rust-lang.org/) using the [Iron](http://ironframework.io/) web framework.

# Install

1. Obtain source code from github
2. Compile and run it with
    cargo run --release

# Usage

slurm_inspector offers the following command line options:

    -p --port=[PORT] Sets the port for the web GUI (default: 4545)
    
    -i --interval=[INTERVAL] Sets the update interval (in sec.) for the web page (default: 60 sec.)
    
    --test create test values, does not call sinfo or squeue
    
    --loglevel=[LOGLEVEL] specify log level: error, info or debug

For example:

    cargo run --release -- -p 1234 -i 120
    
will run slurm_inspector listening on port 1234 and refreshing the SLURM status every 120 seconds.
Start your web browser and go to http://localhost:1234 (or http://myserver.com:1234)

The file "slurm_inspector.conf" contains an example ubuntu service configuration (and some comments on how to install it)

# TODO
- Use https instead of http
- Use login mechanism (user name and password)
- Use HTML template mechanism (HandleBars, Roustache, ...)
- Show more SLURM information values (accounting, etc.)
- Add some fancy charts and shiny CSS
- Maybe some interaction with JavaScript
- Clean up code (better error handling, ...)
- More test cases
- Better documentation
- Maybe use libslurm instead of parsing output of "sinfo" and "squeue"

Any feedback is wellcome! If you have any ideas or an important feature is missing or if you found a bug let me know.
