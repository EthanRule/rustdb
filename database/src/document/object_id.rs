use chrono::{DateTime, TimeZone, Utc};
use hex::{FromHex, ToHex};
use rand::Rng;
use std::fmt;
use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ObjectId {
    bytes: [u8; 12],
}

// Traits
impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in &self.bytes {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
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

    pub fn from_bytes(bytes: [u8; 12]) -> Self {
        ObjectId { bytes }
    }

    pub fn to_bytes(&self) -> [u8; 12] {
        self.bytes
    }

    pub fn from_hex(s: &str) -> Result<Self, hex::FromHexError> {
        let arr = <[u8; 12]>::from_hex(s)?;
        Ok(ObjectId { bytes: arr })
    }

    pub fn to_hex(&self) -> String {
        self.bytes.encode_hex()
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        let ts = u32::from_be_bytes(self.bytes[0..4].try_into().unwrap());
        Utc.timestamp(ts as i64, 0)
    }
}

// ObjectId Benchmark for generation speeds
pub fn Object_Id_Benchmark {
    let start = SystemTime::now()
    let ObjectId = ObjectId::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use proptest::prelude::*;

    #[test]
    fn test_create_new_object_id() {
        let object = ObjectId::new();
        assert_eq!(object.bytes.len(), 12);
    }

    #[test]
    fn test_from_bytes() {
        let bytes = [5u8; 12];
        let object = ObjectId::from_bytes(bytes);
        assert_eq!(object.bytes, bytes);
    }

    #[test]
    fn test_to_bytes() {
        let object = ObjectId { bytes: [5u8; 12] };
        assert_eq!(object.to_bytes(), [5u8; 12])
    }

    #[test]
    fn test_from_hex() {
        let hexstr = "0102030405060708090a0b0c";
        let object = ObjectId::from_hex(hexstr).unwrap();
        assert_eq!(object.bytes, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    }

    #[test]
    fn test_to_hex() {
        let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        let object = ObjectId::from_bytes(bytes);
        assert_eq!(object.to_hex(), "0102030405060708090a0b0c");
    }

    #[test]
    fn test_display_trait() {
        let bytes = [0xde, 0xad, 0xbe, 0xef, 0, 1, 2, 3, 4, 5, 6, 7];
        let object = ObjectId::from_bytes(bytes);
        let display = format!("{}", object);
        assert_eq!(display, "deadbeef0001020304050607");
    }

    #[test]
    fn test_timestamp() {
        let ts_bytes = 1735689600u32.to_be_bytes(); // 2025-01-01 00:00:00 UTC
        let mut bytes = [0u8; 12];
        bytes[0..4].copy_from_slice(&ts_bytes);
        let object = ObjectId::from_bytes(bytes);
        let dt = object.timestamp();
        assert_eq!(dt, Utc.ymd(2025, 1, 1).and_hms(0, 0, 0));
    }

    // ---- PROPERTY-BASED TESTS ----

    proptest! {
        #[test]
        fn prop_hex_roundtrip(bytes in any::<[u8; 12]>()) {
            let obj = ObjectId::from_bytes(bytes);
            let hex = obj.to_hex();
            let obj2 = ObjectId::from_hex(&hex).unwrap();
            prop_assert_eq!(obj2.bytes, bytes);
        }

        #[test]
        fn prop_bytes_roundtrip(hexstr in proptest::array::uniform12(0u8..=255)) {
            let obj = ObjectId::from_bytes(hexstr);
            let hex = obj.to_hex();
            let obj2 = ObjectId::from_hex(&hex).unwrap();
            prop_assert_eq!(obj2.bytes, hexstr);
        }

        #[test]
        fn prop_display_matches_to_hex(bytes in any::<[u8; 12]>()) {
            let obj = ObjectId::from_bytes(bytes);
            prop_assert_eq!(obj.to_hex(), format!("{}", obj));
        }

        #[test]
        fn prop_timestamp_conversion(ts in 0u32..=4102444800) { // up to year 2100
            let mut bytes = [0u8; 12];
            bytes[0..4].copy_from_slice(&ts.to_be_bytes());
            let obj = ObjectId::from_bytes(bytes);
            let dt = obj.timestamp();
            prop_assert_eq!(dt.timestamp() as u32, ts);
        }
    }
}
