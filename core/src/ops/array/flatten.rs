use crate::ops::prelude::*;

#[derive(Debug, Clone, new, Default)]
pub struct Flatten {
    axis: usize,
}

impl Flatten {
    /// Evaluates the operation given the input tensors.
    fn eval_t<T: Datum>(
        &self,
        input: SharedTensor,
        shape: (usize, usize),
    ) -> TractResult<TVec<SharedTensor>> {
        Ok(tvec![input.to_array::<T>()?.into_shape(shape)?.into()])
    }
}

impl Op for Flatten {
    fn name(&self) -> Cow<str> {
        "Flatten".into()
    }
}

impl StatelessOp for Flatten {
    fn eval(&self, mut inputs: TVec<SharedTensor>) -> TractResult<TVec<SharedTensor>> {
        let input = args_1!(inputs);
        let shape_0 = input.shape()[..self.axis].iter().product::<usize>();
        let shape_1 = input.shape()[self.axis..].iter().product::<usize>();
        dispatch_datum!(Self::eval_t(input.datum_type())(
            self,
            input,
            (shape_0, shape_1)
        ))
    }
}

impl InferenceRulesOp for Flatten {
    fn rules<'r, 'p: 'r, 's: 'r>(
        &'s self,
        s: &mut Solver<'r>,
        inputs: &'p TensorsProxy,
        outputs: &'p TensorsProxy,
    ) -> InferenceResult {
        s.equals(&outputs[0].datum_type, &inputs[0].datum_type)?;
        s.given(&inputs[0].shape, move |s, shape| {
            let shape_0 = shape[..self.axis]
                .iter()
                .fold(TDim::from(1), |acc, &v| acc * v);
            let shape_1 = shape[self.axis..]
                .iter()
                .fold(TDim::from(1), |acc, &v| acc * v);
            s.equals(&outputs[0].shape, ShapeFact::from(vec![shape_0, shape_1]))
        })
    }
}
