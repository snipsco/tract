use std::str;

use crate::model::ParsingContext;
use crate::pb::NodeProto;
use tract_core::ops::cnn::KernelFormat;
use tract_hir::internal::*;
use tract_hir::ops::cnn::PaddingSpec;
use tract_hir::ops::nn::DataFormat;
use tract_itertools::izip;

pub fn conv_transpose(
    _ctx: &ParsingContext,
    node: &NodeProto,
) -> TractResult<(Box<dyn InferenceOp>, Vec<String>)> {
    let padding_spec = super::pad(node)?;
    let strides = super::strides(node)?;
    let dilations = super::dilations(node)?;
    let adjustments = node.get_attr_opt_tvec::<usize>("output_padding")?;
    let output_shape = node.get_attr_opt_tvec::<usize>("output_shape")?;
    Ok((
        expand(ConvTranspose::new(padding_spec, strides, dilations, adjustments, output_shape)),
        vec![],
    ))
}

#[derive(Debug, Clone, new, Default, Hash)]
pub struct ConvTranspose {
    padding_spec: PaddingSpec,
    strides: Option<TVec<usize>>,
    dilations: Option<TVec<usize>>,
    adjustments: Option<TVec<usize>>,
    output_shape: Option<TVec<usize>>,
}

impl_dyn_hash!(ConvTranspose);

impl Expansion for ConvTranspose {
    fn name(&self) -> Cow<str> {
        "ConvTranspose".into()
    }

    op_onnx!();

    fn rules<'r, 'p: 'r, 's: 'r>(
        &'s self,
        s: &mut Solver<'r>,
        inputs: &'p [TensorProxy],
        outputs: &'p [TensorProxy],
    ) -> InferenceResult {
        check_input_arity(inputs, 2)?;
        check_output_arity(outputs, 1)?;
        s.equals(&inputs[0].datum_type, &outputs[0].datum_type)?;
        s.equals(&inputs[0].datum_type, &inputs[1].datum_type)?;
        s.equals(&inputs[0].rank, &inputs[1].rank)?;
        s.equals(&inputs[0].rank, &outputs[0].rank)?;
        s.equals(&outputs[0].shape[0], &inputs[0].shape[0])?; // N
        s.equals(&inputs[0].shape[1], &inputs[1].shape[0])?; // O
        s.equals(&outputs[0].shape[1], &inputs[1].shape[1])?; // I

        s.given_2(&inputs[0].shape, &inputs[1].shape, move |s, x_shape, w_shape| {
            if let (Ok(x_shape), Ok(w_shape)) = (
                x_shape.iter().map(|d| d.to_usize()).collect::<TractResult<TVec<usize>>>(),
                w_shape.iter().map(|d| d.to_usize()).collect::<TractResult<TVec<usize>>>(),
            ) {
                let ones = tvec!(1; x_shape.len() - 2);
                let zeros = tvec!(0; x_shape.len() - 2);
                let y_shape = if let Some(output_shape) = &self.output_shape {
                    let mut y_shape = x_shape.clone();
                    y_shape[1] = w_shape[1];
                    for (ix, d) in output_shape.iter().enumerate() {
                        y_shape[ix + 2] = *d;
                    }
                    y_shape
                } else {
                    tract_core::ops::cnn::deconv::output_shape(
                        &DataFormat::NCHW,
                        &KernelFormat::OIHW,
                        &self.padding_spec,
                        &*w_shape,
                        &x_shape,
                        &self.strides.clone().unwrap_or(ones.clone()),
                        &self.dilations.clone().unwrap_or(ones.clone()),
                        &self.adjustments.clone().unwrap_or(zeros.clone()),
                    )?
                };
                let y_shape = y_shape.iter().map(|x| x.to_dim()).collect::<TVec<TDim>>();
                s.equals(&outputs[0].shape, y_shape)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn wire(
        &self,
        prefix: &str,
        target: &mut TypedModel,
        inputs: &[OutletId],
    ) -> TractResult<TVec<OutletId>> {
        if let Some(k) = target.outlet_fact(inputs[1])?.konst.clone() {
            let ones = tvec!(1; k.rank() - 2);
            let zeros = tvec!(0; k.rank() - 2);
            if let Some(output_shape) = &self.output_shape {
                let x_shape = &target.outlet_fact(inputs[0])?.shape;
                let w_shape = &target.outlet_fact(inputs[1])?.shape;
                let adjustments = izip!(
                    &x_shape[2..],
                    &w_shape[2..],
                    &*output_shape,
                    &*self.strides.as_deref().unwrap_or(&ones),
                    &*self.dilations.as_deref().unwrap_or(&ones),
                )
                .map(|(x, k, y, s, d)| {
                    let pad = y - s * (x.to_usize()? - 1) - (k.to_usize()? - 1) * d - 1;
                    Ok(pad) }
                ).collect::<TractResult<TVec<usize>>>()?;
                target.wire_node(
                    prefix,
                    tract_core::ops::cnn::DeconvUnary::new(
                        DataFormat::NCHW,
                        KernelFormat::OIHW,
                        self.padding_spec.clone(),
                        k.clone(),
                        self.strides.clone().unwrap_or(ones.clone()),
                        self.dilations.clone().unwrap_or(ones.clone()),
                        adjustments
                    ),
                    &[inputs[0]],
                )
            } else {
                target.wire_node(
                    prefix,
                    tract_core::ops::cnn::DeconvUnary::new(
                        DataFormat::NCHW,
                        KernelFormat::OIHW,
                        self.padding_spec.clone(),
                        k.clone(),
                        self.strides.clone().unwrap_or(ones.clone()),
                        self.dilations.clone().unwrap_or(ones.clone()),
                        self.adjustments.clone().unwrap_or(zeros.clone()),
                    ),
                    &[inputs[0]],
                )
            }
        } else {
            bail!("Kernel values are expected to be constant.")
        }
    }
}
