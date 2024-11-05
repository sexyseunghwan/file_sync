



pub use std::{ 
    sync::{ Arc, Mutex, mpsc::channel },
    collections::HashMap,
    io::Write,
    env, fs, cmp, thread,
    process::Command,
    time::Duration,
    path::Path
};


pub use reqwest::Client;

pub use tokio::fs::{read_to_string, write};

pub use serde::{Deserialize, Serialize};

pub use actix_files::Files;



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


pub use notify::{
    RecommendedWatcher, RecursiveMode, Watcher, Config, EventKind
};