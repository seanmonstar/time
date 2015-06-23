
pub use self::inner::*;

#[cfg(unix)]
mod inner {
    use libc::{c_int, c_long, c_char, time_t};
    use ::{Tm, Timespec, Local, Utc};

    /// ctime's `tm`
    #[repr(C)]
    struct tm {
        tm_sec: c_int,
        tm_min: c_int,
        tm_hour: c_int,
        tm_mday: c_int,
        tm_mon: c_int,
        tm_year: c_int,
        tm_wday: c_int,
        tm_yday: c_int,
        tm_isdst: c_int,
        tm_gmtoff: c_long,
        tm_zone: *const c_char,
    }

    impl Default for tm {
        fn default() -> tm {
            tm {
                tm_sec: 0,
                tm_min: 0,
                tm_hour: 0,
                tm_mday: 0,
                tm_mon: 0,
                tm_year: 0,
                tm_wday: 0,
                tm_yday: 0,
                tm_isdst: 0,
                tm_gmtoff: 0,
                tm_zone: 0 as *const c_char
            }
        }
    }

    impl<'a, TZ> From<&'a Tm<TZ>> for tm {
        fn from(tm: &'a Tm<TZ>) -> tm {
            tm {
                tm_sec: tm.tm_sec,
                tm_min: tm.tm_min,
                tm_hour: tm.tm_hour,
                tm_mday: tm.tm_mday,
                tm_mon: tm.tm_mon,
                tm_year: tm.tm_year,
                tm_wday: tm.tm_wday,
                tm_yday: tm.tm_yday,
                tm_isdst: tm.tm_isdst,
                tm_gmtoff: tm.tm_utcoff as c_long,
                .. tm::default()
            }
        }
    }

    fn tm_to_rust_tm<TZ>(tm: &tm, rust_tm: &mut Tm<TZ>) {
        rust_tm.tm_sec = tm.tm_sec;
        rust_tm.tm_min = tm.tm_min;
        rust_tm.tm_hour = tm.tm_hour;
        rust_tm.tm_mday = tm.tm_mday;
        rust_tm.tm_mon = tm.tm_mon;
        rust_tm.tm_year = tm.tm_year;
        rust_tm.tm_wday = tm.tm_wday;
        rust_tm.tm_yday = tm.tm_yday;
        rust_tm.tm_isdst = tm.tm_isdst;
    }

    extern {
        fn gmtime_r(time_p: *const time_t, result: *mut tm) -> *mut tm;
        fn localtime_r(time_p: *const time_t, result: *mut tm) -> *mut tm;
        fn timegm(tm: *const tm) -> time_t;
        fn mktime(tm: *const tm) -> time_t;
    }

    impl From<Timespec> for Tm<Utc> {
        fn from(spec: Timespec) -> Tm<Utc> {
            let mut out = tm::default();
            let mut tm = Tm::default();
            unsafe {
                gmtime_r(&spec.sec, &mut out);
            }
            tm_to_rust_tm(&out, &mut tm);
            tm.tm_utcoff = 0;
            tm.tm_nsec = spec.nsec;
            tm
        }
    }

    impl From<Timespec> for Tm<Local> {
        fn from(spec: Timespec) -> Tm<Local> {
            let mut out = tm::default();
            let mut tm = Tm::default();
            unsafe {
                localtime_r(&spec.sec, &mut out);
            }
            tm_to_rust_tm(&out, &mut tm);
            tm.tm_utcoff = out.tm_gmtoff as i32;
            tm.tm_nsec = spec.nsec;
            tm
        }
    }

    impl From<Tm<Utc>> for Timespec {
        fn from(tm: Tm<Utc>) -> Timespec {
             Timespec {
                 sec: unsafe {
                    timegm(&tm::from(&tm))
                 },
                nsec: tm.tm_nsec
             }
        }
    }

    impl From<Tm<Local>> for Timespec {
        fn from(tm: Tm<Local>) -> Timespec {
             Timespec {
                 sec: unsafe {
                    mktime(&tm::from(&tm))
                 },
                nsec: tm.tm_nsec
             }
        }
    }
}

#[cfg(windows)]
mod inner {
    use libc::{WORD, DWORD, LONG};

    #[repr(C)]
    #[derive(Default)]
    struct SystemTime {
        wYear: WORD,
        wMonth: WORD,
        wDayOfWeek: WORD,
        wDay: WORD,
        wHour: WORD,
        wMinute: WORD,
        wSecond: WORD,
        wMilliseconds: WORD,
    }

    #[repr(C)]
    struct FileTime {
        dwLowDateTime: DWORD,
        dwHighDateTime: DWORD,
    }

    const WINDOWS_TICK: i64 = 10_000_000;
    const SEC_TO_UNIX_EPOCH: i64 = 11_644_473_600;

    impl From<i64> for FileTime {
        fn from(sec: i64) -> FileTime {
            let t = (sec + SEC_TO_UNIX_EPOCH) * WINDOWS_TICK;
            FileTime {
                dwLowDateTime: t as DWORD,
                dwHighDateTime: (t >> 32) as DWORD
            }
        }
    }

    impl Into<i64> for FileTime {
        fn into(self) -> i64 {
            let t = ((self.dwHighDateTime as i64) << 32) | (self.dwLowDateTime as i64);
            t / WINDOWS_TICK - SEC_TO_UNIX_EPOCH
        }
    }

    impl<'a> From<&'a Tm> for SystemTime {
        fn from(tm: &'a Tm) -> SystemTime {
            let mut sys = SystemTime::default();
            sys.wSecond = tm.tm_sec;
            sys.wMinute = tm.tm_min;
            sys.wHour = tm.tm_hour;
            sys.wDay = tm.tm_mday;
            sys.wDayOfWeek = tm.tm_wday;
            sys.wMonth = tm.tm_mon + 1;
            sys.wYear = tm.tm_year + 1900;
            sys
        }
    }

    fn system_time_to_tm(sys: &SystemTime, tm: &mut Tm) {
        tm.tm_sec = sys.wSecond;
        tm.tm_min = sys.wMinute;
        tm.tm_hour = sys.wHour;
        tm.tm_mday = sys.wDay;
        tm.tm_wday = sys.wDayOfWeek;
        tm.tm_mon = sys.wMonth - 1;
        tm.tm_year = sys.wYear - 1900;
    }

    #[repr(C)]
    struct TimeZoneInfo {
        Bias: LONG,
        StandardName: [WCHAR; 32],
        StandardDate: SystemTime,
        StandardBias: LONG,
        DaylightName: [WCHAR; 32],
        DaylightDate: SystemTime,
        DaylightBias: LONG,
    }

    extern "system" {
        fn GetSystemTime(out: *mut SystemTime);
        fn FileTimeToLocalFileTime(in_: *const FileTime, out: *mut FileTime) -> bool;
        fn FileTimeToSystemTime(ft: *const FileTime, out: *mut SystemTime) -> bool;
        fn SystemTimeToFileTime(sys: *const SystemTime, ft: *mut FileTime) -> bool;
        fn SystemTimeToTzSpecificLocalTime(tz: *const TimeZoneInfo, utc: *const SystemTime, local: *mut SystemTime) -> bool;
    }
 
    pub fn gmtime(sec: i64, tm: &mut Tm) {
        let mut out = SystemTime::default();
        unsafe {
            FileTimeToSystemTime(&sec.into(), &mut out);
        }
        system_time_to_tm(&out, tm);
        tm.tm_utcoff = 0;
    }

    pub fn localtime(sec: i64, tm: &mut Tm) {
        let mut out = SystemTime::default();
        let mut tz = TimeZoneInfo::default();
        unsafe {
            GetTimeZoneInfo(&mut tz);
            FileTimeToSystemTime(&sec.into(), &mut out);
            SystemTimeToTzSpecificLocalTime(&tz, &out, &mut out);
        }
        system_time_to_tm(&out, tm);
        tm.tm_utcoff = -tz.Bias * 60;
    }

    pub fn timegm_(tm: &Tm) -> i64 {
        let mut ft = FileTime::default();
        unsafe {
            SystemTimeToFileTime(&SystemTime::from(tm), &mut ft);
        }
        ft.into()
    }

    pub fn mktime_(tm: &Tm) -> i64 {
        let mut ft = FileTime::default();
        unsafe {
            SystemTimeToFileTime(&SystemTime::from(tm), &mut ft);
            FileTimeToLocalFileTime(&ft, &mut ft);
        }
        ft.into()
    }
}


