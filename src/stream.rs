use axum::response::sse::{Event, KeepAlive, Sse};
use futures_util::StreamExt;
use futures_util::Stream;
use reqwest::Response as UpstreamResponse;
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::chat::ChatStreamChunk;
pub fn build_stream(
    upstream: UpstreamResponse,
    resp_id: String,
) -> Sse<impl Stream<Item = Result<Event, anyhow::Error>>> {
    let (tx, rx) = mpsc::channel::<Result<Event, anyhow::Error>>(256);

    tokio::spawn(async move {
        let upstream_stream = upstream.bytes_stream();
        let mut first_event = true;
        let mut output_text = String::new();
        let item_id = format!("msg_{}", &resp_id[5..]);

        tokio::pin!(upstream_stream);

        while let Some(chunk) = upstream_stream.next().await {
            match chunk {
                Ok(bytes) => {
                    let text = String::from_utf8_lossy(&bytes);
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data == "[DONE]" {
                                continue;
                            }
                            match serde_json::from_str::<ChatStreamChunk>(data) {
                                Ok(chunk) => {
                                    if let Some(content) = chunk.choices.first().and_then(|c| c.delta.content.as_deref()) {
                                        if first_event {
                                            send_event(&tx, "response.created", serde_json::json!({
                                                "type": "response.created",
                                                "response": {
                                                    "id": resp_id,
                                                    "object": "response",
                                                    "status": "in_progress",
                                                    "output": []
                                                }
                                            })).await;
                                            send_event(&tx, "response.output_item.added", serde_json::json!({
                                                "type": "response.output_item.added",
                                                "output_index": 0,
                                                "item": {
                                                    "id": item_id,
                                                    "type": "message",
                                                    "status": "in_progress",
                                                    "role": "assistant",
                                                    "content": []
                                                }
                                            })).await;
                                            send_event(&tx, "response.content_part.added", serde_json::json!({
                                                "type": "response.content_part.added",
                                                "item_id": item_id,
                                                "output_index": 0,
                                                "content_index": 0,
                                                "part": {
                                                    "type": "output_text",
                                                    "text": "",
                                                    "annotations": []
                                                }
                                            })).await;
                                            first_event = false;
                                        }

                                        output_text.push_str(content);
                                        send_event(&tx, "response.output_text.delta", serde_json::json!({
                                            "type": "response.output_text.delta",
                                            "item_id": item_id,
                                            "output_index": 0,
                                            "content_index": 0,
                                            "delta": content
                                        })).await;
                                    }

                                    if chunk.choices.first().and_then(|c| c.finish_reason.as_deref()).is_some() {
                                        send_event(&tx, "response.output_text.done", serde_json::json!({
                                            "type": "response.output_text.done",
                                            "item_id": item_id,
                                            "output_index": 0,
                                            "content_index": 0,
                                            "text": output_text
                                        })).await;
                                        send_event(&tx, "response.content_part.done", serde_json::json!({
                                            "type": "response.content_part.done",
                                            "item_id": item_id,
                                            "output_index": 0,
                                            "content_index": 0,
                                            "part": {
                                                "type": "output_text",
                                                "text": output_text,
                                                "annotations": []
                                            }
                                        })).await;
                                        send_event(&tx, "response.output_item.done", serde_json::json!({
                                            "type": "response.output_item.done",
                                            "output_index": 0,
                                            "item": {
                                                "id": item_id,
                                                "type": "message",
                                                "status": "completed",
                                                "role": "assistant",
                                                "content": [{
                                                    "type": "output_text",
                                                    "text": output_text,
                                                    "annotations": []
                                                }]
                                            }
                                        })).await;
                                        send_event(&tx, "response.completed", serde_json::json!({
                                            "type": "response.completed",
                                            "response": {
                                                "id": resp_id,
                                                "object": "response",
                                                "status": "completed",
                                                "output": [{
                                                    "id": item_id,
                                                    "type": "message",
                                                    "status": "completed",
                                                    "role": "assistant",
                                                    "content": [{
                                                        "type": "output_text",
                                                        "text": output_text,
                                                        "annotations": []
                                                    }]
                                                }],
                                                "output_text": output_text
                                            }
                                        })).await;
                                    }
                                }
                                Err(e) => {
                                    info!("failed to parse chat stream chunk: {e}");
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("upstream stream error: {e}");
                    break;
                }
            }
        }

        let _ = tx.send(Ok(Event::default().data("[DONE]"))).await;
    });

    let rx_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
    Sse::new(rx_stream).keep_alive(KeepAlive::default())
}

async fn send_event(
    tx: &mpsc::Sender<Result<Event, anyhow::Error>>,
    event_name: &'static str,
    payload: serde_json::Value,
) {
    if let Ok(json) = serde_json::to_string(&payload) {
        let _ = tx.send(Ok(Event::default().event(event_name).data(json))).await;
    }
}
