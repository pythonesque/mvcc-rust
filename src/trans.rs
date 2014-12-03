use self::SpecialTransactionId::*;

use std::mem;

pub const INVALID_TRANSACTION_ID: u32 = 0;
pub const BOOTSTRAP_TRANSACTION_ID: u32 = 1;
pub const FROZEN_TRANSACTION_ID: u32 = 2;
pub const FIRST_NORMAL_TRANSACTION_ID: u32 = 3;
pub const MAX_TRANSACTION_ID: u32 = 0xFFFFFFFF;

#[deriving(Clone,Eq,PartialEq,Show)]
#[repr(C)]
pub struct ValidTransactionId(u32);

impl ValidTransactionId {
    #[inline]
    pub fn unwrap(&self) -> TransactionId {
        TransactionId(self.0)
    }

    #[inline]
    pub fn to_normal(&self) -> Result<NormalTransactionId, SpecialTransactionId> {
        debug_assert!(self.0 > 0) // Should never be INVALID_TRANSACTION_ID
        match self.0 {
            BOOTSTRAP_TRANSACTION_ID => Err(Bootstrap),
            FROZEN_TRANSACTION_ID => Err(Frozen),
            id => Ok(NormalTransactionId(id)),
        }
    }
}

#[deriving(Clone,Eq,Ord,PartialEq,PartialOrd,Show)]
#[repr(C)]
pub struct NormalTransactionId(u32);

impl NormalTransactionId {
    #[inline]
    pub fn unwrap(&self) -> TransactionId {
        TransactionId(self.0)
    }

    #[inline]
    pub fn to_valid(&self) -> ValidTransactionId {
        ValidTransactionId(self.0)
    }
}

#[deriving(Clone,Eq,PartialEq,Show)]
#[repr(u32)]
pub enum SpecialTransactionId {
    Bootstrap = BOOTSTRAP_TRANSACTION_ID,
    Frozen = FROZEN_TRANSACTION_ID,
}

impl SpecialTransactionId {
    pub fn to_valid(&self) -> ValidTransactionId {
        ValidTransactionId(unsafe { mem::transmute(*self) })
    }
}

pub type ValidTransactionIdResult = Result<NormalTransactionId, SpecialTransactionId>;

pub type TransactionIdResult = Result<NormalTransactionId, Option<SpecialTransactionId>>;

#[deriving(Clone,Eq,PartialEq,Show)]
#[repr(C)]
pub struct TransactionId(u32);

impl TransactionId {
    #[inline]
    pub fn new(transaction_id: u32) -> TransactionIdResult {
        TransactionId(transaction_id).to_normal()
    }

    #[inline]
    pub fn to_valid(&self) -> Option<ValidTransactionId> {
        match self.0 {
            INVALID_TRANSACTION_ID => None,
            id => Some(ValidTransactionId(id)),
        }
    }

    #[inline]
    pub fn to_normal(&self) -> TransactionIdResult {
        match self.0 {
            INVALID_TRANSACTION_ID => Err(None),
            BOOTSTRAP_TRANSACTION_ID => Err(Some(Bootstrap)),
            FROZEN_TRANSACTION_ID => Err(Some(Frozen)),
            id => Ok(NormalTransactionId(id)),
        }
    }

    #[inline]
    pub fn is_valid(&self) -> bool {
        self.0 != INVALID_TRANSACTION_ID
    }

    #[inline]
    pub fn is_normal(&self) -> bool {
        self.0 >= FIRST_NORMAL_TRANSACTION_ID
    }

    #[inline]
    pub fn store(&mut self, xid: ValidTransactionId) {
        *self = xid.unwrap();
    }

    #[inline]
    pub fn invalidate(&mut self) {
        self.0 = INVALID_TRANSACTION_ID;
    }

    #[inline]
    pub fn advance(&mut self) {
        self.0 = match self.0 + 1 {
            INVALID_TRANSACTION_ID | BOOTSTRAP_TRANSACTION_ID | FROZEN_TRANSACTION_ID => FIRST_NORMAL_TRANSACTION_ID,
            id => id
        }
    }

    #[inline]
    pub fn retreat(&mut self) {
        loop {
            self.0 -= 1;
            if self.0 >= FIRST_NORMAL_TRANSACTION_ID { break }
        }
    }
}
