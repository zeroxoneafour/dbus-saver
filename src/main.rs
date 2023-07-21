// been a while not gonna lie
use std::{
    io::prelude::*,
    io::BufReader,
    collections::HashMap,
    fs::{
        File,
        DirBuilder,
    },
    path::Path,
    error::Error,
    sync::{
        mpsc,
        mpsc::{Sender, Receiver},
    },
    thread,
};
use dbus_crossroads::{Crossroads, Context};
use signal_hook::{
    consts::{
        SIGINT,
        SIGTERM
    },
    iterator::Signals,
};
use dbus::blocking::Connection;

struct SettingSaver {
    file_path: String,
    settings: HashMap<String, String>,
}

impl Default for SettingSaver {
    fn default() -> Self {
        let mut file_path = String::default();
        if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
            file_path += &xdg_config_home;
        } else {
            file_path += &std::env::var("HOME").unwrap();
            file_path += "/.config";
        }
        file_path += "/SettingSaver";
        DirBuilder::new().recursive(true).create(&file_path).unwrap();
        file_path += "/settings.txt";
        println!("Loading settings from {file_path}");
        let mut ret = Self {
            file_path,
            settings: HashMap::default(),
        };
        ret.read_settings().unwrap();
        ret
    }
}

impl SettingSaver {
    fn read_settings(&mut self) -> Result<(), Box<dyn Error>> {
        let mut read_key = true;
        let mut old_key = "".to_string();
        let Ok(file) = File::open(Path::new(&self.file_path)) else { return Ok(()) };
        for l in BufReader::new(file).lines() {
            let line = l?;
            if read_key { // reading key
                old_key = line;
                read_key = false;
            } else {
                self.settings.insert(old_key.clone(), line);
                read_key = true;
            }
        }
        Ok(())
    }
    fn save_settings(&mut self) -> Result<(), Box<dyn Error>> {
        let mut file = File::create(Path::new(&self.file_path))?;
        for (key, value) in &self.settings {
            file.write((key.clone() + "\n").as_bytes())?;
            file.write((value.clone() + "\n").as_bytes())?;
        }
        file.sync_all()?;
        Ok(())
    }
    fn get_settings(&self, desktop: String) -> String {
        self.settings.get(&desktop).unwrap_or(&"".to_string()).clone()
    }
    fn set_settings(&mut self, desktop: String, settings: String) {
        self.settings.insert(desktop, settings);
    }
}

enum Action {
    GetSettings(String),
    SetSettings(String, String),
    Exit,
}

fn signal_listener(tx: Sender<Action>) {
    let mut signals = Signals::new(&[SIGINT, SIGTERM]).unwrap();
    for _ in signals.forever() {
        tx.send(Action::Exit).unwrap();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut saver = SettingSaver::default();
    let conn = Connection::new_session()?;
    conn.request_name("org.polonium.SettingSaver", false, true, false)?;
    
    let mut cr = Crossroads::new();
    
    let (to_sender_tx, to_sender_rx): (Sender<Action>, Receiver<Action>) = mpsc::channel();
    let (to_dbus_tx, to_dbus_rx): (Sender<String>, Receiver<String>) = mpsc::channel();
    
    let iface_token = cr.register("org.polonium.SettingSaver", |b| {
        b.method("Exists", (), ("true",), move |_, _, _: ()| {
            Ok((true,))
        });
        b.method("GetSettings", ("desktop",), ("desktop","json",), move |_ctx: &mut Context, (tx, rx): &mut (Sender<Action>, Receiver<String>), (desktop,): (String,)| {
            // And here's what happens when the method is called.
            println!("Getting settings");
            let desktop_ret = desktop.clone();
            tx.send(Action::GetSettings(desktop)).unwrap();
            let json = rx.recv().unwrap();
            Ok((desktop_ret,json,))
        });
        b.method("SetSettings", ("desktop", "json",), (), move |_ctx: &mut Context, (tx, _rx): &mut (Sender<Action>, Receiver<String>), (desktop, json,): (String, String,)| {
            // And here's what happens when the method is called.
            println!("Setting settings");
            tx.send(Action::SetSettings(desktop, json)).unwrap();
            Ok(())
        });
    });

    // Let's add the "/hello" path, which implements the com.example.dbustest interface,
    // to the crossroads instance.
    cr.insert("/saver", &[iface_token], (to_sender_tx.clone(), to_dbus_rx));

    
    thread::spawn(move || signal_listener(to_sender_tx));
    
    thread::spawn(move || cr.serve(&conn));
    
    saver.save_settings()?;
    while let Ok(action) = to_sender_rx.recv() {
        match action {
            Action::GetSettings(desktop) => {
                to_dbus_tx.send(saver.get_settings(desktop))?;
            },
            Action::SetSettings(desktop, json) => {
                saver.set_settings(desktop, json);
            }
            Action::Exit => {
                println!("\nExiting");
                saver.save_settings()?;
                break
            },
        }
    }
    Ok(())
}
