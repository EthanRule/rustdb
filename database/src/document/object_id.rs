use rand::Rng;
use std::time::SystemTime;

struct ObjectId {
    bytes: [u8; 12],
}

impl ObjectId {
    pub fn new() -> Self {
        let mut bytes = [0u8; 12];
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as u32;

        bytes[0..4].copy_from_slice(&now.to_be_bytes());

        let mut rng = rand::thread_rng();
        rng.fill(&mut bytes[4..]);

        ObjectId { bytes }
    }
}
