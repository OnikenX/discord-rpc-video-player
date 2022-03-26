extern crate discord_rpc_client;

use discord_rpc_client::Client;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::Read;
use std::iter::Map;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{env, thread, time};

//this funcion spawns xdotool instances to find the window id and the title name of an window
fn getwindownamebypid(pid: &str) -> Option<String> {
    //getting window id
    let mut search_id = Command::new("xdotool");
    search_id
        .arg("search")
        .arg("--pid")
        .arg(pid)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child_search = search_id.spawn().unwrap();
    if child_search.wait().is_err() {
        return None;
    }
    let mut window_id = String::new();
    let _ = child_search.stdout.unwrap().read_to_string(&mut window_id);

    //getting window title
    let mut window_title_get = Command::new("xdotool");
    window_title_get
        .arg("getwindowname")
        .arg(window_id)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child_title = window_title_get.spawn().unwrap();
    if child_title.wait().is_err() {
        return None;
    }
    let mut window_title = String::new();
    let _ = child_title
        .stdout
        .unwrap()
        .read_to_string(&mut window_title);
    return Some(window_title);
}

//its the code of each individial client
fn client(rx: Receiver<Option<String>>) {
    let mut drpc = Client::new(957085916252491806);

    // Start up the client connection, so that we can actually send and receive stuff
    drpc.start();

    loop {
        match rx.recv() {
            Ok(name) => {
                if name.is_none() {
                    break;
                }
                let _ = drpc.set_activity(|act| act.state(name.unwrap()));
            }
            Err(_) => break,
        }
    }
    let _ = drpc.clear_activity();

    // Wait 10 seconds before exiting
}

//the manager for all the clients
//its job is too keep track on all the clients that exist
fn client_manager(rx: Receiver<HashMap<String, String>>) {
    let mut map = HashMap::<String, (Sender<Option<String>>, JoinHandle<()>, String)>::new();
    loop {
        let mut register_pids = HashSet::new();
        map.iter().for_each(|(pid, _)| {
            register_pids.insert(pid.clone());
        });

        let new_set = rx.recv().unwrap();
        for (pid, message) in new_set {
            if map.get(&pid).is_none() {
                let (mut sx, mut rx) = channel();
                let mut joinHandle = thread::spawn(|| client(rx));
                map.insert(pid.clone(), (sx.clone(), joinHandle, message.clone()));
                sx.send(Option::Some(message.clone()));
            } else {
                let x = map.get_mut(&pid).unwrap();
                if x.2 != message {
                    x.0.send(Some(message.clone()));
                    x.2 = message;
                }
                register_pids.remove(&pid);
            }
        }

        for pid in &register_pids {
            let item = map.remove(pid).unwrap();
            let _ = item.0.send(None);
            let joiner = item.1;
            joiner.join().unwrap();
        }

        if env::args()
            .nth(1)
            .and_then(|f| {
                if f.contains("debug") {
                    Some(String::new())
                } else {
                    None
                }
            })
            .is_some()
        {
            println!("N services: {}", map.len());
            for (pid, (_, _, msg)) in &map {
                println!("{{{pid};{msg}}}");
            }
        }
    }
}
fn main() {
    let programs_to_look_for = ["mpv", "vlc"];

    let (sx, rx) = channel();
    thread::spawn(|| client_manager(rx));
    let mut command = Command::new("/bin/ps");
    command.arg("x").stdout(Stdio::piped());
    loop {
        let mut child = command.spawn().unwrap();
        let _ = child.wait();
        let mut output = String::new();
        let _ = child.stdout.take().unwrap().read_to_string(&mut output);
        let mut pid = String::new();
        let mut list_pid_names = HashMap::new();
        for line in output.lines() {
            if programs_to_look_for.iter().any(|f| line.find(f).is_some()) {
                pid = line.split(" ").nth(0).unwrap().to_string();
                let name = getwindownamebypid(&pid);
                if name.is_none() {
                    continue;
                }
                let name = name.unwrap();
                if name.is_empty() {
                    continue;
                }
                list_pid_names.insert(pid, name);
            }
        }
        sx.send(list_pid_names);
        thread::sleep(Duration::from_secs(10));
    }

    // Create the client
}
