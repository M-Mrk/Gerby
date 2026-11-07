use serde::Serialize;

use tracing::{error, info};

pub type MessageData<'a> = Vec<(&'a str, &'a str)>;

#[derive(Serialize, Debug)]
struct Message<'a> {
    kind: &'a str,
    id: &'a str,
    data: MessageData<'a>,
}

fn send_message(kind: &str, id: &str, data: MessageData) {
    let msg = Message { kind, id, data };
    let json = serde_json::to_string(&msg).unwrap_or_else(|err| {
        error!(
            "Failed to create json from message: {:?}, because of error: {}",
            msg, err
        );
        std::process::exit(1);
    });
    info!("Outputing json: {}", json);
    println!("{}", json);
}

// Output update json message
pub fn output_update(id: &str, data: MessageData) {
    send_message("update", id, data);
}

// Output error json message and exit with error
pub fn output_error(id: &str, data: MessageData) -> ! {
    send_message("error", id, data);
    std::process::exit(1);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_message_serialisation() {
        let msg = Message {
            kind: "test",
            id: "valid_test",
            data: vec![("key1", "value1"), ("key2", "value2")],
        };
        let json =
            r#"{"kind":"test","id":"valid_test","data":[["key1","value1"],["key2","value2"]]}"#;
        assert_eq!(json, serde_json::to_string(&msg).unwrap())
    }
}
