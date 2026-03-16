//! Grammar Check CLI Library
//!
//! A CLI tool for checking and fixing grammar using LLMs (local Ollama or cloud APIs).

#![deny(clippy::unwrap_used)]

pub mod commands;
pub mod models;
pub mod output;
pub mod providers;
pub mod services;
