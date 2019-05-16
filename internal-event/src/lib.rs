#![forbid(unsafe_code)]

#[derive(Debug)]
pub enum Event {
    DirectCommand(DirectCommand),
    TimerTick,
}

#[derive(Debug)]
pub enum DirectCommand {
    NewChannel,
}
