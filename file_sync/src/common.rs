pub use std::{ 
    sync::{ 
        Arc, 
        Mutex, 
        mpsc::channel,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard
    },
    collections::HashMap,
    fs, 
    path::{ Path, PathBuf },
    fs::File,
    io::{ BufReader, Write, Read },
    task::{ Context, Poll },
    time::Duration,
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

pub use getset::Getters;

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
    },
    Future
};


pub use once_cell::sync::Lazy as once_lazy;


pub use chrono::{
    NaiveDate,
    NaiveDateTime,
    DateTime,
    Utc
};


pub use elasticsearch::{
    Elasticsearch, 
    http::transport::{ SingleNodeConnectionPool, TransportBuilder },
    http::Url,
    http::response::Response,
    SearchParts, 
    IndexParts, 
    DeleteParts,
    http::transport::{ Transport, ConnectionPool }
};


pub use rand:: {
    prelude::SliceRandom,
    rngs::StdRng,
    SeedableRng
};