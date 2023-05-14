use opencv::dnn;

pub struct Surveillance {
    net: dnn::Net,
}

impl Surveillance {
    pub fn new() -> Self {
        let bytes = opencv::core::Vector::from_slice(include_bytes!("../model.onnx"));

        Self {
            net: dnn::read_net_from_onnx_buffer(&bytes).expect("Failed to load model"),
        }
    }
}
