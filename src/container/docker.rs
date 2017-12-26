

use container::Container;
use libc::pid_t;
use std::process::Command;
use types::{Error, Result};
use unistd::Pid;

#[derive(Clone, Debug)]
pub struct Docker {}

impl Container for Docker {
    fn lookup(&self, container_id: &str) -> Result<Pid> {
        let command = format!(
            "docker inspect --format '{{.State.Status}};{{.State.Pid}}' {}",
            container_id
        );
        let output = tryfmt!(
            Command::new("docker")
                .args(
                    &[
                        "inspect",
                        "--format",
                        "{{.State.Status}};{{.State.Pid}}",
                        container_id,
                    ],
                )
                .output(),
            "Running '{}' failed",
            command
        );

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return errfmt!(format!(
                "Failed to list containers. '{}' exited with {}: {}",
                command,
                output.status,
                stderr.trim_right()
            ));
        }

        let fields: Vec<&[u8]> = output.stdout.splitn(2, |c| *c == b';').collect();
        assert!(fields.len() == 2);

        if fields[0] != b"running" {
            let state = String::from_utf8_lossy(fields[0]);
            return errfmt!(format!(
                "container '{}' is not running, got state: {}",
                container_id,
                state
            ));
        }

        let pid = String::from_utf8_lossy(fields[1]);

        Ok(Pid::from_raw(tryfmt!(
            pid.trim_right().parse::<pid_t>(),
            "expected valid process id from '{}', got: {}",
            command,
            pid
        )))
    }
    fn check_required_tools(&self) -> Result<()> {
        tryfmt!(
            Command::new("docker").arg("--version").output(),
            "cannot execute `docker`"
        );
        Ok(())
    }
}
