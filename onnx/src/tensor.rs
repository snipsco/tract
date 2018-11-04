use pb::*;
use tract_core::f16::f16;
use tract_core::*;

impl Tractify<TensorProto_DataType> for DatumType {
    fn tractify(t: &TensorProto_DataType) -> TractResult<DatumType> {
        use self::TensorProto_DataType::*;
        match t {
            &BOOL => Ok(DatumType::Bool),
            &UINT8 => Ok(DatumType::U8),
            &UINT16 => Ok(DatumType::U16),
            &INT8 => Ok(DatumType::I8),
            &INT16 => Ok(DatumType::I16),
            &INT32 => Ok(DatumType::I32),
            &INT64 => Ok(DatumType::I64),
            &FLOAT16 => Ok(DatumType::F16),
            &FLOAT => Ok(DatumType::F32),
            &DOUBLE => Ok(DatumType::F64),
            &STRING => Ok(DatumType::String),
            _ => Err(format!("Unknown DatumType {:?}", t))?,
        }
    }

    /*
    fn to_onnx(&self) -> TractResult<TensorProto_DataType> {
        use self::TensorProto_DataType::*;
        match self {
            DatumType::U8 => Ok(UINT8),
            DatumType::I8 => Ok(INT8),
            DatumType::I32 => Ok(INT32),
            DatumType::F32 => Ok(FLOAT),
            DatumType::F64 => Ok(DOUBLE),
            DatumType::String => Ok(STRING),
            DatumType::TDim => bail!("Dimension is not translatable in protobuf"),
        }
    }
    */
}

impl Tractify<TypeProto_Tensor> for TensorFact {
    fn tractify(t: &TypeProto_Tensor) -> TractResult<TensorFact> {
        let mut fact = TensorFact::default();
        if t.has_elem_type() {
            fact = fact.with_datum_type(t.get_elem_type().tractify()?);
        }
        if t.has_shape() {
            let shape = t.get_shape();
            let shape: Vec<usize> = shape
                .get_dim()
                .iter()
                .map(|d| d.get_dim_value() as usize)
                .collect();
            fact = fact.with_shape(shape)
        }
        Ok(fact)
    }
}

impl Tractify<TensorProto> for DtArray {
    fn tractify(t: &TensorProto) -> TractResult<DtArray> {
        let dt = t.get_data_type().tractify()?;
        let shape: Vec<usize> = t.get_dims().iter().map(|&i| i as usize).collect();
        if t.has_raw_data() {
            unsafe {
                match dt {
                    DatumType::U8 => DtArray::from_raw::<u8>(&*shape, t.get_raw_data()),
                    DatumType::U16 => DtArray::from_raw::<u16>(&*shape, t.get_raw_data()),
                    DatumType::I8 => DtArray::from_raw::<i8>(&*shape, t.get_raw_data()),
                    DatumType::I16 => DtArray::from_raw::<i16>(&*shape, t.get_raw_data()),
                    DatumType::I32 => DtArray::from_raw::<i32>(&*shape, t.get_raw_data()),
                    DatumType::I64 => DtArray::from_raw::<i64>(&*shape, t.get_raw_data()),
                    DatumType::F16 => DtArray::from_raw::<f16>(&*shape, t.get_raw_data()),
                    DatumType::F32 => DtArray::from_raw::<f32>(&*shape, t.get_raw_data()),
                    DatumType::F64 => DtArray::from_raw::<f64>(&*shape, t.get_raw_data()),
                    DatumType::Bool => Ok(DtArray::from_raw::<u8>(&*shape, t.get_raw_data())?
                        .into_array::<u8>()?
                        .mapv(|x| x != 0)
                        .into()),
                    _ => unimplemented!("FIXME, tensor loading"),
                }
            }
        } else {
            match dt {
                DatumType::Bool => Ok(DtArray::i32s(&*shape, t.get_int32_data())?
                    .into_array::<i32>()?
                    .mapv(|x| x != 0)
                    .into()),
                DatumType::U8 => DtArray::i32s(&*shape, t.get_int32_data())?.cast_to::<u8>(),
                DatumType::U16 => DtArray::i32s(&*shape, t.get_int32_data())?.cast_to::<u16>(),
                DatumType::I8 => DtArray::i32s(&*shape, t.get_int32_data())?.cast_to::<i8>(),
                DatumType::I16 => DtArray::i32s(&*shape, t.get_int32_data())?.cast_to::<i16>(),
                DatumType::I32 => DtArray::i32s(&*shape, t.get_int32_data()),
                DatumType::I64 => DtArray::i64s(&*shape, t.get_int64_data()),
                DatumType::F32 => DtArray::f32s(&*shape, t.get_float_data()),
                DatumType::F64 => DtArray::f64s(&*shape, t.get_double_data()),
                _ => unimplemented!("FIXME, tensor loading"),
            }
        }
    }
}

pub fn from_reader<R: ::std::io::Read>(mut r: R) -> TractResult<DtArray> {
    let tensor: TensorProto = ::protobuf::parse_from_reader(&mut r).unwrap();
    tensor.tractify()
}
