mod utils;

use std::io::{self, Write};
use std::process::Command;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use utils::format_path;
fn main() {
    let home_dir = env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("/")); // found on reddit
    let mut current_dir = env::current_dir().expect("No dir found");

    // shared foreground process for ctrl + c
    let fg_child: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
    {
        let fg_clone = fg_child.clone();
        let _ = ctrlc::set_handler(move || {
            if let Some(pid) = *fg_clone.lock().unwrap() {
                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGINT);
            }
        });
    }

    loop {
        let display_dir = format_path(&current_dir, &home_dir);
        print!("{}:># ", display_dir);
        io::stdout().flush().expect("Unable to flush");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Couldn’t read line");
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // check for background job
        let background = input.ends_with('&');
        let input = if background {
            input.trim_end_matches('&').trim_end()
        } else {
            input
        };

        // split command
        let mut parts = input.split_whitespace();
        let cmd = parts.next().expect("Couldn’t define cmd");
        let args: Vec<&str> = parts.collect();

        match cmd {
            "exit" => break,
            "help" => println!("builtins: help, cd, exit, clear"),
            "cd" => {
                let target = args.get(0).map(|s| *s).unwrap_or("~");
                let full_path: PathBuf = if target == "~" {
                    home_dir.clone()
                } else {
                    let p = Path::new(target);
                    if p.is_absolute() {
                        p.to_path_buf()
                    } else {
                        current_dir.join(p)
                    }
                };

                if let Err(e) = env::set_current_dir(&full_path) {
                    eprintln!("cd: {}: {}", full_path.display(), e);
                } else {
                    current_dir = env::current_dir().unwrap_or(full_path);
                }
            }
            "clear" => print!("\x1B[2J\x1B[1;1H"),
            _ => {
                match Command::new(cmd).args(args).spawn() {
                    Ok(mut child) => {
                        if background {
                            println!("[{}] started", child.id());
                        } else {
                            {
                                let mut fg = fg_child.lock().unwrap();
                                *fg = Some(child.id());
                            }

                            let _ = child.wait();

                            let mut fg = fg_child.lock().unwrap();
                            *fg = None;
                        }
                    }
                    Err(_) => eprintln!("verish: command not found: {}", cmd),
                }
            }
        }
    }
}
