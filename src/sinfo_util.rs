//! Abstraction for the SLURM sinfo command
//! Runs sinfo, parses output into data structure (PartitionNodeInfo)

// System modules:
use std::process::Command;

/// PartitionAvailability, can be "up" or "down"
#[derive(Debug, PartialEq)]
pub enum PartitionAvailability {
    /// SLURM Partition is up and available
    Up,
    /// SLURM partition is not available
    Down
}

/// The ErrorCause why a SLURM partition is down
#[derive(Debug, PartialEq)]
pub enum ErrorCause {
    Down,
    Drained,
    Draining,
    None,
    /// An unknown error occured
    Unknown
}

/// The state of an individual node
#[derive(Debug, PartialEq)]
pub enum NodeState {
    Allocated,
    Completing,
    Down,
    Drained,
    Draining,
    Fail,
    Failing,
    Idle,
    Maint,
    /// The node is in an unknown state
    Unknown
}

/// SLURM partition and node information
#[derive(Debug, PartialEq)]
pub struct PartitionNodeInfo {
    pub partition: String,
    pub availability: PartitionAvailability,
    pub hostname: String,
    pub node: String,
    pub error: ErrorCause,
    pub cpu_load: Option<f64>,
    pub node_state: NodeState,
    pub node_sockets: Option<u32>,
    pub node_cores: Option<u32>,
    pub node_threads: Option<u32>
}

/// Public helper function to generate test data
pub fn get_partition_node_info_test() -> Vec<PartitionNodeInfo> {
    let test_data = "
        esd up node01 node01 none 0.22 idle 2 2 2
        esd up node02 node02 none 0.0 idle 2 8 2
        esd down node03 node03 down - - - - -
        esd up node04 node04 none 0.71 idle 1 1 1
        esd up node05 node05 none 0.0 alloc 1 1 1
        esd up node06 node06 none 0.0 completing 1 1 1
        esd up node07 node07 none 0.0 drained 1 1 1
        esd up node08 node08 none 0.0 draining 1 1 1
        esd up node09 node09 none 0.0 fail 1 1 1
        esd up node10 node10 none 0.0 failing 1 1 1
        esd up node11 node11 none 0.0 maint 1 1 1
        esd up node12 node12 none 0.0 unknown 1 1 1
    ";

    get_pn_info_util(test_data)
}

/// Public helper function to retrieve a list of current SLURM partition and node status
pub fn get_partition_node_info() -> Vec<PartitionNodeInfo> {
    get_pn_info_util(&call_sinfo())
}

// Private helper function to parse the output of "sinfo" and return a list of PartitionNodeInfo
fn get_pn_info_util(sinfo_output: &str) -> Vec<PartitionNodeInfo> {
    let mut result: Vec<PartitionNodeInfo> = Vec::new();

    for line in sinfo_output.lines() {
        let items: Vec<&str> = line.split_whitespace().collect();

        // Skip invalid lines
        if items.len() != 10 {
            debug!("number of items in line: {}", items.len());
            continue;
        }

        result.push( PartitionNodeInfo{
            partition: items[0].to_string(),
            availability: str_to_availability(items[1]),
            hostname: items[2].to_string(),
            node: items[3].to_string(),
            error: str_to_error(items[4]),
            cpu_load: items[5].parse::<f64>().ok(),
            node_state: str_to_node_state(items[6]),
            node_sockets: items[7].parse::<u32>().ok(),
            node_cores: items[8].parse::<u32>().ok(),
            node_threads: items[9].parse::<u32>().ok()
        })
    }

    result
}

#[test]
fn test_get_pn_info_util_empty() {
    assert!(get_pn_info_util("").len() == 0);
}

#[test]
fn test_get_pn_info_util_invalid() {
    assert!(get_pn_info_util("1 2 3").len() == 0);
}

#[test]
fn test_get_pn_info_util_01() {
    let input = "longrun up node01.foo.bar node01 none 0.22 idle 2 2 2";
    let output = vec![PartitionNodeInfo{
        partition: "longrun".to_string(),
        availability: PartitionAvailability::Up,
        hostname: "node01.foo.bar".to_string(),
        node: "node01".to_string(),
        error: ErrorCause::None,
        cpu_load: Some(0.22),
        node_state: NodeState::Idle,
        node_sockets: Some(2),
        node_cores: Some(2),
        node_threads: Some(2)
    }];

    assert_eq!(get_pn_info_util(input), output);
}

#[test]
fn test_get_pn_info_util_02() {
    let input = "longrun up node01.foo.bar node01 none 0.22 idle 2 2 2\nlongrun up node02.foo.bar node02 down 0.1 idle 1 2 4";
    let output = vec![
        PartitionNodeInfo{
            partition: "longrun".to_string(),
            availability: PartitionAvailability::Up,
            hostname: "node01.foo.bar".to_string(),
            node: "node01".to_string(),
            error: ErrorCause::None,
            cpu_load: Some(0.22),
            node_state: NodeState::Idle,
            node_sockets: Some(2),
            node_cores: Some(2),
            node_threads: Some(2)
        },
        PartitionNodeInfo{
            partition: "longrun".to_string(),
            availability: PartitionAvailability::Up,
            hostname: "node02.foo.bar".to_string(),
            node: "node02".to_string(),
            error: ErrorCause::Down,
            cpu_load: Some(0.1),
            node_state: NodeState::Idle,
            node_sockets: Some(1),
            node_cores: Some(2),
            node_threads: Some(4)
        }
    ];

    assert_eq!(get_pn_info_util(input), output);
}

/*
    sinfo -o "%R %a %n %N %E %O %T %X %Y %Z" -h
    %R: partition name
    %a: availability of partition (up / down)
    %n: host name
    %N: node name
    %E: reason of error if any (down, drained, draining, none)
    %O: CPU load of node
    %T: sdtate of node
    %X: number of sockets per node
    %Y: number of cores per node
    %Z: number of threads per node

    example output:

    high_mem up node01.foo.bar node01 none 0.01 idle 2 2 2
    high_mem up node02.foo.bar node02 none 0.02 idle 2 2 2
    high_mem up node03.foo.bar node03 none 0.12 idle 2 2 2
    high_mem up node04.foo.bar node04 none 0.03 idle 2 6 2
    high_mem up node05.foo.bar node05 none 0.01 idle 2 4 2
    high_mem up node06.foo.bar node06 none 0.01 idle 2 4 2
    high_mem up node07.foo.bar node07 none 0.01 idle 2 4 2
    high_mem up node08.foo.bar node08 none 0.01 idle 2 8 2
    high_mem up node09.foo.bar node09 none 0.01 idle 2 8 2
    high_mem up node10.foo.bar node10 none 0.01 idle 2 8 2
    high_mem up node11.foo.bar node11 none 0.01 idle 2 2 2
*/

// Private helper function to execute the "sinfo" SLURM command and return its output as a string
// On error returns an empty string. TODO: better error handling
fn call_sinfo() -> String {
    let output = Command::new("sinfo")
        .arg("-h")
        .arg("-o")
        .arg("%R %a %n %N %E %O %T %X %Y %Z")
        .output();

    match output {
        Result::Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Result::Err(err) => {
            error!("Could not execute 'sinfo': {}", err);
            // return empty string on error, but continue with the program
            String::new()
        }
    }
}

// Private helper function to parse partition availability
fn str_to_availability(avail: &str) -> PartitionAvailability {
    match &*avail.to_lowercase() {
        "up" => PartitionAvailability::Up,
        _ => PartitionAvailability::Down
    }
}

#[test]
fn test_str_to_availability_up() {
    assert_eq!(str_to_availability("up"), PartitionAvailability::Up);
    assert_eq!(str_to_availability("Up"), PartitionAvailability::Up);
    assert_eq!(str_to_availability("UP"), PartitionAvailability::Up);
}

#[test]
fn test_str_to_availability_down() {
    assert_eq!(str_to_availability("down"), PartitionAvailability::Down);
    assert_eq!(str_to_availability("Down"), PartitionAvailability::Down);
    assert_eq!(str_to_availability("DOWN"), PartitionAvailability::Down);
}

// Private helper function to parse partition error cause
fn str_to_error(error: &str) -> ErrorCause {
    match &*error.to_lowercase() {
        "down" => ErrorCause::Down,
        "drained" => ErrorCause::Drained,
        "draining" => ErrorCause::Draining,
        "none" => ErrorCause::None,
        _ => ErrorCause::Unknown
    }
}

#[test]
fn test_str_to_error_down() {
    assert_eq!(str_to_error("down"), ErrorCause::Down);
    assert_eq!(str_to_error("Down"), ErrorCause::Down);
    assert_eq!(str_to_error("DOWN"), ErrorCause::Down);
}

#[test]
fn test_str_to_error_drained() {
    assert_eq!(str_to_error("drained"), ErrorCause::Drained);
    assert_eq!(str_to_error("Drained"), ErrorCause::Drained);
    assert_eq!(str_to_error("DRAINED"), ErrorCause::Drained);
}

#[test]
fn test_str_to_error_draining() {
    assert_eq!(str_to_error("draining"), ErrorCause::Draining);
    assert_eq!(str_to_error("Draining"), ErrorCause::Draining);
    assert_eq!(str_to_error("DRAINING"), ErrorCause::Draining);
}

#[test]
fn test_str_to_error_none() {
    assert_eq!(str_to_error("none"), ErrorCause::None);
    assert_eq!(str_to_error("None"), ErrorCause::None);
    assert_eq!(str_to_error("NONE"), ErrorCause::None);
}

// Private helper function to parse node state
fn str_to_node_state(n_state: &str) -> NodeState {
    match &*n_state.to_lowercase() {
            "alloc" | "allocated" => NodeState::Allocated,
            "completing" => NodeState::Completing,
            "down" => NodeState::Down,
            "drained" => NodeState::Drained,
            "draining" => NodeState::Draining,
            "fail" => NodeState::Fail,
            "failing" => NodeState::Failing,
            "idle" => NodeState::Idle,
            "maint" => NodeState::Maint,
            _ => NodeState::Unknown
    }
}

#[test]
fn test_str_to_node_state_alloc() {
    assert_eq!(str_to_node_state("alloc"), NodeState::Allocated);
    assert_eq!(str_to_node_state("Alloc"), NodeState::Allocated);
    assert_eq!(str_to_node_state("ALLOC"), NodeState::Allocated);
    assert_eq!(str_to_node_state("allocated"), NodeState::Allocated);
    assert_eq!(str_to_node_state("allocated"), NodeState::Allocated);
    assert_eq!(str_to_node_state("ALLOCATED"), NodeState::Allocated);
}

#[test]
fn test_str_to_node_state_completing() {
    assert_eq!(str_to_node_state("completing"), NodeState::Completing);
    assert_eq!(str_to_node_state("Completing"), NodeState::Completing);
    assert_eq!(str_to_node_state("COMPLETING"), NodeState::Completing);
}

#[test]
fn test_str_to_node_state_down() {
    assert_eq!(str_to_node_state("down"), NodeState::Down);
    assert_eq!(str_to_node_state("Down"), NodeState::Down);
    assert_eq!(str_to_node_state("DOWN"), NodeState::Down);
}

#[test]
fn test_str_to_node_state_drained() {
    assert_eq!(str_to_node_state("drained"), NodeState::Drained);
    assert_eq!(str_to_node_state("Drained"), NodeState::Drained);
    assert_eq!(str_to_node_state("DRAINED"), NodeState::Drained);
}

#[test]
fn test_str_to_node_state_draining() {
    assert_eq!(str_to_node_state("draining"), NodeState::Draining);
    assert_eq!(str_to_node_state("Draining"), NodeState::Draining);
    assert_eq!(str_to_node_state("DRAINING"), NodeState::Draining);
}

#[test]
fn test_str_to_node_state_fail() {
    assert_eq!(str_to_node_state("fail"), NodeState::Fail);
    assert_eq!(str_to_node_state("Fail"), NodeState::Fail);
    assert_eq!(str_to_node_state("FAIL"), NodeState::Fail);
}

#[test]
fn test_str_to_node_state_failing() {
    assert_eq!(str_to_node_state("failing"), NodeState::Failing);
    assert_eq!(str_to_node_state("Failing"), NodeState::Failing);
    assert_eq!(str_to_node_state("FAILING"), NodeState::Failing);
}

#[test]
fn test_str_to_node_state_idle() {
    assert_eq!(str_to_node_state("idle"), NodeState::Idle);
    assert_eq!(str_to_node_state("Idle"), NodeState::Idle);
    assert_eq!(str_to_node_state("IDLE"), NodeState::Idle);
}

#[test]
fn test_str_to_node_state_maint() {
    assert_eq!(str_to_node_state("maint"), NodeState::Maint);
    assert_eq!(str_to_node_state("Maint"), NodeState::Maint);
    assert_eq!(str_to_node_state("MAINT"), NodeState::Maint);
}

#[test]
fn test_str_to_node_state_unknown() {
    assert_eq!(str_to_node_state("unknown"), NodeState::Unknown);
    assert_eq!(str_to_node_state("Unknown"), NodeState::Unknown);
    assert_eq!(str_to_node_state("UNKNOWN"), NodeState::Unknown);
}
