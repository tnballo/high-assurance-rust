// ANCHOR: detailed_state
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StopKind {
    Mandatory, // Linux SIGSTOP
    Ignorable, // Linux SIGTSTP
}

#[derive(Debug)]
pub enum DetailedState {
    Running,
    Stopped { reason: StopKind },
    Sleeping { wake_time: u64 },
}
// ANCHOR_END: detailed_state

// ANCHOR: state
#[derive(Debug)]
pub enum State {
    Running,
    Stopped,
    Sleeping,
}
// ANCHOR_END: state

// ANCHOR: process_struct
#[derive(Debug)]
pub struct Proc<'a> {
    name: &'static str,          // Process name
    state: State,                // Current state
    children: Vec<&'a Proc<'a>>, // Children (update: now borrowed!)
}

impl<'a> Proc<'a> {
    pub fn new(name: &'static str, state: State, children: Vec<&'a Proc>) -> Self {
        Proc {
            name,
            state,
            children,
        }
    }
}
// ANCHOR_END: process_struct

// ANCHOR: drop
impl Drop for Proc<'_> {
    fn drop(&mut self) {
        println!("De-alloc-ing \'{}\' proc @ {:p}", self.name, self);
    }
}
// ANCHOR_END: drop

fn main() {
    // ANCHOR: initial_tree
    // Alloc bash
    let bash = Proc::new("bash", State::Running, Vec::new());

    // Alloc rsyslogd, 1st move: bash -> rsyslogd
    let rsyslogd = Proc::new("rsyslogd", State::Running, vec![&bash]);

    // Print owned value (new!)
    dbg!(&bash);

    // Alloc cron
    let cron = Proc::new("cron", State::Sleeping, Vec::new());

    // Alloc init, 2nd and 3rd moves: cron -> init, rsyslogd -> init
    let init = Proc::new("init", State::Running, vec![&cron, &rsyslogd]);

    // Print another owned value (new!)
    dbg!(&cron);

    // Print serialized tree to see ownership hierarchy
    dbg!(&init);
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
