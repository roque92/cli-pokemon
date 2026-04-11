//! Launches (or stops) the background daemon that draws the walking pet.

use std::{
    io::{self, IsTerminal},
    path::PathBuf,
    process::Stdio,
};

use anyhow::Result;

use crate::{
    renderer::RendererKind,
    sprite,
    state::{party::Party, pet::PetStatus},
};

// ── Public API ────────────────────────────────────────────────────────────────

/// Start the pet daemon. Caches the sprite first so the daemon starts
/// instantly, then detaches it into the background and returns the prompt.
pub async fn run(party: Party, renderer_kind: RendererKind) -> Result<()> {
    if !io::stdout().is_terminal() {
        anyhow::bail!("petcli requires an interactive terminal.");
    }

    // Already running?
    if let Some(pid) = read_pid()? {
        if process_alive(pid) {
            println!("Pet already running (PID {pid}). Use `petcli stop` to remove it.");
            return Ok(());
        }
        // Stale pid file
        let _ = std::fs::remove_file(pid_path()?);
    }

    // Collect all alive pets
    let alive: Vec<_> = party
        .slots
        .iter()
        .flatten()
        .filter(|p| matches!(p.status, PetStatus::Alive))
        .collect();

    if alive.is_empty() {
        println!("No Pokemon set. Run `petcli set <name>` first.");
        return Ok(());
    }

    // Warm sprite cache for all alive pets so the daemon starts instantly
    for pet in &alive {
        print!("Loading {}... ", pet.display_name());
        let _ = io::Write::flush(&mut io::stdout());
        match sprite::load_sheet(pet.species_id, pet.shiny).await {
            Ok(_) => println!("ok"),
            Err(e) => println!("(offline: {e})"),
        }
    }

    // Spawn daemon (self with hidden --daemon flag)
    let exe = std::env::current_exe()?;
    let renderer_flag = match renderer_kind {
        RendererKind::Kitty   => "kitty",
        RendererKind::Unicode => "unicode",
    };

    let mut cmd = std::process::Command::new(&exe);
    cmd.args(["--daemon", renderer_flag])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    // On Windows, prevent a console window from appearing for the child process
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let child = cmd.spawn()?;
    let pid = child.id();
    std::fs::write(pid_path()?, pid.to_string())?;
    std::mem::forget(child); // detach — let it run in background

    let names: Vec<String> = alive.iter().map(|p| p.display_name()).collect();
    println!("{} now walking in your terminal!", names.join(", "));
    println!("Run `petcli stop` to remove  |  `petcli add <name>` to add more");
    Ok(())
}

/// Kill the daemon and restore the terminal.
pub fn stop() -> Result<()> {
    let Some(pid) = read_pid()? else {
        println!("No pet is running.");
        return Ok(());
    };

    if process_alive(pid) {
        kill_process(pid);
        // Give it a moment to clean up
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("Pet stopped.");
    } else {
        println!("Pet was not running (stale PID file).");
    }

    let _ = std::fs::remove_file(pid_path()?);
    Ok(())
}

// ── Platform: process management ─────────────────────────────────────────────

#[cfg(unix)]
fn kill_process(pid: u32) {
    unsafe { libc::kill(pid as i32, libc::SIGTERM) };
}

#[cfg(windows)]
fn kill_process(pid: u32) {
    // taskkill /F forces immediate termination (no graceful SIGTERM equivalent
    // for arbitrary processes on Windows without the windows-sys crate)
    let _ = std::process::Command::new("taskkill")
        .args(["/F", "/PID", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(unix)]
fn process_alive(pid: u32) -> bool {
    // kill(pid, 0) returns 0 if process exists and we can signal it
    unsafe { libc::kill(pid as i32, 0) == 0 }
}

#[cfg(windows)]
fn process_alive(pid: u32) -> bool {
    // Query via tasklist; filter by exact PID in CSV output
    std::process::Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/NH", "/FO", "CSV"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
        .map(|o| {
            let out = String::from_utf8_lossy(&o.stdout);
            // tasklist CSV output: "petcli.exe","1234",...
            out.contains(&format!("\"{pid}\""))
        })
        .unwrap_or(false)
}

// ── PID file helpers ──────────────────────────────────────────────────────────

fn pid_path() -> Result<PathBuf> {
    let dir = crate::state::persist::data_dir()?;
    Ok(dir.join("daemon.pid"))
}

fn read_pid() -> Result<Option<u32>> {
    let path = pid_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let s = std::fs::read_to_string(&path)?;
    Ok(s.trim().parse::<u32>().ok())
}
