use bytes::BytesMut;
use std::collections::HashMap;

#[derive(Debug)]
pub struct MultiPart {
    parts: Vec<Part>,
}

impl MultiPart {
    pub fn new(body: BytesMut, boundary: String) -> MultiPart {
        let parts = get_parts(body, boundary);
        MultiPart { parts }
    }
}

fn get_parts(body: BytesMut, boundary: String) -> Vec<Part> {
    let mut retval: Vec<Part> = Vec::new();
    let data = String::from_utf8(body.to_vec()).unwrap();
    let splits = data.split(boundary.as_str());

    for split in splits {
        if split != "--\r\n" {
            retval.push(Part::new(split));
        }
    }

    retval
}

#[derive(Debug)]
struct Part {
    content_disposition: HashMap<String, String>,
    content_type: Option<String>,
    data: Vec<u8>,
}

impl Part {
    pub fn new(header: &str) -> Part {
        let (content_disposition, content_type, data) = build_attributes(header);
        return Part {
            content_disposition,
            content_type,
            data,
        };
    }
}

fn build_attributes(raw_content: &str) -> (HashMap<String, String>, Option<String>, Vec<u8>) {
    let mut splits = raw_content.split("\r\n\r\n");
    let content_disposition = get_content_disposition(splits.next().unwrap());
    let part2 = splits.next().unwrap();
    if let Some(part3) = splits.next() {
        let type_splits = part2.split(":").collect::<Vec<_>>();
        return (
            content_disposition,
            Some(type_splits[1].to_string()),
            part3.to_string().as_bytes().to_vec(),
        );
    } else {
        return (
            content_disposition,
            None,
            part2.to_string().as_bytes().to_vec(),
        );
    }
}

fn get_content_disposition(raw_content_disposition: &str) -> HashMap<String, String> {
    let mut content_disposition = HashMap::new();
    let splits = raw_content_disposition.split(";");
    for split in splits {
        let mut chunks = split.split("=");
        content_disposition.insert(
            chunks.next().unwrap().to_string(),
            chunks.next().unwrap().to_string(),
        );
    }
    content_disposition
}
