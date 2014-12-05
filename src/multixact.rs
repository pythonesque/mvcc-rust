use trans::{
    TransactionId,
};

pub const INVALID_MULTI_XACT_ID: u32 = 0;
pub const FIRST_MULTI_XACT_ID: u32 = 1;
pub const MAX_MULTI_XACT_ID: u32 = 0xFFFFFFFF;

#[deriving(Clone,Eq,PartialEq,Show)]
#[repr(C)]
pub struct ValidMultiXactId(u32);

#[deriving(Clone)]
#[repr(C)]
pub struct MultiXactId(u32);

impl ValidMultiXactId {
    #[inline]
    pub fn unwrap(&self) -> MultiXactId {
        MultiXactId(self.0)
    }
}

impl MultiXactId {
    #[inline]
    pub fn new(multi_xact_id: u32) -> Option<ValidMultiXactId> {
        MultiXactId(multi_xact_id).to_valid()
    }

    #[inline]
    pub fn to_valid(&self) -> Option<ValidMultiXactId> {
        match self.0 {
            INVALID_MULTI_XACT_ID => None,
            id => Some(ValidMultiXactId(id)),
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0 != INVALID_MULTI_XACT_ID
    }
}

#[deriving(Clone)]
#[repr(C)]
pub struct MultiXactOffset(u32);

pub const MAX_MULTI_XACT_OFFSET: u32 = 0xFFFFFFFF;

/* Number of SLRU buffers to use for multixact */
pub const NUM_MXACTOFFSET_BUFFERS: uint = 8u;
pub const NUM_MXACTMEMBER_BUFFERS: uint = 16u;

#[deriving(Clone,Eq,Ord,PartialEq,PartialOrd,Show)]
#[repr(C)]
pub enum MultiXactStatus {
    ForKeyShare = 0,
    ForShare = 0x01,
    ForNoKeyUpdate = 0x02,
    ForUpdate = 0x03,
    /* an update that doesn't touch "key" columns */
    NoKeyUpdate = 0x04,/* other updates, and delete */
    /* other updates, and delete */
    Update = 0x05,
}

pub const MAX_MULTI_XACT_STATUS: uint = MultiXactStatus::Update as uint;


impl MultiXactStatus {
    #[inline]
    pub fn is_update(&self) -> bool {
        *self > MultiXactStatus::ForUpdate
    }
}

pub struct MultiXactMember {
    xid: TransactionId,
    status: MultiXactStatus,
}

#[cfg(test)]
mod tests {
    use std::mem;
    use super::{
        MultiXactId,
    };
    use trans::TransactionId;

    #[test]
    fn multi_xact_id_same_size_as_transaction_id() {
        assert_eq!(mem::size_of::<MultiXactId>(), mem::size_of::<TransactionId>());
    }
}
