// This file is generated. Do not edit
// @generated

// https://github.com/Manishearth/rust-clippy/issues/702



use protobuf::Message as Message_imported_for_functions;
use protobuf::ProtobufEnum as ProtobufEnum_imported_for_functions;

#[derive(PartialEq,Clone,Default)]
pub struct NodeDef {
    // message fields
    pub name: ::std::string::String,
    pub op: ::std::string::String,
    pub input: ::protobuf::RepeatedField<::std::string::String>,
    pub device: ::std::string::String,
    pub attr: ::std::collections::HashMap<::std::string::String, super::attr_value::AttrValue>,
    // special fields
    unknown_fields: ::protobuf::UnknownFields,
    cached_size: ::protobuf::CachedSize,
}

// see codegen.rs for the explanation why impl Sync explicitly
unsafe impl ::std::marker::Sync for NodeDef {}

impl NodeDef {
    pub fn new() -> NodeDef {
        ::std::default::Default::default()
    }

    pub fn default_instance() -> &'static NodeDef {
        static mut instance: ::protobuf::lazy::Lazy<NodeDef> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const NodeDef,
        };
        unsafe {
            instance.get(NodeDef::new)
        }
    }

    // string name = 1;

    pub fn clear_name(&mut self) {
        self.name.clear();
    }

    // Param is passed by value, moved
    pub fn set_name(&mut self, v: ::std::string::String) {
        self.name = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_name(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // Take field
    pub fn take_name(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.name, ::std::string::String::new())
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    fn get_name_for_reflect(&self) -> &::std::string::String {
        &self.name
    }

    fn mut_name_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.name
    }

    // string op = 2;

    pub fn clear_op(&mut self) {
        self.op.clear();
    }

    // Param is passed by value, moved
    pub fn set_op(&mut self, v: ::std::string::String) {
        self.op = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_op(&mut self) -> &mut ::std::string::String {
        &mut self.op
    }

    // Take field
    pub fn take_op(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.op, ::std::string::String::new())
    }

    pub fn get_op(&self) -> &str {
        &self.op
    }

    fn get_op_for_reflect(&self) -> &::std::string::String {
        &self.op
    }

    fn mut_op_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.op
    }

    // repeated string input = 3;

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    // Param is passed by value, moved
    pub fn set_input(&mut self, v: ::protobuf::RepeatedField<::std::string::String>) {
        self.input = v;
    }

    // Mutable pointer to the field.
    pub fn mut_input(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.input
    }

    // Take field
    pub fn take_input(&mut self) -> ::protobuf::RepeatedField<::std::string::String> {
        ::std::mem::replace(&mut self.input, ::protobuf::RepeatedField::new())
    }

    pub fn get_input(&self) -> &[::std::string::String] {
        &self.input
    }

    fn get_input_for_reflect(&self) -> &::protobuf::RepeatedField<::std::string::String> {
        &self.input
    }

    fn mut_input_for_reflect(&mut self) -> &mut ::protobuf::RepeatedField<::std::string::String> {
        &mut self.input
    }

    // string device = 4;

    pub fn clear_device(&mut self) {
        self.device.clear();
    }

    // Param is passed by value, moved
    pub fn set_device(&mut self, v: ::std::string::String) {
        self.device = v;
    }

    // Mutable pointer to the field.
    // If field is not initialized, it is initialized with default value first.
    pub fn mut_device(&mut self) -> &mut ::std::string::String {
        &mut self.device
    }

    // Take field
    pub fn take_device(&mut self) -> ::std::string::String {
        ::std::mem::replace(&mut self.device, ::std::string::String::new())
    }

    pub fn get_device(&self) -> &str {
        &self.device
    }

    fn get_device_for_reflect(&self) -> &::std::string::String {
        &self.device
    }

    fn mut_device_for_reflect(&mut self) -> &mut ::std::string::String {
        &mut self.device
    }

    // repeated .tensorflow.NodeDef.AttrEntry attr = 5;

    pub fn clear_attr(&mut self) {
        self.attr.clear();
    }

    // Param is passed by value, moved
    pub fn set_attr(&mut self, v: ::std::collections::HashMap<::std::string::String, super::attr_value::AttrValue>) {
        self.attr = v;
    }

    // Mutable pointer to the field.
    pub fn mut_attr(&mut self) -> &mut ::std::collections::HashMap<::std::string::String, super::attr_value::AttrValue> {
        &mut self.attr
    }

    // Take field
    pub fn take_attr(&mut self) -> ::std::collections::HashMap<::std::string::String, super::attr_value::AttrValue> {
        ::std::mem::replace(&mut self.attr, ::std::collections::HashMap::new())
    }

    pub fn get_attr(&self) -> &::std::collections::HashMap<::std::string::String, super::attr_value::AttrValue> {
        &self.attr
    }

    fn get_attr_for_reflect(&self) -> &::std::collections::HashMap<::std::string::String, super::attr_value::AttrValue> {
        &self.attr
    }

    fn mut_attr_for_reflect(&mut self) -> &mut ::std::collections::HashMap<::std::string::String, super::attr_value::AttrValue> {
        &mut self.attr
    }
}

impl ::protobuf::Message for NodeDef {
    fn is_initialized(&self) -> bool {
        true
    }

    fn merge_from(&mut self, is: &mut ::protobuf::CodedInputStream) -> ::protobuf::ProtobufResult<()> {
        while !is.eof()? {
            let (field_number, wire_type) = is.read_tag_unpack()?;
            match field_number {
                1 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.name)?;
                },
                2 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.op)?;
                },
                3 => {
                    ::protobuf::rt::read_repeated_string_into(wire_type, is, &mut self.input)?;
                },
                4 => {
                    ::protobuf::rt::read_singular_proto3_string_into(wire_type, is, &mut self.device)?;
                },
                5 => {
                    ::protobuf::rt::read_map_into::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeMessage<super::attr_value::AttrValue>>(wire_type, is, &mut self.attr)?;
                },
                _ => {
                    ::protobuf::rt::read_unknown_or_skip_group(field_number, wire_type, is, self.mut_unknown_fields())?;
                },
            };
        }
        ::std::result::Result::Ok(())
    }

    // Compute sizes of nested messages
    #[allow(unused_variables)]
    fn compute_size(&self) -> u32 {
        let mut my_size = 0;
        if !self.name.is_empty() {
            my_size += ::protobuf::rt::string_size(1, &self.name);
        }
        if !self.op.is_empty() {
            my_size += ::protobuf::rt::string_size(2, &self.op);
        }
        for value in &self.input {
            my_size += ::protobuf::rt::string_size(3, &value);
        };
        if !self.device.is_empty() {
            my_size += ::protobuf::rt::string_size(4, &self.device);
        }
        my_size += ::protobuf::rt::compute_map_size::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeMessage<super::attr_value::AttrValue>>(5, &self.attr);
        my_size += ::protobuf::rt::unknown_fields_size(self.get_unknown_fields());
        self.cached_size.set(my_size);
        my_size
    }

    fn write_to_with_cached_sizes(&self, os: &mut ::protobuf::CodedOutputStream) -> ::protobuf::ProtobufResult<()> {
        if !self.name.is_empty() {
            os.write_string(1, &self.name)?;
        }
        if !self.op.is_empty() {
            os.write_string(2, &self.op)?;
        }
        for v in &self.input {
            os.write_string(3, &v)?;
        };
        if !self.device.is_empty() {
            os.write_string(4, &self.device)?;
        }
        ::protobuf::rt::write_map_with_cached_sizes::<::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeMessage<super::attr_value::AttrValue>>(5, &self.attr, os)?;
        os.write_unknown_fields(self.get_unknown_fields())?;
        ::std::result::Result::Ok(())
    }

    fn get_cached_size(&self) -> u32 {
        self.cached_size.get()
    }

    fn get_unknown_fields(&self) -> &::protobuf::UnknownFields {
        &self.unknown_fields
    }

    fn mut_unknown_fields(&mut self) -> &mut ::protobuf::UnknownFields {
        &mut self.unknown_fields
    }

    fn as_any(&self) -> &::std::any::Any {
        self as &::std::any::Any
    }
    fn as_any_mut(&mut self) -> &mut ::std::any::Any {
        self as &mut ::std::any::Any
    }
    fn into_any(self: Box<Self>) -> ::std::boxed::Box<::std::any::Any> {
        self
    }

    fn descriptor(&self) -> &'static ::protobuf::reflect::MessageDescriptor {
        ::protobuf::MessageStatic::descriptor_static(None::<Self>)
    }
}

impl ::protobuf::MessageStatic for NodeDef {
    fn new() -> NodeDef {
        NodeDef::new()
    }

    fn descriptor_static(_: ::std::option::Option<NodeDef>) -> &'static ::protobuf::reflect::MessageDescriptor {
        static mut descriptor: ::protobuf::lazy::Lazy<::protobuf::reflect::MessageDescriptor> = ::protobuf::lazy::Lazy {
            lock: ::protobuf::lazy::ONCE_INIT,
            ptr: 0 as *const ::protobuf::reflect::MessageDescriptor,
        };
        unsafe {
            descriptor.get(|| {
                let mut fields = ::std::vec::Vec::new();
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "name",
                    NodeDef::get_name_for_reflect,
                    NodeDef::mut_name_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "op",
                    NodeDef::get_op_for_reflect,
                    NodeDef::mut_op_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_repeated_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "input",
                    NodeDef::get_input_for_reflect,
                    NodeDef::mut_input_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_simple_field_accessor::<_, ::protobuf::types::ProtobufTypeString>(
                    "device",
                    NodeDef::get_device_for_reflect,
                    NodeDef::mut_device_for_reflect,
                ));
                fields.push(::protobuf::reflect::accessor::make_map_accessor::<_, ::protobuf::types::ProtobufTypeString, ::protobuf::types::ProtobufTypeMessage<super::attr_value::AttrValue>>(
                    "attr",
                    NodeDef::get_attr_for_reflect,
                    NodeDef::mut_attr_for_reflect,
                ));
                ::protobuf::reflect::MessageDescriptor::new::<NodeDef>(
                    "NodeDef",
                    fields,
                    file_descriptor_proto()
                )
            })
        }
    }
}

impl ::protobuf::Clear for NodeDef {
    fn clear(&mut self) {
        self.clear_name();
        self.clear_op();
        self.clear_input();
        self.clear_device();
        self.clear_attr();
        self.unknown_fields.clear();
    }
}

impl ::std::fmt::Debug for NodeDef {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        ::protobuf::text_format::fmt(self, f)
    }
}

impl ::protobuf::reflect::ProtobufValue for NodeDef {
    fn as_ref(&self) -> ::protobuf::reflect::ProtobufValueRef {
        ::protobuf::reflect::ProtobufValueRef::Message(self)
    }
}

static file_descriptor_proto_data: &'static [u8] = b"\
    \n(tensorflow/core/framework/node_def.proto\x12\ntensorflow\x1a*tensorfl\
    ow/core/framework/attr_value.proto\"\xde\x01\n\x07NodeDef\x12\x12\n\x04n\
    ame\x18\x01\x20\x01(\tR\x04name\x12\x0e\n\x02op\x18\x02\x20\x01(\tR\x02o\
    p\x12\x14\n\x05input\x18\x03\x20\x03(\tR\x05input\x12\x16\n\x06device\
    \x18\x04\x20\x01(\tR\x06device\x121\n\x04attr\x18\x05\x20\x03(\x0b2\x1d.\
    tensorflow.NodeDef.AttrEntryR\x04attr\x1aN\n\tAttrEntry\x12\x10\n\x03key\
    \x18\x01\x20\x01(\tR\x03key\x12+\n\x05value\x18\x02\x20\x01(\x0b2\x15.te\
    nsorflow.AttrValueR\x05value:\x028\x01B*\n\x18org.tensorflow.frameworkB\
    \tNodeProtoP\x01\xf8\x01\x01b\x06proto3\
";

static mut file_descriptor_proto_lazy: ::protobuf::lazy::Lazy<::protobuf::descriptor::FileDescriptorProto> = ::protobuf::lazy::Lazy {
    lock: ::protobuf::lazy::ONCE_INIT,
    ptr: 0 as *const ::protobuf::descriptor::FileDescriptorProto,
};

fn parse_descriptor_proto() -> ::protobuf::descriptor::FileDescriptorProto {
    ::protobuf::parse_from_bytes(file_descriptor_proto_data).unwrap()
}

pub fn file_descriptor_proto() -> &'static ::protobuf::descriptor::FileDescriptorProto {
    unsafe {
        file_descriptor_proto_lazy.get(|| {
            parse_descriptor_proto()
        })
    }
}
