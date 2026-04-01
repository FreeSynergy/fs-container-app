//! `fs-container` — FreeSynergy container service daemon and CLI.
//!
//! # Environment variables
//!
//! | Variable       | Default |
//! |----------------|---------|
//! | `FS_GRPC_PORT` | `50093` |
//! | `FS_REST_PORT` | `8093`  |

#![deny(clippy::all, clippy::pedantic, warnings)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::ignored_unit_patterns)]
#![allow(clippy::needless_pass_by_value)]

use clap::Parser as _;
use fs_container::QuadletManager;
use tracing_subscriber::{fmt, EnvFilter};

use fs_container_app::{
    cli::{Cli, Command},
    controller::ContainerAppController,
    grpc::{ContainerAppServiceServer, GrpcContainerApp},
    rest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let args = Cli::parse();
    let ctrl = ContainerAppController::new(QuadletManager::user_default());

    match args.command {
        Command::Daemon => run_daemon(ctrl).await?,
        ref cmd => run_cli(cmd, &ctrl).await,
    }
    Ok(())
}

async fn run_daemon(
    ctrl: ContainerAppController<QuadletManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    let grpc_port: u16 = std::env::var("FS_GRPC_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(50_093);
    let rest_port: u16 = std::env::var("FS_REST_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8_093);

    let grpc_addr: std::net::SocketAddr = ([0, 0, 0, 0], grpc_port).into();
    let rest_addr: std::net::SocketAddr = ([0, 0, 0, 0], rest_port).into();

    tracing::info!("gRPC on {grpc_addr}, REST on {rest_addr}");

    let grpc_ctrl = ctrl.clone();
    let grpc_task = tokio::spawn(async move {
        tonic::transport::Server::builder()
            .add_service(ContainerAppServiceServer::new(GrpcContainerApp::new(
                grpc_ctrl,
            )))
            .serve(grpc_addr)
            .await
            .unwrap();
    });

    let rest_task = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(rest_addr).await.unwrap();
        axum::serve(listener, rest::router(ctrl)).await.unwrap();
    });

    tokio::try_join!(grpc_task, rest_task)?;
    Ok(())
}

async fn run_cli(cmd: &Command, ctrl: &ContainerAppController<QuadletManager>) {
    match cmd {
        Command::Daemon => unreachable!(),
        Command::List => {
            ctrl.refresh().await;
            let snap = ctrl.snapshot();
            if snap.containers.is_empty() {
                println!("(no services)");
            } else {
                for c in snap.containers {
                    println!("{:30}  {}", c.name, c.state_label);
                }
            }
        }
        Command::Start { name } => match ctrl.start(name).await {
            Ok(()) => println!("Started {name}"),
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        },
        Command::Stop { name } => match ctrl.stop(name).await {
            Ok(()) => println!("Stopped {name}"),
            Err(e) => {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
        },
        Command::Logs { name, lines } => {
            for line in ctrl.logs(name, *lines).await {
                println!("{line}");
            }
        }
    }
}
