pub use std::{ 
    sync::{ Arc, Mutex, mpsc::channel },
    collections::HashMap,
    env, fs, cmp, thread,
    process::Command,
    time::Duration,
    path::{ Path, PathBuf },
    fs::File,
    io::{ BufReader, Write, BufRead, Read }
};


pub use reqwest::Client;

pub use tokio::fs::{read_to_string, write};

pub use serde::{
    Deserialize, 
    Serialize,
    de::DeserializeOwned
};

pub use serde_json::{Value, from_reader};

pub use actix_files::Files;

pub use async_trait::async_trait;

pub use log::{info, error};

pub use flexi_logger::{
    Logger, 
    FileSpec, 
    Criterion, 
    Age, 
    Naming, 
    Cleanup, 
    Record
};


pub use actix_web::{
    web, 
    App, 
    HttpServer, 
    Responder, 
    HttpRequest
};


pub use hotwatch::{Hotwatch, Event, EventKind as WatchEventKind};

pub use sha2::{Sha256, Digest};