use std::fmt;

use crate::internal::*;
use ndarray::prelude::*;

// Scan node outer interface:
// inputs: [ hidden_state_len initial values ][ num_scan_inputs inputs ][ implicit capture inputs ]
// outputs: [ hidden_state_len final values ][ aggregated outputs ]

#[derive(Debug, Clone, new, Default)]
pub struct Scan<TI, O>
where
    TI: TensorInfo,
    O: fmt::Debug + fmt::Display + AsRef<Op> + AsMut<Op>,
{
    pub body: Model<TI, O>,
    num_scan_inputs: usize,
    closure_inputs: usize,
    scan_input_axes: Vec<usize>,
    scan_output_axes: Vec<usize>,
}

impl<TI, O> Scan<TI, O>
where
    TI: TensorInfo,
    O: fmt::Debug + fmt::Display + AsRef<Op> + AsMut<Op>,
{
    fn slice_input_t<T: Datum>(
        &self,
        scan_inputs: &[Arc<Tensor>],
        input: usize,
        i: usize,
    ) -> TractResult<Tensor> {
        let view = scan_inputs[input].to_array_view::<T>()?;
        let axis = self.scan_input_axes.get(input).cloned().unwrap_or(0);
        let slice = view.slice_axis(Axis(axis), (i..=i).into());
        Ok(slice.to_owned().into_tensor())
    }

    fn alloc_output_t<T: Datum + Default>(
        &self,
        element_shape: &[usize],
        output: usize,
        iters: usize,
    ) -> TractResult<Tensor> {
        let axis = self.scan_output_axes.get(output).cloned().unwrap_or(0);
        let mut shape = element_shape.to_vec();
        shape[axis] = iters;
        Ok(Array::<T, _>::default(&*shape).into())
    }

    fn assign_output_t<T: Datum + Default>(
        &self,
        output: &mut Tensor,
        output_id: usize,
        element_value: &Tensor,
        i: usize,
    ) -> TractResult<()> {
        let axis = self.scan_output_axes.get(output_id).cloned().unwrap_or(0);
        let mut view = output.to_array_view_mut::<T>()?;
        let mut slice = view.slice_axis_mut(Axis(axis), (i..=i).into());
        let element = element_value.to_array_view::<T>()?;
        slice.assign(&element);
        Ok(())
    }
}

impl Op for Scan<TensorFact, Box<InferenceOp>> {
    fn name(&self) -> Cow<str> {
        "Scan".into()
    }

    fn to_typed(&self) -> TractResult<Option<Box<Op>>> {
        let typed_model = self.body.clone().into_typed()?;
        Ok(Some(Box::new(Scan::new(
            typed_model,
            self.num_scan_inputs,
            self.closure_inputs,
            self.scan_input_axes.clone(),
            self.scan_output_axes.clone(),
        ))))
    }
}

impl StatelessOp for Scan<TensorFact, Box<InferenceOp>> {
    fn eval(&self, _inputs: TVec<Arc<Tensor>>) -> TractResult<TVec<Arc<Tensor>>> {
        unimplemented!()
    }
}

impl Op for Scan<TypedTensorInfo, Box<Op>> {
    fn name(&self) -> Cow<str> {
        "Scan".into()
    }
}

impl StatelessOp for Scan<TypedTensorInfo, Box<Op>> {
    fn eval(&self, mut inputs: TVec<Arc<Tensor>>) -> TractResult<TVec<Arc<Tensor>>> {
        let hidden_state_len = inputs.len() - self.num_scan_inputs - self.closure_inputs;

        // extract hidden state original values from inputs
        let mut state: TVec<Tensor> = tvec!();
        for _ in 0..hidden_state_len {
            state.push(inputs.remove(0).into_tensor());
        }

        let iters = inputs[0].shape()[self.scan_input_axes.get(0).cloned().unwrap_or(0)];

        let mut scan_outputs = tvec!();
        for i in 0..(self.body.output_outlets()?.len() - hidden_state_len) {
            let fact = self.body.output_fact(hidden_state_len + i)?;
            let dt = fact.datum_type;
            let shape = fact.shape.as_finite().unwrap();
            let t = dispatch_datum!(Self::alloc_output_t(dt)(self, &*shape, i, iters))?;
            scan_outputs.push(t);
        }

        let plan = SimplePlan::new(&self.body)?;

        for i in 0..iters {
            // body inputs are state + one slice of each input
            let mut iter_inputs: TVec<Tensor> = state.drain().collect();
            for input in 0..self.num_scan_inputs {
                let tensor = dispatch_datum!(Self::slice_input_t(inputs[input].datum_type())(
                    self, &*inputs, input, i
                ))?;
                if cfg!(debug_assert) {
                    self.body
                        .input_fact(input + hidden_state_len)?
                        .to_tensor_fact()
                        .unify(&TensorFact::from(tensor.clone()))?;
                }
                iter_inputs.push(tensor);
            }
            for i in 0..self.closure_inputs {
                iter_inputs
                    .push(inputs[inputs.len() - self.closure_inputs + i].clone().into_tensor());
            }
            println!("inputs: {:?}", iter_inputs);
            let mut iter_outputs = plan.run(iter_inputs).unwrap();
            println!("outputs: {:?}", iter_outputs);
            for _ in 0..hidden_state_len {
                state.push(iter_outputs.remove(0).into_tensor());
            }
            for (ix, o) in scan_outputs.iter_mut().enumerate() {
                dispatch_datum!(Self::assign_output_t(o.datum_type())(
                    self,
                    o,
                    ix,
                    &iter_outputs[ix],
                    i
                ))?;
            }
        }
        let mut output: TVec<Arc<Tensor>> =
            state.into_iter().map(|t| t.into_arc_tensor()).collect();
        output.extend(scan_outputs.into_iter().map(|t| t.into_arc_tensor()));
        Ok(output)
    }
}

impl Scan<TensorFact, Box<InferenceOp>> {
    fn unify_facts(
        &mut self,
        inputs: &mut [TensorFact],
        outputs: &mut [TensorFact],
    ) -> TractResult<()> {
        let body_inputs = self.body.input_outlets()?.len();
        let body_outputs = self.body.output_outlets()?.len();
        let hidden_state_len = body_inputs - self.num_scan_inputs - self.closure_inputs;
        let num_scan_outputs = body_outputs - hidden_state_len;
        for i in 0..hidden_state_len {
            trace!("Unify hidden state #{}", i);
            let mut merged =
                self.body.input_fact(i)?.datum_type.unify(&self.body.output_fact(i)?.datum_type)?;
            Fact::unify_all(&mut [
                &mut merged,
                &mut inputs[i].datum_type,
                &mut outputs[i].datum_type,
            ])
            .map_err(|e| format!("while unifying hidden state datum_types #{}: {}", i, e))?;
            self.body.input_fact_mut(i)?.datum_type.unify_with(&mut merged)?;
            self.body.output_fact_mut(i)?.datum_type.unify_with(&mut merged)?;

            let mut merged =
                self.body.input_fact(i)?.shape.unify(&self.body.output_fact(i)?.shape)?;
            Fact::unify_all(&mut [&mut merged, &mut inputs[i].shape, &mut outputs[i].shape])
                .map_err(|e| format!("while unifying hidden state shapes #{}: {}", i, e))?;
            self.body.input_fact_mut(i)?.shape.unify_with(&mut merged)?;
            self.body.output_fact_mut(i)?.shape.unify_with(&mut merged)?;
        }
        let mut iters: Option<TDim> = None;
        for i in 0..self.num_scan_inputs {
            let axis = self.scan_input_axes.get(i).cloned().unwrap_or(0);
            let input = &mut inputs[hidden_state_len + i];
            input.shape.ensure_rank_at_least(axis);
            iters = iters.or_else(|| input.shape.dims().nth(axis).unwrap().concretize());
        }
        for i in 0..num_scan_outputs {
            let axis = self.scan_output_axes.get(i).cloned().unwrap_or(0);
            let output = &mut outputs[hidden_state_len + i];
            output.shape.ensure_rank_at_least(axis);
            iters = iters.or_else(|| output.shape.dims().nth(axis).unwrap().concretize());
        }
        trace!("Iterations: {:?}", iters);
        for i in 0..self.num_scan_inputs {
            trace!("Unifying scan input #{}", hidden_state_len + i);
            let incoming = &mut inputs[hidden_state_len + i];
            let inner = self.body.input_fact_mut(hidden_state_len + i)?;
            incoming.datum_type.unify_with(&mut inner.datum_type)?;
            if let Some(shape) = incoming.shape.concretize() {
                let mut shape: Vec<TDim> = shape.to_vec();
                let axis = self.scan_input_axes.get(i).cloned().unwrap_or(0);
                shape.remove(axis);
                inner.shape.unify_with(&mut ShapeFact::from(shape))?;
            }
        }
        for i in 0..self.closure_inputs {
            let id = hidden_state_len + self.num_scan_inputs + i;
            trace!("Unifying closure input #{}", id);
            inputs[id].unify_with(self.body.input_fact_mut(id)?)?;
        }
        for i in 0..num_scan_outputs {
            let inner = self.body.output_fact_mut(hidden_state_len + i)?;
            let outgoing = &mut outputs[hidden_state_len + i];
            outgoing.datum_type.unify_with(&mut inner.datum_type)?;
            if let (Some(shape), Some(iters)) = (inner.shape.concretize(), iters.clone()) {
                let mut shape: Vec<TDim> = shape.to_vec();
                let axis = self.scan_output_axes.get(i).cloned().unwrap_or(0);
                shape.insert(axis, iters);
                outgoing.shape.unify_with(&mut ShapeFact::from(shape))?;
            }
        }
        Ok(())
    }
}

impl InferenceOp for Scan<TensorFact, Box<InferenceOp>> {
    fn infer_facts(
        &mut self,
        inputs: TVec<&TensorFact>,
        outputs: TVec<&TensorFact>,
        _observed: TVec<&TensorFact>,
    ) -> TractResult<(TVec<TensorFact>, TVec<TensorFact>, TVec<TensorFact>)> {
        let body_inputs = self.body.input_outlets()?.len();
        let body_outputs = self.body.output_outlets()?.len();
        if inputs.len() != body_inputs {
            bail!("Scan receives {} inputs, inner model expects {}", inputs.len(), body_inputs)
        }
        if outputs.len() != body_outputs {
            bail!("Scan has {} outputs, inner model expects {}", outputs.len(), body_outputs)
        }
        let mut inputs: TVec<TensorFact> = inputs.into_iter().cloned().collect();
        let mut outputs: TVec<TensorFact> = outputs.into_iter().cloned().collect();
        self.unify_facts(&mut inputs, &mut outputs)?;
        trace!("Starting inner model analyse");
        for (ix, input) in self.body.input_outlets()?.iter().enumerate() {
            trace!("  Input inner model: {} {:?} {:?}", ix, input, self.body.input_fact(ix));
        }
        for (ix, output) in self.body.output_outlets()?.iter().enumerate() {
            trace!("  Output inner model: {} {:?} {:?}", ix, output, self.body.output_fact(ix));
        }
        self.body
            .analyse(false)
            .map_err(|e| format!("analysing inner model: {}\n{:#?}", e, self.body))?;
        trace!("Finished inner model analyse");
        self.unify_facts(&mut inputs, &mut outputs)?;
        Ok((inputs, outputs, tvec!()))
    }

    inference_op_as_op!();
}
