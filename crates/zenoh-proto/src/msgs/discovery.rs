use crate::*;
use crate::msgs::InitIdentifier;

/// SCOUT message — discover peers/routers on the network.
#[derive(ZStruct, Debug, PartialEq, Default)]
#[zenoh(header = "Z|_:1|S|ID:5=0x00")]
pub struct Scout {
    #[zenoh(presence = header(S), default = 0)]
    pub what: u8,
}

/// HELLO message — response to SCOUT with identity and locators.
#[derive(ZStruct, Debug, PartialEq, Default)]
#[zenoh(header = "Z|_:1|L|ID:5=0x02")]
pub struct Hello<'a> {
    pub version: u8,
    pub identifier: InitIdentifier,

    #[zenoh(presence = header(L), size = prefixed)]
    pub locators: Option<&'a str>,
}
