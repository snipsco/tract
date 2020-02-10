use crate::internal::*;
use ndarray::prelude::*;
use num_traits::cast::AsPrimitive;

macro_rules! reduce_numbers {
    ($($path:ident)::* ($dt:expr) ($($args:expr),*)) => {
        match $dt {
            DatumType::U8   => $($path)::*::<u8,_>($($args),*),
            DatumType::U16  => $($path)::*::<u16,_>($($args),*),
            DatumType::I8   => $($path)::*::<i8,_>($($args),*),
            DatumType::I16  => $($path)::*::<i16,_>($($args),*),
            DatumType::I32  => $($path)::*::<i32,_>($($args),*),
            DatumType::I64  => $($path)::*::<i64,_>($($args),*),
            DatumType::F32  => $($path)::*::<f32,_>($($args),*),
            DatumType::F64  => $($path)::*::<f64,_>($($args),*),
            _ => bail!("{:?} is not a number", $dt)
        }
    }
}

macro_rules! reduce_floatlike {
    ($($path:ident)::* ($dt:expr) ($($args:expr),*)) => {
        match $dt {
            DatumType::F32  => $($path)::*::<f32,_>($($args),*),
            DatumType::F64  => $($path)::*::<f64,_>($($args),*),
            _ => bail!("{:?} is not float like", $dt)
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Reducer {
    L1,
    L2,
    LogSum,
    LogSumExp,
    Max,
    Mean,
    Min,
    Prod,
    Sum,
    SumSquare,
}

impl Reducer {
    pub fn reduce(&self, axes: &[usize], input: Arc<Tensor>) -> TractResult<Tensor> {
        let dt = input.datum_type();
        match self {
            Reducer::L1 => match dt {
                DatumType::U8 => self.reduce_t::<u8, _>(axes, input, l1u_t),
                DatumType::U16 => self.reduce_t::<u16, _>(axes, input, l1u_t),
                DatumType::I8 => self.reduce_t::<i8, _>(axes, input, l1s_t),
                DatumType::I16 => self.reduce_t::<i16, _>(axes, input, l1s_t),
                DatumType::I32 => self.reduce_t::<i32, _>(axes, input, l1s_t),
                DatumType::I64 => self.reduce_t::<i64, _>(axes, input, l1s_t),
                DatumType::F32 => self.reduce_t::<f32, _>(axes, input, l1s_t),
                DatumType::F64 => self.reduce_t::<f64, _>(axes, input, l1s_t),
                _ => bail!("{:?} is not a number valid for L1 norm", dt),
            },
            Reducer::L2 => reduce_numbers!(Self::reduce_t(dt)(self, axes, input, l2_t)),
            Reducer::LogSum => reduce_floatlike!(Self::reduce_t(dt)(self, axes, input, log_sum_t)),
            Reducer::LogSumExp => {
                reduce_floatlike!(Self::reduce_t(dt)(self, axes, input, log_sum_exp_t))
            }
            Reducer::Mean => reduce_numbers!(Self::reduce_t(dt)(self, axes, input, mean_t)),
            Reducer::Min => reduce_numbers!(Self::reduce_t(dt)(self, axes, input, min_t)),
            Reducer::Max => reduce_numbers!(Self::reduce_t(dt)(self, axes, input, max_t)),
            Reducer::Prod => reduce_numbers!(Self::reduce_t(dt)(self, axes, input, prod_t)),
            Reducer::Sum => reduce_numbers!(Self::reduce_t(dt)(self, axes, input, sum_t)),
            Reducer::SumSquare => {
                reduce_numbers!(Self::reduce_t(dt)(self, axes, input, sum_square_t))
            }
        }
    }

    fn reduce_t<T, F>(&self, axes: &[usize], input: Arc<Tensor>, f: F) -> TractResult<Tensor>
    where
        F: for<'a> Fn(ArrayViewD<'a, T>) -> T,
        T: Copy + Datum,
    {
        use ndarray::*;
        let input = input.to_array_view::<T>()?;
        let full_output_shape: Vec<usize> = input
            .shape()
            .iter()
            .enumerate()
            .map(|(ax, &d)| if axes.contains(&ax) { 1 } else { d })
            .collect();
        let result = Array::from_shape_fn(&*full_output_shape, |coords| {
            let slice_spec: Vec<SliceOrIndex> = coords
                .slice()
                .iter()
                .enumerate()
                .map(|(ax, &d)| if axes.contains(&ax) { (..).into() } else { d.into() })
                .collect();
            let slice_info = SliceInfo::new(&slice_spec).unwrap();
            let slice = input.slice(slice_info.as_ref());
            f(slice)
        });
        Ok(result.into_tensor())
    }
}

fn l1s_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Signed + num_traits::Zero,
{
    v.fold(T::zero(), |acc, &v| acc + v.abs())
}

fn l1u_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Unsigned + num_traits::Zero,
{
    v.fold(T::zero(), |acc, &v| acc + v)
}

fn l2_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + AsPrimitive<f64>,
    f64: AsPrimitive<T>,
{
    v.fold(0.0f64, |acc, &v| acc + (v.as_()).powi(2)).sqrt().as_()
}

fn log_sum_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Zero + num_traits::Float,
{
    v.scalar_sum().ln()
}

fn log_sum_exp_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Zero + num_traits::Float,
{
    let max = v.fold(T::min_value(), |acc, &v| if acc > v { acc } else { v });
    max + v.fold(T::zero(), |acc, &v| acc + (v - max).exp()).ln()
}

fn max_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Bounded + ::std::cmp::PartialOrd,
{
    v.fold(T::min_value(), |acc, &v| if acc > v { acc } else { v })
}

fn mean_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Zero + ::std::ops::Div<Output = T>,
    usize: AsPrimitive<T>,
{
    let (sum, count) = v.fold((T::zero(), 0), |acc, &v| (acc.0 + v, acc.1 + 1));
    sum / count.as_()
}

fn min_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Bounded + ::std::cmp::PartialOrd,
{
    v.fold(T::max_value(), |acc, &v| if acc < v { acc } else { v })
}

fn prod_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::One,
{
    v.fold(T::one(), |acc, &v| acc * v)
}

fn sum_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Zero,
{
    v.scalar_sum()
}

fn sum_square_t<'a, T>(v: ArrayViewD<'a, T>) -> T
where
    T: Copy + Datum + num_traits::Zero + ::std::ops::Mul<T, Output = T>,
{
    v.fold(T::zero(), |acc, &v| acc + v * v)
}

#[derive(Clone, Debug, new)]
pub struct Reduce {
    axes: TVec<usize>,
    reducer: Reducer,
}

impl Op for Reduce {
    fn name(&self) -> Cow<str> {
        format!("Reduce<{:?}>", self.reducer).into()
    }
    fn info(&self) -> TractResult<Vec<String>> {
        Ok(vec![format!("axes: {:?}", self.axes)])
    }
    canonic!();
    op_as_typed_op!();
    op_as_pulsed_op!();
}

impl StatelessOp for Reduce {
    fn eval(&self, mut inputs: TVec<Arc<Tensor>>) -> TractResult<TVec<Arc<Tensor>>> {
        Ok(tvec!(self.reducer.reduce(&*self.axes, args_1!(inputs))?.into_arc_tensor()))
    }
}

impl TypedOp for Reduce {
    as_op!();
    fn output_facts(&self, inputs: &[&TypedFact]) -> TractResult<TVec<TypedFact>> {
        let mut shape: TVec<_> = inputs[0].shape.to_tvec();
        for &ax in &self.axes {
            shape[ax] = 1.to_dim();
        }
        Ok(tvec!(TypedFact::dt_shape(inputs[0].datum_type, &*shape)?))
    }

    #[allow(unused_variables)]
    fn invariants(&self, model: &TypedModel, node: &TypedNode) -> TractResult<Invariants> {
        let input = model.outlet_fact(node.inputs[0])?;
        let axes = (0..input.rank())
            .filter(|axis| !self.axes.contains(axis))
            .map(|axis| AxisInfo::simple(axis))
            .collect::<TVec<_>>();
        Ok(axes.into())
    }

    fn change_axes(
        &self,
        model: &TypedModel,
        node: &TypedNode,
        _io: InOut,
        change: &AxisOp,
    ) -> TractResult<Option<AxisChangeConsequence>> {
        let mut axes = tvec!();
        for reduced in &self.axes {
            if let Some(axis) = change.transform_axis(*reduced) {
                axes.push(axis);
            } else {
                return Ok(None);
            }
        }
        let op = Some(Box::new(Self { axes, ..self.clone() }) as _);
        Ok(Some(AxisChangeConsequence::new(model, node, op, change)))
    }

    fn pulsify(
        &self,
        _source: &NormalizedModel,
        node: &NormalizedNode,
        target: &mut PulsedModel,
        mapping: &HashMap<OutletId, OutletId>,
        _pulse: usize,
    ) -> TractResult<TVec<OutletId>> {
        let input = mapping[&node.inputs[0]];
        let axis = target.outlet_fact(input)?.axis;
        if self.axes.contains(&axis) {
            bail!("Can not reduce over streaming axis");
        }
        target.wire_node(&*node.name, self.clone(), &[input])
    }
}

impl PulsedOp for Reduce {
    fn pulsed_output_facts(&self, inputs: &[&PulsedFact]) -> TractResult<TVec<PulsedFact>> {
        let mut fact = inputs[0].clone();
        for &ax in &self.axes {
            fact.shape[ax] = 1;
        }
        Ok(tvec!(fact))
    }

    as_op!();
    pulsed_op_to_typed_op!();
}
