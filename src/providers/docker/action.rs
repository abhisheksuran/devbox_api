use axum::extract::ws::{Message, WebSocket};
use bollard::Docker;
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

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

    if let bollard::exec::StartExecResults::Attached {
        mut output,
        mut input,
    } = docker.start_exec(&exec_id, None).await.unwrap()
    {
        // Forward WebSocket input to Docker exec stdin
        tokio::spawn(async move {
            while let Some(Ok(msg)) = receiver.next().await {
                if let Message::Text(text) = msg {
                    input.write_all(text.as_bytes()).await.ok();
                }
            }
        });

        // Forward Docker exec stdout to WebSocket
        // while let Some(Ok(msg)) = output.next().await {
        //     let bytes = msg.into_bytes();
        //     if sender.send(Message::Binary(bytes)).await.is_err() {
        //         break;
        //     }
        // }
        use axum::extract::ws::Utf8Bytes;
        while let Some(Ok(msg)) = output.next().await {
            let bytes = msg.into_bytes();
            let text = String::from_utf8_lossy(&bytes).to_string();
            let utf8 = Utf8Bytes::from(text);
            if sender.send(Message::Text(utf8)).await.is_err() {
                break;
            }
        }
    }
}
