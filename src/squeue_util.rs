//! Abstraction for the SLURM squeue command
//! Runs squeue, parses output into data structure (JobInfo)

// System modules:
use std::process::Command;

/// State reason, why is the job in the current state ?
#[derive(Debug)]
pub enum StateReason {
        Dependency,
        None,
        PartitionDown,
        PartitionNodeLimit,
        PartitionTimeLimit,
        Priority,
        Resources,
        NodeDown,
        BadConstraints,
        SystemFailure,
        JobLaunchFailure,
        NonZeroExitCode,
        TimeLimit,
        InactiveLimit,
        Unknown
}

/// In which state is the current job in ?
#[derive(Debug)]
pub enum JobState {
        Cancelled,
        Completed,
        Configuring,
        Completing,
        Failed,
        NodeFail,
        Pending,
        Preempted,
        Running,
        Suspended,
        Timeout,
        Unknown
}

/// All the information about a SLURM job
#[derive(Debug)]
pub struct JobInfo {
    pub executing_host: String,
    pub minimum_cpu: Option<u32>,
    pub num_cpu: Option<u32>,
    pub num_nodes: Option<u32>,
    pub job_array_id: Option<u32>,
    pub num_sockets: Option<u32>,
    pub job_id: Option<u32>,
    pub num_cores: Option<u32>,
    pub job_name: String,
    pub num_threads: Option<u32>,
    pub job_array_index: Option<u32>,
    pub run_time: String,
    pub list_of_nodes: Vec<String>,
    pub priority: Option<f64>,
    pub state_reason: StateReason,
    pub start_time: String,
    pub job_state: JobState,
    pub user_name: String,
    pub user_id: Option<u32>
}

/// Public helper function to generate test data
pub fn get_job_info_test() -> Vec<JobInfo> {
    let test_data = "
        node01 1 2 1 N/A * 1 * small_test01 * N/A 1:00 node01 0.9 None 2000-01-01T09:00:00 RUNNING user01 1000
        node01 1 2 2 N/A * 2 * small_test02 * N/A 1:15 node01,node02 0.9 None 2000-01-01T09:00:00 cancelled user02 1001
        node01 1 2 4 N/A * 3 * small_test03 * N/A 2:00 node01 0.1 None 2000-01-01T09:00:00 completed user03 1002
        node02 1 2 1 N/A * 4 * small_test04 * N/A 2:00 node01 0.2 None 2000-01-01T09:00:00 configuring user04 1003
        node03 1 2 1 N/A * 5 * small_test05 * N/A 2:46 node01 0.9 None 2000-01-01T09:00:00 Completing user05 1004
        node04 1 2 6 N/A * 6 * small_test06 * N/A 3:12 node03,node04,node05 0.9 None 2000-01-01T09:00:00 FAILED user05 1004
        node05 1 2 1 N/A * 7 * small_test07 * N/A 4:02 node01 0.9 None 2000-01-01T09:00:00 nodefail user01 1000
        node06 1 2 1 N/A * 8 * small_test08 * N/A 5:00 node01 0.9 None 2000-01-01T09:00:00 Pending user02 1001
        node07 1 2 2 N/A * 9 * small_test09 * N/A 1:00 node01 0.5 None 2000-01-01T09:00:00 preempted user02 1001
        node08 1 2 2 N/A * 10 * small_test10 * N/A 2:01 node01 0.6 None 2000-01-01T09:00:00 suspended user03 1002
        node08 1 2 10 N/A * 11 * small_test11 * N/A 2:06 node01 0.9 None 2000-01-01T09:00:00 timeout user04 1003
        node08 1 2 6 N/A * 12 * small_test12 * N/A 4:09 node01 0.2 None 2000-01-01T09:00:00 UNKNOWN user05 1004
    ";

    get_job_info_util(test_data)
}

/// Public helper function to retrieve the current list of jobs and their states
pub fn get_job_info() -> Vec<JobInfo> {
    get_job_info_util(&call_squeue())
}

// Private helper function to parse the output of "squeue" and return a list of JobInfo
fn get_job_info_util(squeue_output: &str) -> Vec<JobInfo> {
    let mut result: Vec<JobInfo> = Vec::new();

    for line in squeue_output.lines() {
        let items: Vec<&str> = line.split_whitespace().collect();

        // Skip invalid line
        if items.len() != 19 {
            debug!("number of items in line: {}", items.len());
            continue
        }

        result.push( JobInfo{
                executing_host: items[0].to_string(),
                minimum_cpu: items[1].parse::<u32>().ok(),
                num_cpu: items[2].parse::<u32>().ok(),
                num_nodes: items[3].parse::<u32>().ok(),
                job_array_id: items[4].parse::<u32>().ok(),
                num_sockets: items[5].parse::<u32>().ok(),
                job_id: items[6].parse::<u32>().ok(),
                num_cores: items[7].parse::<u32>().ok(),
                job_name: items[8].to_string(),
                num_threads: items[9].parse::<u32>().ok(),
                job_array_index: items[10].parse::<u32>().ok(),
                run_time: items[11].to_string(),
                list_of_nodes: str_to_list_of_nodes(items[12]),
                priority: items[13].parse::<f64>().ok(),
                state_reason: str_to_state_reason(items[14]),
                start_time: items[15].to_string(),
                job_state: str_to_job_state(items[16]),
                user_name: items[17].to_string(),
                user_id: items[18].parse::<u32>().ok()
        })
    }

    result
}

#[test]
fn test_get_job_info_util_empty() {
    assert_eq!(get_job_info_util("").len(), 0);
}

#[test]
fn test_get_job_info_util_invalid() {
    assert_eq!(get_job_info_util("1 2 3 4").len(), 0);
}

/*
    squeue -h -o "%B %c %C %D %F %H %i %I %j %J %K %M %N %p %r %S %T %u %U"
    %B: Executing host
    %c: Minimum number of CPUs
    %C: Number of CPUs
    %D: Number of nodes allocated
    %F: Job array's job ID
    %H: Number of sockets
    %i: Job id / job step id
    %I: Number of cores
    %j: Job name / job step name
    %J: Number of threads
    %K: Job array index
    %M: Time used by the job
    %N: List of nodes allocated
    %p: Priority
    %r: Reason of job state
    %S: Actual or expected start time
    %T: Job state
    %u: User name
    %U: User ID

    Output looks like:
    agassiz 1 2 1 N/A * 82 * small_test * N/A 2:46 agassiz 0.99998474074527 None 2015-11-12T09:51:32 RUNNING willi 1000
    82 agassiz 1 2 1 N/A * 82 * small_test * N/A 2:46 agassiz 0.99998474074527 None 2015-11-12T09:51:32 RUNNING willi 1000
*/

// Private helper function to execute the external "squeue" SLURM command and return its output into a string
// On error returns an empty string. TODO: better error handling
fn call_squeue() -> String {
    let output = Command::new("squeue")
        .arg("-h")
        .arg("-o")
        .arg("%B %c %C %D %F %H %i %I %j %J %K %M %N %p %r %S %T %u %U")
        .output();

    match output {
        Result::Ok(val) => String::from_utf8_lossy(&val.stdout).to_string(),
        Result::Err(err) => {
            error!("Could not execute 'squeue': {}", err);
            // return empty string on error, but continue with the program
            String::new()
        }
    }
}

// Private helper function to parse the list of node the job is runnin on
fn str_to_list_of_nodes(nodes: &str) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();

    if nodes.len() == 0 {
        return Vec::new();
    }

    for node_name in nodes.split(',') {
        result.push(node_name.to_string());
    }

    result
}

#[test]
fn test_str_to_list_of_nodes_empty() {
    assert_eq!(str_to_list_of_nodes("").len(), 0);
}

#[test]
fn test_str_to_list_of_nodes_01() {
    assert_eq!(str_to_list_of_nodes("node1"), vec!["node1"]);
}

#[test]
fn test_str_to_list_of_nodes_02() {
    assert_eq!(str_to_list_of_nodes("node1,node2"), vec!["node1", "node2"]);
}

#[test]
fn test_str_to_list_of_nodes_03() {
    assert_eq!(str_to_list_of_nodes("node1,node2,node3"), vec!["node1", "node2", "node3"]);
}

// Private helper function to parse the job state reason
fn str_to_state_reason(reason: &str) -> StateReason {
    match &*reason.to_lowercase() {
        "dependency" => StateReason::Dependency,
        "none" => StateReason::None,
        "partitiondown" => StateReason::PartitionDown,
        "partitionnodelimit" => StateReason::PartitionNodeLimit,
        "partitiontimelimit" => StateReason::PartitionTimeLimit,
        "priority" => StateReason::Priority,
        "resources" => StateReason::Resources,
        "nodedown" => StateReason::NodeDown,
        "badconstraints" => StateReason::BadConstraints,
        "systemfailure" => StateReason::SystemFailure,
        "joblaunchfailure" => StateReason::JobLaunchFailure,
        "nonzeroexitcode" => StateReason::NonZeroExitCode,
        "timelimit" => StateReason::TimeLimit,
        "inactivelimit" => StateReason::InactiveLimit,
        _ => StateReason::Unknown
    }
}

#[test]
fn test_str_to_state_reason_dependency() {
    assert_eq!(str_to_state_reason("dependency"), StateReason::Dependency);
    assert_eq!(str_to_state_reason("Dependency"), StateReason::Dependency);
    assert_eq!(str_to_state_reason("DEPENDENCY"), StateReason::Dependency);
}

#[test]
fn test_str_to_state_reason_none() {
    assert_eq!(str_to_state_reason("none"), StateReason::None);
    assert_eq!(str_to_state_reason("None"), StateReason::None);
    assert_eq!(str_to_state_reason("NONE"), StateReason::None);
}

#[test]
fn test_str_to_state_reason_partition_down() {
    assert_eq!(str_to_state_reason("partitiondown"), StateReason::PartitionDown);
    assert_eq!(str_to_state_reason("PartitionDown"), StateReason::PartitionDown);
    assert_eq!(str_to_state_reason("PARTITIONDOWN"), StateReason::PartitionDown);
}

#[test]
fn test_str_to_state_reason_partition_node_limit() {
    assert_eq!(str_to_state_reason("partitionnodelimit"), StateReason::PartitionNodeLimit);
    assert_eq!(str_to_state_reason("PartitionNodeLimit"), StateReason::PartitionNodeLimit);
    assert_eq!(str_to_state_reason("PARTITIONNODELIMIT"), StateReason::PartitionNodeLimit);
}

#[test]
fn test_str_to_state_reason_partition_time_limit() {
    assert_eq!(str_to_state_reason("partitiontimelimit"), StateReason::PartitionTimeLimit);
    assert_eq!(str_to_state_reason("PartitionTimeLimit"), StateReason::PartitionTimeLimit);
    assert_eq!(str_to_state_reason("PARTITIONTIMELIMIT"), StateReason::PartitionTimeLimit);
}

#[test]
fn test_str_to_state_reason_priority() {
    assert_eq!(str_to_state_reason("priority"), StateReason::Priority);
    assert_eq!(str_to_state_reason("Priority"), StateReason::Priority);
    assert_eq!(str_to_state_reason("PRIORITY"), StateReason::Priority);
}

#[test]
fn test_str_to_state_reason_resources() {
    assert_eq!(str_to_state_reason("resources"), StateReason::Resources);
    assert_eq!(str_to_state_reason("Resources"), StateReason::Resources);
    assert_eq!(str_to_state_reason("RESOURCES"), StateReason::Resources);
}

#[test]
fn test_str_to_state_reason_node_down() {
    assert_eq!(str_to_state_reason("nodedown"), StateReason::NodeDown);
    assert_eq!(str_to_state_reason("NodeDown"), StateReason::NodeDown);
    assert_eq!(str_to_state_reason("NODEDOWN"), StateReason::NodeDown);
}

#[test]
fn test_str_to_state_reason_bad_constrains() {
    assert_eq!(str_to_state_reason("badconstraints"), StateReason::BadConstraints);
    assert_eq!(str_to_state_reason("BadConstraints"), StateReason::BadConstraints);
    assert_eq!(str_to_state_reason("BADCONSTRAINTS"), StateReason::BadConstraints);
}

#[test]
fn test_str_to_state_reason_system_failure() {
    assert_eq!(str_to_state_reason("systemfailure"), StateReason::SystemFailure);
    assert_eq!(str_to_state_reason("SystemFailure"), StateReason::SystemFailure);
    assert_eq!(str_to_state_reason("SYSTEMFAILURE"), StateReason::SystemFailure);
}

#[test]
fn test_str_to_state_reason_job_launch_failure() {
    assert_eq!(str_to_state_reason("joblaunchfailure"), StateReason::JobLaunchFailure);
    assert_eq!(str_to_state_reason("JobLaunchFailure"), StateReason::JobLaunchFailure);
    assert_eq!(str_to_state_reason("JOBLAUNCHFAILURE"), StateReason::JobLaunchFailure);
}

#[test]
fn test_str_to_state_reason_non_zero_exit_code() {
    assert_eq!(str_to_state_reason("nonzeroexitcode"), StateReason::NonZeroExitCode);
    assert_eq!(str_to_state_reason("NonZeroExitcode"), StateReason::NonZeroExitCode);
    assert_eq!(str_to_state_reason("NONZEROEXITCODE"), StateReason::NonZeroExitCode);
}

#[test]
fn test_str_to_state_reason_time_limit() {
    assert_eq!(str_to_state_reason("timelimit"), StateReason::TimeLimit);
    assert_eq!(str_to_state_reason("TimeLimit"), StateReason::TimeLimit);
    assert_eq!(str_to_state_reason("TIMELIMIT"), StateReason::TimeLimit);
}

#[test]
fn test_str_to_state_reason_inactive_limit() {
    assert_eq!(str_to_state_reason("inactivelimit"), StateReason::InactiveLimit);
    assert_eq!(str_to_state_reason("Inactivelimit"), StateReason::InactiveLimit);
    assert_eq!(str_to_state_reason("INACTIVELIMIT"), StateReason::InactiveLimit);
}

#[test]
fn test_str_to_state_reason_unknown() {
    assert_eq!(str_to_state_reason("unknown"), StateReason::Unknown);
    assert_eq!(str_to_state_reason("Unknown"), StateReason::Unknown);
    assert_eq!(str_to_state_reason("UNKNOWN"), StateReason::Unknown);
}

// Private helper function to parse the job state
fn str_to_job_state(state: &str) -> JobState {
    match &*state.to_lowercase() {
        "cancelled" => JobState::Cancelled,
        "completed" => JobState::Completed,
        "configuring" => JobState::Configuring,
        "completing" => JobState::Completing,
        "failed" => JobState::Failed,
        "node_fail" => JobState::NodeFail,
        "pending" => JobState::Pending,
        "preempted" => JobState::Preempted,
        "running" => JobState::Running,
        "suspended" => JobState::Suspended,
        "timeout" => JobState::Timeout,
        _ => JobState::Unknown
    }
}



#[test]
fn test_str_to_job_state_cancelled() {
    assert_eq!(str_to_job_state("cancelled"), JobState::Cancelled);
    assert_eq!(str_to_job_state("Cancelled"), JobState::Cancelled);
    assert_eq!(str_to_job_state("CANCELLED"), JobState::Cancelled);
}

#[test]
fn test_str_to_job_state_completed() {
    assert_eq!(str_to_job_state("completed"), JobState::Completed);
    assert_eq!(str_to_job_state("Completed"), JobState::Completed);
    assert_eq!(str_to_job_state("COMPLETED"), JobState::Completed);
}

#[test]
fn test_str_to_job_state_configuring() {
    assert_eq!(str_to_job_state("configuring"), JobState::Configuring);
    assert_eq!(str_to_job_state("Configuring"), JobState::Configuring);
    assert_eq!(str_to_job_state("CONFIGURING"), JobState::Configuring);
}

#[test]
fn test_str_to_job_state_completing() {
    assert_eq!(str_to_job_state("completing"), JobState::Completing);
    assert_eq!(str_to_job_state("Completing"), JobState::Completing);
    assert_eq!(str_to_job_state("COMPLETING"), JobState::Completing);
}

#[test]
fn test_str_to_job_state_failed() {
    assert_eq!(str_to_job_state("failed"), JobState::Failed);
    assert_eq!(str_to_job_state("Failed"), JobState::Failed);
    assert_eq!(str_to_job_state("FAILED"), JobState::Failed);
}

#[test]
fn test_str_to_job_state_node_fail() {
    assert_eq!(str_to_job_state("node_fail"), JobState::NodeFail);
    assert_eq!(str_to_job_state("Node_Fail"), JobState::NodeFail);
    assert_eq!(str_to_job_state("NODE_FAIL"), JobState::NodeFail);
}

#[test]
fn test_str_to_job_state_pending() {
    assert_eq!(str_to_job_state("pending"), JobState::Pending);
    assert_eq!(str_to_job_state("Pending"), JobState::Pending);
    assert_eq!(str_to_job_state("PENDING"), JobState::Pending);
}

#[test]
fn test_str_to_job_state_preempted() {
    assert_eq!(str_to_job_state("preempted"), JobState::Preempted);
    assert_eq!(str_to_job_state("Preempted"), JobState::Preempted);
    assert_eq!(str_to_job_state("PREEMPTED"), JobState::Preempted);
}

#[test]
fn test_str_to_job_state_running() {
    assert_eq!(str_to_job_state("running"), JobState::Running);
    assert_eq!(str_to_job_state("Running"), JobState::Running);
    assert_eq!(str_to_job_state("RUNNING"), JobState::Running);
}

#[test]
fn test_str_to_job_state_suspended() {
    assert_eq!(str_to_job_state("suspended"), JobState::Suspended);
    assert_eq!(str_to_job_state("Suspended"), JobState::Suspended);
    assert_eq!(str_to_job_state("SUSPENDED"), JobState::Suspended);
}

#[test]
fn test_str_to_job_state_timeout() {
    assert_eq!(str_to_job_state("timeout"), JobState::Timeout);
    assert_eq!(str_to_job_state("Timeout"), JobState::Timeout);
    assert_eq!(str_to_job_state("TIMEOUT"), JobState::Timeout);
}

#[test]
fn test_str_to_job_state_unknown() {
    assert_eq!(str_to_job_state("unknown"), JobState::Unknown);
    assert_eq!(str_to_job_state("Unknown"), JobState::Unknown);
    assert_eq!(str_to_job_state("UNKNOWN"), JobState::Unknown);
}
