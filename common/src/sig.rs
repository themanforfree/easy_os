use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct SignalFlags: i32 {
        const SIGDEF = 1;
        const SIGHUP = 1 << 1;
        const SIGINT = 1 << 2;
        const SIGQUIT = 1 << 3;
        const SIGILL = 1 << 4;
        const SIGTRAP = 1 << 5;
        const SIGABRT = 1 << 6;
        const SIGBUS = 1 << 7;
        const SIGFPE = 1 << 8;
        const SIGKILL = 1 << 9;
        const SIGUSR1 = 1 << 10;
        const SIGSEGV = 1 << 11;
        const SIGUSR2 = 1 << 12;
        const SIGPIPE = 1 << 13;
        const SIGALRM = 1 << 14;
        const SIGTERM = 1 << 15;
        const SIGSTKFLT = 1 << 16;
        const SIGCHLD = 1 << 17;
        const SIGCONT = 1 << 18;
        const SIGSTOP = 1 << 19;
        const SIGTSTP = 1 << 20;
        const SIGTTIN = 1 << 21;
        const SIGTTOU = 1 << 22;
        const SIGURG = 1 << 23;
        const SIGXCPU = 1 << 24;
        const SIGXFSZ = 1 << 25;
        const SIGVTALRM = 1 << 26;
        const SIGPROF = 1 << 27;
        const SIGWINCH = 1 << 28;
        const SIGIO = 1 << 29;
        const SIGPWR = 1 << 30;
        const SIGSYS = 1 << 31;
    }
}

impl SignalFlags {
    pub fn to_number(&self) -> i32 {
        let bits = self.bits().cast_unsigned();
        if !(bits > 0 && bits.is_power_of_two()) {
            panic!("SignalFlags must be a single bit set, got: {:#b}", bits);
        }
        bits.trailing_zeros() as i32
    }

    pub fn from_number(num: i32) -> Self {
        if !(0..=31).contains(&num) {
            panic!("Invalid signal number: {}", num);
        }
        SignalFlags::from_bits(1 << num).expect("Invalid signal flag")
    }
}

#[cfg(all(unix, test))]
mod test {
    #[test]
    fn test_signal_flags() {
        use super::SignalFlags;
        assert_eq!(SignalFlags::SIGDEF.to_number(), 0);
        assert_eq!(SignalFlags::SIGHUP.to_number(), 1);
        assert_eq!(SignalFlags::SIGINT.to_number(), 2);
        assert_eq!(SignalFlags::SIGTERM.to_number(), 15);

        assert_eq!(SignalFlags::from_number(0), SignalFlags::SIGDEF);
        assert_eq!(SignalFlags::from_number(1), SignalFlags::SIGHUP);
        assert_eq!(SignalFlags::from_number(2), SignalFlags::SIGINT);
        assert_eq!(SignalFlags::from_number(15), SignalFlags::SIGTERM);
    }
}
