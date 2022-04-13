use std::io::Error as IoError;
use std::path::Path;
use std::sync::Arc;

use async_std::{fs::OpenOptions, io};
use tempfile::TempDir;
use envoy::{prelude::*, Context};
use envoy::{Body, Response, StatusCode};

#[derive(Clone)]
struct TempDirState {
    tempdir: Arc<TempDir>,
}

impl TempDirState {
    fn try_new() -> Result<Self, IoError> {
        Ok(Self {
            tempdir: Arc::new(tempfile::tempdir()?),
        })
    }

    fn path(&self) -> &Path {
        self.tempdir.path()
    }
}

#[async_std::main]
async fn main() -> Result<(), IoError> {
    envoy::log::start();
    let mut app = envoy::with_state(TempDirState::try_new()?);

    // To test this example:
    // $ cargo run --example upload
    // $ curl -T ./README.md localhost:8080 # this writes the file to a temp directory
    // $ curl localhost:8080/README.md # this reads the file from the same temp directory

    app.at(":file")
        .put(|ctx: Context<TempDirState>| async move {
            let path = ctx.param("file")?;
            let fs_path = ctx.state().path().join(path);

            let file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(&fs_path)
                .await?;

            let bytes_written = io::copy(ctx.req, file).await?;

            envoy::log::info!("file written", {
                bytes: bytes_written,
                path: fs_path.canonicalize()?.to_str()
            });

            Ok(json!({ "bytes": bytes_written }))
        })
        .get(|ctx: Context<TempDirState>| async move {
            let path = ctx.param("file")?;
            let fs_path = ctx.state().path().join(path);

            if let Ok(body) = Body::from_file(fs_path).await {
                Ok(body.into())
            } else {
                Ok(Response::new(StatusCode::NotFound))
            }
        });

    app.listen("127.0.0.1:8080").await?;
    Ok(())
}
