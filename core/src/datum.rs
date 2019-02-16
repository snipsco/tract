//! `Tensor` is the equivalent of SharedTensor Tensor.
use crate::dim::TDim;
use crate::tensor::Tensor;
use crate::TractResult;
use ndarray::prelude::*;
use std::fmt;

use crate::ndarray_dummy_packed_mm::*;
use tract_linalg::f16::f16;

#[cfg(feature = "serialize")]
use serde::ser::{Serialize, Serializer};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
pub enum DatumType {
    Bool,
    U8,
    U16,
    I8,
    I16,
    I32,
    I64,
    F16,
    F32,
    F64,
    TDim,
    String,
}

impl DatumType {
    pub fn super_types(&self) -> &'static [DatumType] {
        match self {
            DatumType::Bool => &[DatumType::Bool],
            DatumType::U8 => &[
                DatumType::U8,
                DatumType::I16,
                DatumType::I32,
                DatumType::I64,
                DatumType::TDim,
            ],
            DatumType::U16 => &[
                DatumType::U16,
                DatumType::I32,
                DatumType::I64,
                DatumType::TDim,
            ],
            DatumType::I8 => &[
                DatumType::I8,
                DatumType::I16,
                DatumType::I32,
                DatumType::I64,
                DatumType::TDim,
            ],
            DatumType::I16 => &[
                DatumType::I16,
                DatumType::I32,
                DatumType::I64,
                DatumType::TDim,
            ],
            DatumType::I32 => &[DatumType::I32, DatumType::I64, DatumType::TDim],
            DatumType::I64 => &[DatumType::I64, DatumType::TDim],
            DatumType::F16 => &[DatumType::F16, DatumType::F32, DatumType::F64],
            DatumType::F32 => &[DatumType::F32, DatumType::F64],
            DatumType::F64 => &[DatumType::F64],
            DatumType::String => &[DatumType::String],
            DatumType::TDim => &[DatumType::TDim],
        }
    }

    pub fn super_type_for<I: IntoIterator<Item = DatumType>>(i: I) -> Option<DatumType> {
        let mut iter = i.into_iter();
        let mut current = match iter.next() {
            None => return None,
            Some(it) => it,
        };
        while let Some(n) = iter.next() {
            match current.common_super_type(n) {
                None => return None,
                Some(it) => current = it,
            }
        }
        Some(current)
    }

    pub fn common_super_type(&self, rhs: DatumType) -> Option<DatumType> {
        for mine in self.super_types() {
            for theirs in rhs.super_types() {
                if mine == theirs {
                    return Some(*mine);
                }
            }
        }
        return None;
    }

    pub fn size_of(&self) -> usize {
        match self {
            DatumType::Bool => std::mem::size_of::<bool>(),
            DatumType::U8 => std::mem::size_of::<u8>(),
            DatumType::U16 => std::mem::size_of::<u16>(),
            DatumType::I8 => std::mem::size_of::<i8>(),
            DatumType::I16 => std::mem::size_of::<i16>(),
            DatumType::I32 => std::mem::size_of::<i32>(),
            DatumType::I64 => std::mem::size_of::<i64>(),
            DatumType::F16 => std::mem::size_of::<f16>(),
            DatumType::F32 => std::mem::size_of::<f32>(),
            DatumType::F64 => std::mem::size_of::<f64>(),
            DatumType::TDim => std::mem::size_of::<TDim>(),
            DatumType::String => std::mem::size_of::<String>(),
        }
    }

    pub fn alignment(&self) -> usize {
        match self {
            DatumType::TDim => std::mem::size_of::<usize>(),
            DatumType::String => std::mem::size_of::<usize>(),
            _ => self.size_of(),
        }
    }
}

pub trait Datum:
    Clone + Send + Sync + fmt::Debug + fmt::Display + Default + 'static + PartialEq
{
    fn name() -> &'static str;
    fn datum_type() -> DatumType;

    fn packed_mat_mul(m: usize, k: usize, n: usize) -> Option<Box<tract_linalg::MatMul<Self>>>;
}

pub(crate) trait TryInto<D: Datum> {
    fn try_into(&self) -> TractResult<D>;
}

macro_rules! datum {
    ($t:ident, $v:ident) => {
        datum!($t, $v, |_, _, _| None);
    };
    ($t:ident, $v:ident, $matmul:expr) => {
        impl From<$t> for Tensor {
            fn from(it: $t) -> Tensor {
                arr0(it).into()
            }
        }

        impl Datum for $t {
            fn name() -> &'static str {
                stringify!($t)
            }

            fn datum_type() -> DatumType {
                DatumType::$v
            }

            fn packed_mat_mul(
                m: usize,
                k: usize,
                n: usize,
            ) -> Option<Box<tract_linalg::MatMul<Self>>> {
                $matmul(m, k, n)
            }
        }
    };
}

macro_rules! try_into {
    ($f:ty, $t:ty) => {
        impl TryInto<$t> for $f {
            fn try_into(&self) -> TractResult<$t> {
                Ok(*self as $t)
            }
        }
    };
}

try_into!(i8, i16);
try_into!(i8, i32);
try_into!(i8, i64);
try_into!(i16, i32);
try_into!(i16, i64);
try_into!(i32, i64);

try_into!(i16, i8);
try_into!(i32, i8);
try_into!(i64, i8);
try_into!(i32, i16);
try_into!(i64, i16);
try_into!(i64, i32);

try_into!(f64, f32);
try_into!(f32, f64);

try_into!(i8, f32);
try_into!(i16, f32);
try_into!(i32, f32);
try_into!(i64, f32);

impl TryInto<TDim> for i32 {
    fn try_into(&self) -> TractResult<TDim> {
        Ok((*self).into())
    }
}

impl TryInto<i32> for TDim {
    fn try_into(&self) -> TractResult<i32> {
        self.to_integer().map(|i| i as i32)
    }
}

impl TryInto<i64> for TDim {
    fn try_into(&self) -> TractResult<i64> {
        self.to_integer().map(|i| i as i64)
    }
}

impl TryInto<f32> for bool {
    fn try_into(&self) -> TractResult<f32> {
        if *self {
            Ok(1.0)
        } else {
            Ok(0.0)
        }
    }
}

impl TryInto<f32> for f16 {
    fn try_into(&self) -> TractResult<f32> {
        Ok(self.0.to_f32())
    }
}

impl TryInto<f64> for f16 {
    fn try_into(&self) -> TractResult<f64> {
        Ok(self.0.to_f64())
    }
}

impl TryInto<f16> for f32 {
    fn try_into(&self) -> TractResult<f16> {
        Ok(f16(half::f16::from_f32(*self)))
    }
}

impl TryInto<f16> for f64 {
    fn try_into(&self) -> TractResult<f16> {
        Ok(f16(half::f16::from_f64(*self)))
    }
}

impl TryInto<String> for f32 {
    fn try_into(&self) -> TractResult<String> {
        Ok(self.to_string())
    }
}

impl TryInto<f32> for String {
    fn try_into(&self) -> TractResult<f32> {
        // this is onnx casts
        if self == "INF" || self == "+INF" {
            Ok(std::f32::INFINITY)
        } else if self == "-INF" {
            Ok(-std::f32::INFINITY)
        } else {
            Ok(self.parse::<f32>().map_err(|_| format!("Can not parse {} as f32", self))?)
        }
    }
}

datum!(bool, Bool);
datum!(f16, F16, |m, k, n| Some(
    Box::new(NdArrayDummyPackedMatMul::new(m, k, n)) as _
));
datum!(f32, F32, |m, k, n| if m != 1 {
    Some((tract_linalg::ops().smm)(m, k, n))
} else {
    Some(Box::new(NdArrayDummyPackedMatMul1xKxN::new(k, n)) as _)
});
datum!(f64, F64, |m, k, n| Some((tract_linalg::ops().dmm)(m, k, n)));
datum!(i8, I8);
datum!(i16, I16);
datum!(i32, I32);
datum!(i64, I64);
datum!(u8, U8);
datum!(u16, U16);
datum!(TDim, TDim);
datum!(String, String);

#[cfg(test)]
mod tests {
    use crate::datum::*;
    use crate::dim::ToDim;

    #[test]
    fn test_array_to_tensor_to_array() {
        let array = arr1(&[12i32, 42]);
        let dt_array = Tensor::from(array.clone());
        let view = dt_array.to_array_view::<i32>().unwrap();
        assert_eq!(array, view.into_dimensionality().unwrap());
    }

    #[test]
    fn test_cast_dim_to_dim() {
        let t_dim: Tensor = arr1(&[12isize.to_dim(), 42isize.to_dim()]).into();
        let t_i32 = t_dim.cast_to::<i32>().unwrap();
        let t_dim_2 = t_i32.cast_to::<TDim>().unwrap().into_owned();
        assert_eq!(t_dim, t_dim_2);
    }

    #[test]
    fn test_cast_i32_to_dim() {
        let t_i32: Tensor = arr1(&[0i32, 0]).into();
        t_i32.cast_to::<TDim>().unwrap();
    }
}
