use axum::extract::ws::{Message, WebSocket};
use bollard::Docker;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tracing::{error, info};

pub async fn handle_exec_stream(stream: WebSocket, docker: Arc<Docker>, id: String) {
    let (mut sender, mut receiver) = stream.split();

    let exec_id = docker
        .create_exec(
            &id,
            bollard::models::ExecConfig {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                attach_stdin: Some(true),
                tty: Some(true),
                cmd: Some(vec![String::from("bash")]),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .id;

    // if let bollard::exec::StartExecResults::Attached {
    //     mut output,
    //     mut input,
    // } = docker.start_exec(&exec_id, None).await.unwrap()
    // {
    //     // Forward WebSocket input to Docker exec stdin
    //     tokio::spawn(async move {
    //         while let Some(Ok(msg)) = receiver.next().await {
    //             if let Message::Text(text) = msg {
    //                 info!(" Websocket input to Docker exec {:?}", text);
    //                 input.write_all(text.as_bytes()).await.ok();
    //             }
    //         }
    //     });

    // Forward Docker exec stdout to WebSocket
    // while let Some(Ok(msg)) = output.next().await {
    //     let bytes = msg.into_bytes();
    //     if sender.send(Message::Binary(bytes)).await.is_err() {
    //         break;
    //     }
    // }

    if let bollard::exec::StartExecResults::Attached {
        mut output,
        mut input,
    } = docker.start_exec(&exec_id, None).await.unwrap()
    {
        // Forward WebSocket input to Docker exec stdin
        tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(text) = msg {
                    info!("WebSocket input to Docker exec: {:?}", text);

                    if text.trim() == "__disconnect__" {
                        info!("Disconnect message received. Closing Docker exec input.");
                        break; // Exit the loop to stop forwarding
                    }

                    if let Err(e) = input.write_all(text.as_bytes()).await {
                        error!("Failed to write to Docker exec stdin: {:?}", e);
                        break;
                    }
                }
            }

            // Optional: close the input stream explicitly
            drop(input);
            info!("Docker exec input stream closed.");
        });

        use axum::extract::ws::Utf8Bytes;
        while let Some(Ok(msg)) = output.next().await {
            let bytes = msg.into_bytes();
            let text = String::from_utf8_lossy(&bytes).to_string();
            info!("Docker exec output {:?}", text);
            let utf8 = Utf8Bytes::from(text);
            if sender.send(Message::Text(utf8)).await.is_err() {
                break;
            }
        }
    }
}
