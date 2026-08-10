use core::fmt;

#[derive(Clone, Copy)]
pub struct AssertFail {
    pub message: &'static str,
}

impl AssertFail {
    pub const fn msg(message: &'static str) -> Self {
        Self { message }
    }
}

impl fmt::Display for AssertFail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

pub type TestResult = core::result::Result<(), AssertFail>;

#[macro_export]
macro_rules! check {
    ($cond:expr, $msg:literal) => {
        if !$cond {
            return Err($crate::harness::AssertFail::msg($msg));
        }
    };
}

#[macro_export]
macro_rules! check_eq {
    ($left:expr, $right:expr, $msg:literal) => {{
        let l = $left;
        let r = $right;
        if l != r {
            return Err($crate::harness::AssertFail::msg($msg));
        }
    }};
}

#[macro_export]
macro_rules! check_ok {
    ($expr:expr, $msg:literal) => {
        match $expr {
            Ok(v) => v,
            Err(_) => return Err($crate::harness::AssertFail::msg($msg)),
        }
    };
}

#[macro_export]
macro_rules! check_err {
    ($expr:expr, $errno:expr, $msg:literal) => {
        match $expr {
            Err(e) if e == $errno => {}
            Ok(_) => return Err($crate::harness::AssertFail::msg($msg)),
            Err(_) => return Err($crate::harness::AssertFail::msg($msg)),
        }
    };
}
