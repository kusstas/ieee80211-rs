#![cfg(feature = "crypto")]

use ieee80211::{
    crypto::MicState,
    data_frame::{
        DataFrame, DataFrameReadPayload, PotentiallyWrappedPayload, builder::DataFrameBuilder,
    },
};
use mac_parser::MACAddress;
use scroll::{Pread, Pwrite, ctx::MeasureWith};

const OUR_MAC_ADDRESS: MACAddress = MACAddress::new([0x00, 0x20, 0x91, 0x13, 0x37, 0x00]);
const AP_MAC_ADDRESS: MACAddress = MACAddress::new([0x00, 0x20, 0x91, 0x13, 0x37, 0x01]);

const EXPECTED_DATA_FRAME: DataFrame<'_, &[u8]> = DataFrameBuilder::new()
    .to_ds()
    .category_data()
    .payload::<&[u8]>(&[0x13, 0x37])
    .destination_address(AP_MAC_ADDRESS)
    .source_address(OUR_MAC_ADDRESS)
    .bssid(AP_MAC_ADDRESS)
    .build();
const EXPECTED_BYTES: &[u8] = &[
    0x08, 0x01, 0x00, 0x00, 0x00, 0x20, 0x91, 0x13, 0x37, 0x01, 0x00, 0x20, 0x91, 0x13, 0x37, 0x00,
    0x00, 0x20, 0x91, 0x13, 0x37, 0x01, 0x00, 0x00, 0x13, 0x37,
];

#[test]
fn test_data_frame_rw() {
    let read = EXPECTED_BYTES.pread_with::<DataFrame>(0, false).unwrap();
    assert_eq!(read.header, EXPECTED_DATA_FRAME.header);
    let Some(PotentiallyWrappedPayload::Unwrapped(DataFrameReadPayload::Single(payload))) =
        read.potentially_wrapped_payload(None)
    else {
        panic!("Data frame payload wasn't single.");
    };
    assert_eq!(payload, EXPECTED_DATA_FRAME.payload.unwrap());

    let mut buf = vec![0x00u8; EXPECTED_DATA_FRAME.measure_with(&false)];
    buf.pwrite(EXPECTED_DATA_FRAME, 0).unwrap();
    assert_eq!(buf, EXPECTED_BYTES);
}

const ENCRYPTED_DATA_FRAME_BYTES: &[u8] = &[
    0x88, 0x41, 0x30, 0x00, 0x7c, 0x5a, 0x1c, 0x15, 0xcf, 0xa7, 0x74, 0xd8, 0x3e, 0x9e, 0xff, 0x57,
    0x84, 0x39, 0x8f, 0xe2, 0x57, 0x39, 0xb0, 0x03, 0x00, 0x00, 0x3a, 0x00, 0x00, 0x20, 0x00, 0x00,
    0x00, 0x00, 0x81, 0x5b, 0x20, 0xb6, 0x32, 0x4d, 0xfb, 0x2a, 0x3d, 0xd6, 0x65, 0x00, 0x78, 0x1c,
    0xa3, 0xf7, 0x64, 0x1c, 0xec, 0x21, 0xdc, 0x05, 0xa7, 0xd5, 0x92, 0x47, 0x50, 0xe6, 0x83, 0x46,
    0x3a, 0x48, 0x82, 0x55, 0x46, 0xff, 0x33, 0x19, 0x01, 0xa0,
];
#[test]
fn test_encrypted_data_frame() {
    let read = ENCRYPTED_DATA_FRAME_BYTES
        .pread_with::<DataFrame>(0, false)
        .unwrap();
    let Some(PotentiallyWrappedPayload::CryptoWrapped(crypto_wrapper)) =
        read.potentially_wrapped_payload(Some(MicState::NotPresent))
    else {
        panic!();
    };
    assert_eq!(crypto_wrapper.crypto_header.packet_number(), 0x3a);
    assert_eq!(crypto_wrapper.crypto_header.key_id(), 0);
}
