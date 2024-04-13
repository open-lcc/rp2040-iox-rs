pub(crate) mod binary_output;
pub(crate) mod analog_output;
pub(crate) mod analog_input;

pub(crate) trait Flushable {
    async fn flush(&mut self);
}