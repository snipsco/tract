//! TensorFlow Ops
use std::fmt::Debug;

use downcast_rs::Downcast;

use model::TVec;

use objekt;

#[macro_use]
mod macros;

pub mod array;
pub mod identity;
#[cfg(features = "image_ops")]
pub mod image;
pub mod konst;
pub mod logic;
pub mod math;
pub mod nn;
pub mod source;
pub mod unimpl;

mod types;

#[derive(Debug, Copy, Clone, Default)]
pub struct StreamInfo {
    pub axis: usize,
    pub len: TDim,
}

pub mod prelude {
    pub use super::{InferenceOp, Op, OpState, ReducedOpRewire, StatefullOp, StatelessOp, StreamInfo};
    pub use dim::{DimLike, TDim, ToDim};
    pub use analyser::types::*;
    pub use analyser::types::TypeFact;
    pub use analyser::rules::{ InferenceResult, InferenceRulesOp, Solver, TensorsProxy };
    pub use analyser::rules::expr::{ IntoExp, ToDimExp };
    pub use model::TVec;
    pub use ops::types::Value;
    pub use pulse::{PulsifiedOp, PulsedTensorFact};
    pub use std::collections::HashMap;
    pub use std::marker::PhantomData;
    pub use tensor::arr4;
    pub use tensor::{Datum, DatumType, Tensor};
    pub use TfdResult;
}

use self::prelude::*;

pub trait OpState: Debug {
    fn eval(&mut self, op: &Op, inputs: TVec<Value>) -> TfdResult<TVec<Value>>;
}

impl OpState for Option<Box<OpState>> {
    fn eval(&mut self, op: &Op, inputs: TVec<Value>) -> TfdResult<TVec<Value>> {
        match self {
            Some(state) => state.eval(op, inputs),
            None => op.as_stateless().unwrap().eval(inputs),
        }
    }
}

pub trait StatelessOp {
    fn eval(&self, inputs: TVec<Value>) -> TfdResult<TVec<Value>>;
}

pub trait StatefullOp {
    fn state(&self) -> TfdResult<Option<Box<OpState>>>;
    fn as_stateless(&self) -> Option<&StatelessOp> {
        None
    }
}

impl<O: StatelessOp + Clone> StatefullOp for O {
    fn state(&self) -> TfdResult<Option<Box<OpState>>> {
        Ok(None)
    }

    fn as_stateless(&self) -> Option<&StatelessOp> {
        Some(self)
    }
}

/// A Tensorflow operation.
impl_downcast!(Op);
pub trait Op:
    Debug + objekt::Clone + Send + Sync + 'static + InferenceOp + Downcast + StatefullOp
{
    fn name(&self) -> &str;

    /// Infers properties about the input and output tensors.
    ///
    /// The `inputs` and `outputs` arguments correspond to properties about
    /// the input and output tensors that are already known.
    ///
    /// Returns Err in case of an unrecoverable error during the inference,
    /// and the refined properties about the inputs and outputs otherwise.
    fn infer(
        &self,
        inputs: TVec<&TensorFact>,
        outputs: TVec<&TensorFact>,
    ) -> TfdResult<(TVec<TensorFact>, TVec<TensorFact>)> {
        let (infered_inputs, infered_outputs) = self.infer_facts(inputs, outputs)?;

        if let Some(stateless) = self.as_stateless() {
            if infered_inputs.iter().all(|i| i.value.is_concrete()) {
                let input_values = infered_inputs
                    .iter()
                    .map(|i| i.value.concretize().unwrap().clone().into())
                    .collect(); // checked
                let output_value = stateless.eval(input_values)?.pop().unwrap();
                return Ok((
                    infered_inputs,
                    tvec![::analyser::helpers::tensor_to_fact(
                        output_value.into_tensor(),
                    )],
                ));
            }
        }

        Ok((infered_inputs, infered_outputs))
    }

    fn reduce(
        &self,
        _inputs: TVec<&TensorFact>,
        _outputs: TVec<&TensorFact>,
    ) -> TfdResult<Option<ReducedOpRewire>> {
        Ok(None)
    }

    fn pulsify(
        &self,
        _inputs: TVec<&PulsedTensorFact>,
    ) -> TfdResult<::pulse::PulsifiedOp> {
        bail!("Operator {} do not support pulsification", self.name())
    }

    fn const_value(&self) -> Option<Value> {
        None
    }

    fn rounding_errors(&self) -> bool {
        false
    }

    fn noutputs(&self) -> usize {
        1
    }
}

pub trait InferenceOp {
    fn infer_facts(
        &self,
        inputs: TVec<&TensorFact>,
        outputs: TVec<&TensorFact>,
    ) -> TfdResult<(TVec<TensorFact>, TVec<TensorFact>)>;
}

clone_trait_object!(Op);

#[derive(Clone, Debug, new)]
pub struct ReducedOpRewire {
    pub new_op: Box<Op>,
    pub rewired: TVec<usize>,
}
