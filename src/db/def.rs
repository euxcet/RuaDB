pub enum Data {
    IntData(i32),
    FloatData(f32),
    StringData(String),
}

pub struct Payload {
    key: String,
    data: Data,
}
