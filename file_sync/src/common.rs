pub use std::{ 
    sync::{ 
        Arc, 
        Mutex, 
        mpsc::channel
    },
    collections::HashMap,
    fs, 
    path::{ Path, PathBuf },
    fs::File,
    io::{ BufReader, Write, Read },
    task::{ Context, Poll }
};

pub use derive_new::new;

pub use reqwest::{ 
    Client, 
    Body
};

pub use tokio::{
    io::AsyncReadExt,
    task
}; 


pub use serde::{
    Deserialize, 
    Serialize,
    de::DeserializeOwned
};

pub use anyhow::anyhow;

pub use serde_json::{Value, from_reader};

pub use async_trait::async_trait;

pub use log::{info, error, warn};

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
    HttpResponse,
    // web::{Bytes,route},
    dev::{ ServiceRequest, ServiceResponse, Transform, Service },
    Error
};


pub use hotwatch::{
    Hotwatch, 
    Event, 
    EventKind as WatchEventKind
};


pub use sha2::{
    Sha256, 
    Digest
};


pub use futures::{
    stream::TryStreamExt,
    future::join_all,
    future::{
        Ready as FuterReady,
        ok
    }
};


pub use once_cell::sync::Lazy as once_lazy;


pub use chrono::{
    NaiveDate,
    NaiveDateTime,
    DateTime,
    Utc
};