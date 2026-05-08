use crate::{exts::*, fields::ZenohIdProto, msgs::*};

macro_rules! roundtrip {
    ($ty:ty) => {{
        let mut rand = [0u8; MAX_PAYLOAD_SIZE];
        let mut data = [0u8; MAX_PAYLOAD_SIZE];

        for _ in 0..NUM_ITER {
            let value = <$ty>::rand(&mut &mut rand[..]);

            let len = $crate::ZLen::z_len(&value);
            $crate::ZEncode::z_encode(&value, &mut &mut data[..]).unwrap();

            let ret = <$ty as $crate::ZDecode>::z_decode(&mut &data[..len]).unwrap();

            assert_eq!(ret, value);
        }
    }};

    (ext, $ty:ty) => {{
        let mut rand = [0u8; MAX_PAYLOAD_SIZE];
        let mut data = [0u8; MAX_PAYLOAD_SIZE];

        for _ in 0..NUM_ITER {
            let value = <$ty>::rand(&mut &mut rand[..]);

            $crate::zext_encode::<_, 0x1, true>(&value, &mut &mut data[..], false).unwrap();

            let ret = $crate::zext_decode::<$ty>(&mut &data[..]).unwrap();

            assert_eq!(ret, value);
        }
    }};
}

macro_rules! roundtrips {
    (ext, $namespace:ident, $($ty:ty),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<$namespace _proto_ext_ $ty:lower>]() {
                    roundtrip!(ext, $ty);
                }
            }
        )*
    };

    ($namespace:ident, $($ty:ty),* $(,)?) => {
        $(
            paste::paste! {
                #[test]
                fn [<$namespace _proto_ $ty:lower>]() {
                    roundtrip!($ty);
                }
            }
        )*
    };
}

const NUM_ITER: usize = 100;
const MAX_PAYLOAD_SIZE: usize = 512;

roundtrips!(ext, zenoh, EntityGlobalId, SourceInfo, Value, Attachment);
roundtrips!(zenoh, Err, Put, Query, Reply,);

roundtrips!(
    ext,
    network,
    QoS,
    NodeId,
    QueryTarget,
    Budget,
    QueryableInfo
);

roundtrips!(
    network,
    DeclareKeyExpr,
    UndeclareKeyExpr,
    DeclareSubscriber,
    UndeclareSubscriber,
    DeclareQueryable,
    UndeclareQueryable,
    DeclareToken,
    UndeclareToken,
    DeclareFinal,
    Declare,
    Interest,
    InterestFinal,
    Push,
    Request,
    Response,
    ResponseFinal,
);

roundtrips!(ext, transport, Auth, Patch);
roundtrips!(
    transport,
    Close,
    FrameHeader,
    InitSyn,
    InitAck,
    KeepAlive,
    OpenSyn,
    OpenAck
);

#[test]
fn scout_roundtrip() {
    let msg = Scout { what: 0x02 };
    let mut buf = [0u8; 256];
    let mut w = &mut buf[..];
    <Scout as crate::ZEncode>::z_encode(&msg, &mut w).unwrap();
    let len = <Scout as crate::ZLen>::z_len(&msg);
    let decoded = <Scout as crate::ZDecode>::z_decode(&mut &buf[..len]).unwrap();
    assert_eq!(msg, decoded);

    let msg = Scout { what: 0x01 };
    let mut buf = [0u8; 256];
    let mut w = &mut buf[..];
    <Scout as crate::ZEncode>::z_encode(&msg, &mut w).unwrap();
    let len = <Scout as crate::ZLen>::z_len(&msg);
    let decoded = <Scout as crate::ZDecode>::z_decode(&mut &buf[..len]).unwrap();
    assert_eq!(msg, decoded);
}

#[test]
fn hello_roundtrip() {
    let msg = Hello {
        version: crate::VERSION,
        identifier: InitIdentifier { zid: ZenohIdProto::default(), whatami: crate::fields::WhatAmI::Peer },
        locators: None,
    };
    let mut buf = [0u8; 256];
    let mut w = &mut buf[..];
    <Hello<'_> as crate::ZEncode>::z_encode(&msg, &mut w).unwrap();
    let len = <Hello<'_> as crate::ZLen>::z_len(&msg);
    let decoded = <Hello<'_> as crate::ZDecode>::z_decode(&mut &buf[..len]).unwrap();
    assert_eq!(msg, decoded);

    let msg = Hello {
        version: crate::VERSION,
        identifier: InitIdentifier { zid: ZenohIdProto::default(), whatami: crate::fields::WhatAmI::Client },
        locators: Some("tcp/127.0.0.1:7447"),
    };
    let mut buf = [0u8; 256];
    let mut w = &mut buf[..];
    <Hello<'_> as crate::ZEncode>::z_encode(&msg, &mut w).unwrap();
    let len = <Hello<'_> as crate::ZLen>::z_len(&msg);
    let decoded = <Hello<'_> as crate::ZDecode>::z_decode(&mut &buf[..len]).unwrap();
    assert_eq!(msg, decoded);
}
