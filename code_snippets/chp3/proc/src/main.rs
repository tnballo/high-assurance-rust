// ANCHOR: detailed_state
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StopKind {
    Mandatory, // Linux SIGSTOP
    Ignorable, // Linux SIGTSTP
}

pub enum DetailedState {
    Running,
    Stopped { reason: StopKind },
    Sleeping { start_time: u64 },
}
// ANCHOR_END: detailed_state

// ANCHOR: process_struct
#[derive(Debug)]
// ANCHOR: state
pub enum State {
    Running,
    Stopped,
    Sleeping,
}
// ANCHOR_END: state

#[derive(Debug)]
pub struct Proc {
    name: &'static str,  // Process name (update: nicer print than u32 pid)
    state: State,        // Current state
    children: Vec<Proc>, // Children (update: now owned!)
}

impl Proc {
    pub fn new(name: &'static str, state: State, children: Vec<Proc>) -> Self {
        Proc {
            name,
            state,
            children,
        }
    }
}
// ANCHOR_END: process_struct

// ANCHOR: drop
impl Drop for Proc {
    fn drop(&mut self) {
        println!("De-alloc-ing \'{}\' Proc @ {:p}", self.name, self);
    }
}
// ANCHOR_END: drop

fn main() {
    // ANCHOR: initial_tree
    // Build process tree using 3 "moves" (more info soon):
    //
    // init
    //  |- cron
    //  |- rsyslogd
    //      |- bash
    //
    // Run "pstree -n -g" (in container) to see your OS's real process tree!

    // Alloc bash
    let bash = Proc::new("bash", State::Running, Vec::new());

    // Alloc rsyslogd, 1st move: bash -> rsyslogd
    let rsyslogd = Proc::new("rsyslogd", State::Running, vec![bash]);

    // Alloc cron
    let cron = Proc::new("cron", State::Sleeping, Vec::new());

    // Alloc init, 2nd and 3rd moves: cron -> init, rsyslogd -> init
    let init = Proc::new("init", State::Running, vec![cron, rsyslogd]);

    // Print serialized tree to see ownership hierarchy
    dbg!(init);
    // ANCHOR_END: initial_tree
}

#[cfg(test)]
mod tests {
    use super::{DetailedState, Proc, State, StopKind};

    #[test]
    fn test_size() {
        assert_eq!(core::mem::size_of::<Proc>(), 48);
    }

    #[test]
    fn test_stop_match() {
        let s = State::Stopped;
        match s {
            State::Running => unreachable!(),
            State::Stopped => {}
            State::Sleeping => unreachable!(),
        }
    }

    #[test]
    fn test_detailed_stop_match() {
        let s = DetailedState::Stopped {
            reason: StopKind::Mandatory,
        };
        match s {
            DetailedState::Stopped { reason } => {
                assert_eq!(reason, StopKind::Mandatory);
            }
            _ => unreachable!(),
        }
    }
}
