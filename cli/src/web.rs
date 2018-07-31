use WebParameters;
use std::fs::File;
use tfdeploy::*;
use errors::Result as CliResult;

#[derive(Serialize)]
struct EdgeSummary {
    id: usize,
    from_node: usize,
    from_out: usize,
    to_node: usize,
}

/// Generates a JSON representation of a Tensorflow graph.
pub fn generate_json(
    tfd: &Model,
    params: &WebParameters,
) -> Result<Vec<u8>> {
    let mut nodes: Vec<Node> = vec![];
    let mut edges: Vec<EdgeSummary> = vec![];

    for node in tfd.nodes() {
        if params.noconst && node.op_name == "Const" {
            continue;
        }
        nodes.push(node.clone());

        for &(from, port) in &node.inputs {
            let edge = EdgeSummary {
                id: edges.len(),
                from_node: from,
                from_out: port.unwrap_or(0),
                to_node: node.id,
            };

            edges.push(edge);
        }
    }

    let graph = (&nodes, &edges);
    let json = ::serde_json::to_vec(&graph).unwrap();

    Ok(json)
}

/// Starts a web server for TFVisualizer and opens its webroot in a browser.
pub fn open_web(model: &Model, params: &WebParameters) -> CliResult<()> {
    use rouille::Response;

    let data = generate_json(model, params)?;

    println!("TFVisualizer is now running on http://127.0.0.1:8000/.");
    let _ = ::open::that("http://127.0.0.1:8000/");

    ::rouille::start_server("0.0.0.0:8000", move |request| {
        if request.remove_prefix("/dist").is_some() || request.remove_prefix("/public").is_some() {
            return ::rouille::match_assets(&request, "../visualizer");
        }

        return router!(request,
            (GET) (/) => {
                let index = File::open("../visualizer/index.html").unwrap();
                Response::from_file("text/html", index)
            },

            (GET) (/current) => {
                Response::from_data("application/json", data.clone())
            },

            _ => Response::empty_404(),
        );
    });
}

