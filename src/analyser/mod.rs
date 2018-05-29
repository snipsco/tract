use errors::*;
use ops::Op;
use Model;
use Plan;

mod types;
pub use self::types::*;

#[macro_use]
pub mod macros;
#[macro_use]
pub mod helpers;
pub mod graphviz;
#[cfg(test)]
mod tests;

/// Attempts to unify two abstract tensors into a more specialized one.
pub fn unify(x: &ATensor, y: &ATensor) -> Result<ATensor> {
    let tensor = ATensor {
        datatype: unify_datatype(&x.datatype, &y.datatype)?,
        shape: unify_shape(&x.shape, &y.shape)?,
        value: unify_value(&x.value, &y.value)?,
    };

    Ok(tensor)
}

/// Attempts to unify two abstract datatypes.
pub fn unify_datatype(x: &AType, y: &AType) -> Result<AType> {
    use self::AType::*;

    let datatype = match (x, y) {
        (_, Any) => x.clone(),
        (Any, _) => y.clone(),
        (Only(a), Only(b)) => if a == b {
            x.clone()
        } else {
            bail!("Impossible to unify datatypes {:?} and {:?}.", x, y);
        },
    };

    Ok(datatype)
}

/// Attempts to unify two abstract shapes.
pub fn unify_shape(x: &AShape, y: &AShape) -> Result<AShape> {
    use self::ADimension::*;
    use self::AShape::*;
    use itertools::EitherOrBoth::{Both, Left, Right};
    use itertools::Itertools;

    let xi = x.inner().iter();
    let yi = y.inner().iter();

    let dimensions: Vec<_> = xi.zip_longest(yi)
        .map(|r| match r {
            Both(Any, Any) => Ok(Any),

            Both(Only(i), Any) | Both(Any, Only(i)) => Ok(Only(*i)),

            Both(Only(i), Only(j)) if i == j => Ok(Only(*i)),
            Both(Only(i), Only(j)) => bail!("Impossible to unify dimensions {:?} and {:?}.", i, j),

            Left(d) if y.is_open() => Ok(d.clone()),
            Right(d) if x.is_open() => Ok(d.clone()),

            Left(_) | Right(_) => bail!(
                "Impossible to unify closed shapes of different rank (found {:?} and {:?}).",
                x,
                y
            ),
        })
        .collect::<Result<_>>()?;

    if x.is_open() && y.is_open() {
        Ok(Open(dimensions))
    } else {
        Ok(Closed(dimensions))
    }
}

/// Attempts to unify two abstract values.
pub fn unify_value(x: &AValue, y: &AValue) -> Result<AValue> {
    use self::AValue::*;

    let value = match (x, y) {
        (_, Any) => x.clone(),
        (Any, _) => y.clone(),
        (Only(a), Only(b)) => if a == b {
            x.clone()
        } else {
            bail!("Impossible to unify values {:?} and {:?}.", x, y);
        },
    };

    Ok(value)
}

#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    pub id: usize,
    pub from_node: usize,
    pub from_out: usize,
    pub to_node: usize,
    pub tensor: ATensor,
}

/// Runs the analyser on the given graph.
///
/// The output argument is used to infer an execution plan for the graph.
/// Changing it won't alter the correctness of the analysis, but it might
/// take much longer to complete.
pub fn analyse<'a>(model: &'a Model, output: usize, debug: bool) -> Result<Vec<Edge>> {
    // We first give an identity to each edge of the graph.
    let mut edges = vec![];
    let mut prev_edges = vec![Vec::new(); model.nodes().len()];
    let mut next_edges = vec![Vec::new(); model.nodes().len()];

    for node in model.nodes() {
        for input in &node.inputs {
            let id = edges.len();

            edges.push(Edge {
                id,
                from_node: input.0,
                from_out: input.1.unwrap_or(0),
                to_node: node.id,
                tensor: ATensor::new(),
            });

            prev_edges[node.id].push(id);
            next_edges[input.0].push(id);
        }
    }

    // Compute and run an execution plan for the graph.
    let plan = Plan::for_node(model, output)?;
    let mut changed;
    let mut forward = true;

    macro_rules! one_pass {
        ($source:ident, $target:ident, $fn:ident) => {{
            // TODO(liautaud): Remove this.
            if debug {
                println!("");
                println!("Starting a round of {}.", stringify!($fn));
            }

            for &n in &plan.order {
                let inferred = {
                    let sources: Vec<_> = $source[n].iter().map(|&i| &edges[i].tensor).collect();

                    let node = model.get_node_by_id(n)?;
                    let inferred = node.op.$fn(sources);

                    if inferred.is_err() {
                        // TODO(liautaud): Remove this.
                        if debug {
                            println!("- {} ({}): {}", n, node.op_name, inferred.unwrap_err());
                        }
                        continue;
                    }

                    inferred.unwrap()
                };

                for (i, &j) in $target[n].iter().enumerate() {
                    let unified = unify(&inferred[i], &edges[j].tensor)?;
                    if unified != edges[j].tensor {
                        edges[j].tensor = unified;
                        changed = true;
                    }

                    // TODO(liautaud): Remove this.
                    if debug {
                        let node_name = format!("[{}] ({})", n, model.get_node_by_id(n)?.op_name);
                        let mut inferred_display = format!("{:?}", inferred);
                        let mut unified_display = format!("{:?}", edges[j].tensor);
                        inferred_display.truncate(150);
                        unified_display.truncate(150);
                        println!(
                            "{}\n- Inferred: {}\n- Unified: {}",
                            node_name, inferred_display, unified_display,
                        );
                    }
                }

                // TODO(liautaud): Remove this.
                if debug && model.get_node_by_id(n)?.op_name != "Const" {
                    graphviz::display_graph("debug".to_string(), &model, &edges, &vec![n])?;
                }
            }
        }};
    };

    // TODO(liautaud): Remove this.
    if debug {
        graphviz::display_graph("debug".to_string(), &model, &edges, &vec![])?;
    }

    loop {
        changed = false;

        if forward {
            one_pass!(prev_edges, next_edges, infer_forward);
        } else {
            one_pass!(next_edges, prev_edges, infer_backward);
        }

        forward = !forward;

        if !changed {
            break;
        }
    }

    Ok(edges)
}
