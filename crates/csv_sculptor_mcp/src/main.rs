use csv_sculptor_mcp::{CsvSculptorServer, MAX_STDIO_MESSAGE_BYTES};
use futures_util::{StreamExt, future::ready};
use rmcp::{
    RoleServer, ServiceExt,
    service::{RxJsonRpcMessage, TxJsonRpcMessage},
    transport::async_rw::JsonRpcMessageCodec,
};
use std::error::Error;
use tokio::io::{stdin, stdout};
use tokio_util::codec::{FramedRead, FramedWrite};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    eprintln!("csv_sculptor_mcp starting in bounded stdio mode");

    let reader = FramedRead::new(
        stdin(),
        JsonRpcMessageCodec::<RxJsonRpcMessage<RoleServer>>::new_with_max_length(
            MAX_STDIO_MESSAGE_BYTES,
        ),
    )
    .filter_map(|message| {
        ready(match message {
            Ok(message) => Some(message),
            Err(error) => {
                eprintln!("MCP input discarded: {error}");
                None
            }
        })
    });
    let writer = FramedWrite::new(
        stdout(),
        JsonRpcMessageCodec::<TxJsonRpcMessage<RoleServer>>::new_with_max_length(
            MAX_STDIO_MESSAGE_BYTES,
        ),
    );

    let service = CsvSculptorServer::new().serve((writer, reader)).await?;
    service.waiting().await?;
    Ok(())
}
