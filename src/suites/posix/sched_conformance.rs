//! Scheduling conformance deepeners (TPS).

use crate::check;
use crate::check_eq;
use crate::check_ok;
use crate::harness::TestResult;
use crate::syscall::{self, Errno, PRIO_PROCESS, SCHED_OTHER};

#[crate::lctp_test(suite = posix)]
fn schedc_yield_1() -> TestResult {
    for _ in 0..1 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_2() -> TestResult {
    for _ in 0..2 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_3() -> TestResult {
    for _ in 0..3 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_4() -> TestResult {
    for _ in 0..4 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_5() -> TestResult {
    for _ in 0..5 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_6() -> TestResult {
    for _ in 0..6 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_7() -> TestResult {
    for _ in 0..7 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_8() -> TestResult {
    for _ in 0..8 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_9() -> TestResult {
    for _ in 0..9 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_10() -> TestResult {
    for _ in 0..10 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_11() -> TestResult {
    for _ in 0..11 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_12() -> TestResult {
    for _ in 0..12 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_13() -> TestResult {
    for _ in 0..13 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_14() -> TestResult {
    for _ in 0..14 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_15() -> TestResult {
    for _ in 0..15 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_16() -> TestResult {
    for _ in 0..16 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_17() -> TestResult {
    for _ in 0..17 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_18() -> TestResult {
    for _ in 0..18 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_19() -> TestResult {
    for _ in 0..19 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_20() -> TestResult {
    for _ in 0..20 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_21() -> TestResult {
    for _ in 0..21 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_22() -> TestResult {
    for _ in 0..22 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_23() -> TestResult {
    for _ in 0..23 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_24() -> TestResult {
    for _ in 0..24 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_25() -> TestResult {
    for _ in 0..25 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_26() -> TestResult {
    for _ in 0..26 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_27() -> TestResult {
    for _ in 0..27 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_28() -> TestResult {
    for _ in 0..28 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_29() -> TestResult {
    for _ in 0..29 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_30() -> TestResult {
    for _ in 0..30 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_31() -> TestResult {
    for _ in 0..31 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_32() -> TestResult {
    for _ in 0..32 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_33() -> TestResult {
    for _ in 0..33 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_34() -> TestResult {
    for _ in 0..34 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_35() -> TestResult {
    for _ in 0..35 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_36() -> TestResult {
    for _ in 0..36 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_37() -> TestResult {
    for _ in 0..37 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_38() -> TestResult {
    for _ in 0..38 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_39() -> TestResult {
    for _ in 0..39 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_yield_40() -> TestResult {
    for _ in 0..40 {
        check_ok!(syscall::sched_yield(), "yield");
    }
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_1() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_2() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_3() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_4() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_5() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_6() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_7() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_8() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_9() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_10() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_11() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_12() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_13() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_14() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_15() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_16() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_17() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_18() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_19() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getaffinity_20() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "aff");
    check!(mask.iter().any(|&b| b != 0), "cpu");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_1() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_2() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_3() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_4() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_5() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_6() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_7() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_8() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_9() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_10() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_11() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_12() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_13() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_14() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_15() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_16() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_17() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_18() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_19() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getscheduler_20() -> TestResult {
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    check_eq!(pol, SCHED_OTHER, "other");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_1() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_2() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_3() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_4() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_5() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_6() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_7() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_8() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_9() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_10() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_11() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_12() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_13() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_14() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_15() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_16() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_17() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_18() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_19() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_getpriority_20() -> TestResult {
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "prio");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_1() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_2() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_3() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_4() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_5() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_6() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_7() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_8() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_9() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_10() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_11() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_combo_12() -> TestResult {
    check_ok!(syscall::sched_yield(), "y");
    let pol = check_ok!(syscall::sched_getscheduler(0), "pol");
    let p = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "p");
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "a");
    check_eq!(pol, SCHED_OTHER, "other");
    check!(p >= -20 && p <= 19, "nice");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_setpriority_same_1() -> TestResult {
    let cur = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "cur");
    match syscall::setpriority(PRIO_PROCESS, 0, cur) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setprio")); }
    }
    let after = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "after");
    check_eq!(after, cur, "stable");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_setpriority_same_2() -> TestResult {
    let cur = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "cur");
    match syscall::setpriority(PRIO_PROCESS, 0, cur) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setprio")); }
    }
    let after = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "after");
    check_eq!(after, cur, "stable");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_setpriority_same_3() -> TestResult {
    let cur = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "cur");
    match syscall::setpriority(PRIO_PROCESS, 0, cur) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setprio")); }
    }
    let after = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "after");
    check_eq!(after, cur, "stable");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_setpriority_same_4() -> TestResult {
    let cur = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "cur");
    match syscall::setpriority(PRIO_PROCESS, 0, cur) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setprio")); }
    }
    let after = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "after");
    check_eq!(after, cur, "stable");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_setpriority_same_5() -> TestResult {
    let cur = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "cur");
    match syscall::setpriority(PRIO_PROCESS, 0, cur) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setprio")); }
    }
    let after = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "after");
    check_eq!(after, cur, "stable");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_setpriority_same_6() -> TestResult {
    let cur = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "cur");
    match syscall::setpriority(PRIO_PROCESS, 0, cur) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setprio")); }
    }
    let after = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "after");
    check_eq!(after, cur, "stable");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_setpriority_same_7() -> TestResult {
    let cur = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "cur");
    match syscall::setpriority(PRIO_PROCESS, 0, cur) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setprio")); }
    }
    let after = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "after");
    check_eq!(after, cur, "stable");
    Ok(())
}
#[crate::lctp_test(suite = posix)]
fn schedc_setpriority_same_8() -> TestResult {
    let cur = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "cur");
    match syscall::setpriority(PRIO_PROCESS, 0, cur) {
        Ok(()) => {}
        Err(Errno::EPERM) | Err(Errno::EACCES) | Err(Errno::EINVAL) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setprio")); }
    }
    let after = check_ok!(syscall::getpriority(PRIO_PROCESS, 0), "after");
    check_eq!(after, cur, "stable");
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_affinity_roundtrip_1() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setaff")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_affinity_roundtrip_2() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setaff")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_affinity_roundtrip_3() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setaff")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_affinity_roundtrip_4() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setaff")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_affinity_roundtrip_5() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setaff")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_affinity_roundtrip_6() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setaff")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_affinity_roundtrip_7() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setaff")); }
    }
    Ok(())
}
#[crate::lctp_test(suite = posix, full)]
fn schedc_affinity_roundtrip_8() -> TestResult {
    let mut mask = [0u8; 64];
    check_ok!(syscall::sched_getaffinity(0, &mut mask), "get");
    match syscall::sched_setaffinity(0, &mask) {
        Ok(()) => {}
        Err(Errno::EINVAL) | Err(Errno::EPERM) | Err(Errno::ESRCH) => {}
        Err(_) => { return Err(crate::harness::AssertFail::msg("setaff")); }
    }
    Ok(())
}