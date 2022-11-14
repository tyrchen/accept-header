use snafu::Snafu;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    #[snafu(display("Invalid media type: {value}"))]
    MediaType {
        value: String,
        source: mime::FromStrError,
    },
    #[snafu(display("Invalid weight: {value}"))]
    ParseWeight {
        value: String,
        source: std::num::ParseFloatError,
    },
    #[snafu(display("Weight should be 0.0-1.0. Got {value}"))]
    WeightRange { value: f32 },
}
