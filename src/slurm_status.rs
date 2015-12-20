//! SLURM status data structure, updating and converting to HTML
//! Contains partition, node and job information and also time of last update
//! Runs a background thread in an endless loop to periodically check SLURM status
//! and updates data structure accordingly

// System modules:
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;

// External modules:
use time::{strftime, now};

// Internal modules:
use sinfo_util::{PartitionNodeInfo, PartitionAvailability, get_partition_node_info, get_partition_node_info_test};
use squeue_util::{JobInfo, get_job_info, get_job_info_test};
use configuration::Configuration;

/// SlurmStatus type. This is shared accross the Iron response threads and the SLURM status thread
pub struct SlurmStatus {
    /// List of partition and node information
    pub node_info: Vec<PartitionNodeInfo>,
    /// List of job information
    pub job_info: Vec<JobInfo>,
    /// The last time the above two lists have been updated
    /// Format: %Y.%m.%d - %H:%M
    pub last_update: String
}

/// Public function that starts the SLURM status thread and update the SlurmStatus object accordingly every time interval
/// TODO: better error handling
pub fn check_slurm_status(local_slurm_status: &Arc<Mutex<SlurmStatus>>, config: Configuration) {
    let shared_slurm_status = local_slurm_status.clone();

    thread::spawn(move || {
        // Endless loop, just keep checking the status of SLURM
        loop {
            match shared_slurm_status.lock() {
                Ok(mut status) => {
                    debug!("Update slurm status");
                    if config.test_mode {
                        status.node_info = get_partition_node_info_test();
                        status.job_info = get_job_info_test();
                    } else {
                        status.node_info = get_partition_node_info();
                        status.job_info = get_job_info();
                    }
                    status.last_update = strftime("%Y.%m.%d - %H:%M", &now()).unwrap();
                },
                Err(err) => {
                    error!("Could not lock Mutex: {}", err);
                }
            }

            sleep(Duration::new(config.interval, 0));
        }
    });
}

/// Public helper function accepts SlurmStatus and returns a string containing the HTML representation of the status
/// TODO: use some template mechanism (HandleBars, Roustache, ...)
pub fn status_to_html(status: &SlurmStatus) -> String {
    let mut result = String::new();

    // Page header and some CSS
    result.push_str("<html>\n");
    result.push_str("<head>\n");
    result.push_str("<title>Slurm Inspector</title>\n");
    result.push_str("<style>\n");
    result.push_str("table, { border: 1px solid black; }\n");
    result.push_str("th, td { border: 1px solid black; padding: 10px; }\n");
    result.push_str("th { background: #e0e0e0; }\n");
    result.push_str("#partition_down { background: #ffa0a0; }\n");
    result.push_str("</style>\n");
    result.push_str("</head>\n");
    result.push_str("<body>\n");

    // When was the SLURM status information last updated ?
    result.push_str(&format!("<h3>Last update: {}</h3>", status.last_update));

    result.push_str("<br>\n<br>\n<br>\n<br>\n");

    // Prepare first table (partition and node) with header
    result.push_str("<h3>Partition and node information:</h3>\n");
    result.push_str("<table>\n");
    result.push_str("<tr>\n");
    result.push_str("<th>Partition</th>");
    result.push_str("<th>Availability</th>");
    result.push_str("<th>Hostname</th>");
    result.push_str("<th>Node</th>");
    result.push_str("<th>Error</th>");
    result.push_str("<th>CPU load</th>");
    result.push_str("<th>Node state</th>");
    result.push_str("<th>Node sockets</th>");
    result.push_str("<th>Node cores</th>");
    result.push_str("<th>Node threads</th>");
    result.push_str("</tr>\n");

    for node in &status.node_info {
        result.push_str("<tr>\n");
        result.push_str(&format!("<td>{}</td>", node.partition));
        result.push_str(
            match node.availability {
                PartitionAvailability::Up => "<td>Up</td>",
                _ => "<td id=\"partition_down\">Down</td>"
            }
        );
        result.push_str(&format!("<td>{}</td>", node.hostname));
        result.push_str(&format!("<td>{}</td>", node.node));
        result.push_str(&format!("<td>{:?}</td>", node.error));
        result.push_str(node.cpu_load.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(format!("<td>{:?}</td>", node.node_state).as_ref());
        result.push_str(node.node_sockets.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(node.node_cores.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(node.node_threads.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str("</tr>\n");
    }

    result.push_str("</table>\n");

    result.push_str("<br>\n<br>\n<br>\n<br>\n");

    // Prepare second table (job information) with header
    result.push_str("<h3>Job information:</h3>\n");
    result.push_str("<table>\n");
    result.push_str("<tr>\n");
    result.push_str("<th>Executing host</th>");
    result.push_str("<th>Min CPU</th>");
    result.push_str("<th>Num CPU</th>");
    result.push_str("<th>Num nodes</th>");
    result.push_str("<th>Job array ID</th>");
    result.push_str("<th>Number of Sockets</th>");
    result.push_str("<th>Job ID</th>");
    result.push_str("<th>Number of Cores</th>");
    result.push_str("<th>Job name</th>");
    result.push_str("<th>Number of threads</th>");
    result.push_str("<th>Job array index</th>");
    result.push_str("<th>Run time</th>");
    result.push_str("<th>List of nodes</th>");
    result.push_str("<th>Priority</th>");
    result.push_str("<th>State reason</th>");
    result.push_str("<th>Start time</th>");
    result.push_str("<th>Job state</th>");
    result.push_str("<th>User name</th>");
    result.push_str("<th>User ID</th>");
    result.push_str("</tr>\n");

    for job in &status.job_info {
        result.push_str("<tr>\n");
        result.push_str(&format!("<td>{}</td>", job.executing_host));

        result.push_str(job.minimum_cpu.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(job.num_cpu.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(job.num_nodes.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(job.job_array_id.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(job.num_sockets.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(job.job_id.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(job.num_cores.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(&format!("<td>{}</td>", job.job_name));
        result.push_str(job.num_threads.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(job.job_array_index.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(&format!("<td>{}</td>", job.run_time));
        result.push_str("<td>");
        for node in &job.list_of_nodes {
            result.push_str(&format!("{},", node));
        }
        result.push_str("</td>");
        result.push_str(job.priority.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str(&format!("<td>{:?}</td>", job.state_reason));
        result.push_str(&format!("<td>{}</td>", job.start_time));
        result.push_str(&format!("<td>{:?}</td>", job.job_state));
        result.push_str(&format!("<td>{}</td>", job.user_name));
        result.push_str(job.user_id.map_or("<td>-</td>".to_string(), |val| format!("<td>{}</td>", val)).as_ref());
        result.push_str("</tr>\n");
    }

    result.push_str("</table>\n");

    result.push_str("</body>\n");
    result.push_str("</html>\n");

    result
}
