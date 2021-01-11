pub trait Codec {
    fn decode(&mut self);
    fn encode(&mut self);
}
