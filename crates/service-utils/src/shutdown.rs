use tokio::signal;
use tokio_util::sync::CancellationToken;
use tracing::warn;

pub async fn wait_for_shutdown_signal(token: CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Couldn't set up Ctrl+C handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Couldn't set up SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            warn!("Got Ctrl+C (SIGINT). Cancelling token...");
        },
        _ = sigterm => {
            warn!("Got SIGTERM. Cancelling token...");
        },
    }

    token.cancel();
}
