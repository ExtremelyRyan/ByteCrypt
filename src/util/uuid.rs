use std::time::SystemTime;

pub fn generate_uuid() {
    let ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let random_bytes = rand::random();

    let uuid = uuid::Builder::from_unix_timestamp_millis(
        ts.as_millis().try_into().unwrap(),
        &random_bytes,
    )
    .into_uuid();

    println!("{:?}", uuid);
}
