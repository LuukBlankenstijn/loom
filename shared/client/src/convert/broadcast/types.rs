use uuid::Uuid;

pub struct StationState {
    pub ip: String,
    pub connected: bool,
    pub logged_in: bool,
}

pub struct StationAssignment {
    pub ip: String,
    pub seat_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum BroadcastType {
    ConnectionState = 0,
    AssignmentState = 1,
}

pub enum BroadcastEvent {
    ConnectionState(Vec<StationState>),
    AssignmentState(Vec<StationAssignment>),
}
