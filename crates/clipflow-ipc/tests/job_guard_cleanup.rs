use clipflow_ipc::job::JobGuard;
use std::process::Command;

#[test]
fn test_job_guard_cleanup() {
    let job = JobGuard::new().expect("Failed to create job object");
    
    let mut child = Command::new("ping")
        .args(["127.0.0.1", "-n", "30"])
        .spawn()
        .expect("Failed to spawn child process");
        
    let pid = child.id();
    
    job.assign_process(pid).expect("Failed to assign process to job");
    
    drop(job);
    
    std::thread::sleep(std::time::Duration::from_millis(200));
    
    let status = child.try_wait().expect("Failed to try_wait on child process");
    
    assert!(status.is_some(), "Child process is still running after JobGuard dropped!");
}
