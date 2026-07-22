#![no_main]

use libfuzzer_sys::fuzz_target;
use prost::Message;

fuzz_target!(|data: &[u8]| {
    // Try to decode as a protobuf Notebook
    if let Ok(proto) = onb_kernel::proto::Notebook::decode(data) {
        // If it decodes, try to convert to internal format
        let _ = onb_kernel::notebook::format::notebook_from_proto(proto);
    }
});
