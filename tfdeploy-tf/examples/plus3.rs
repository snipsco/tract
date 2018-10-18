extern crate ndarray;
extern crate tfdeploy;
extern crate tfdeploy_tf;
use tfdeploy_tf::tfpb;
use tfdeploy_tf::tfpb::types::DataType::DT_FLOAT;
use tfdeploy_tf::ToTensorflow;

fn main() {
    let plus3 = tfpb::node()
        .op("Add")
        .name("output")
        .attr("T", DT_FLOAT)
        .input("input")
        .input("three");
    let konst = tfpb::node()
        .op("Const")
        .name("three")
        .attr("dtype", DT_FLOAT)
        .attr(
            "value",
            tfdeploy::tensor::Tensor::from(::ndarray::arr1(&[3.0f32]))
                .to_tf()
                .unwrap(),
        );
    let input = tfpb::node()
        .op("Placeholder")
        .name("input")
        .attr("dtype", DT_FLOAT);
    let graph = tfpb::graph().node(input).node(konst).node(plus3);
    graph.save_to("tests/plus3.pb").unwrap();
}
