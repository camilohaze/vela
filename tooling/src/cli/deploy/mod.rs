//! Cloud deployment providers for Vela
//!
//! This module contains implementations for deploying Vela applications
//! to various cloud platforms including AWS Lambda, Vercel, Netlify, and Azure Functions.

pub mod aws_lambda;
pub mod common;

pub use aws_lambda::AwsLambdaDeployer;
pub use common::{DeploymentConfig, DeploymentResult, Deployer};