/// Convert a domain type into its proto representation.
pub trait IntoProto<P> {
    fn into_proto(self) -> P;
}

/// Convert a proto type into its domain representation (infallible).
pub trait FromProto<P> {
    fn from_proto(proto: P) -> Self;
}

/// Convert a proto type into its domain representation (fallible).
pub trait TryIntoCore<C>: Sized {
    type Error;
    fn try_into_core(self) -> Result<C, Self::Error>;
}

#[cfg(feature = "admin-v1")]
pub mod admin;

#[cfg(feature = "station-v1")]
pub mod station;

#[cfg(feature = "map-v1")]
pub mod map;
#[cfg(feature = "map-v1")]
pub use loom_core::map::{MapElement, MapMetadata, Point, Rotation};
#[cfg(feature = "map-v1")]
pub use loom_core::map::{door::Door, seat::Seat, wall::Wall};

#[cfg(feature = "broadcast-v1")]
pub mod broadcast;
#[cfg(feature = "broadcast-v1")]
pub use loom_core::event::broadcast::{BroadcastEvent, BroadcastType, StationAssignment, StationConnectionState};

#[cfg(feature = "admin-v1")]
pub(crate) fn to_timestamp(dt: chrono::DateTime<chrono::Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as i32,
    }
}
