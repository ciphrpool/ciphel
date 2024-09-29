#[macro_export]
macro_rules! e_static {
    ($type_def:expr) => {
        crate::semantic::EType::Static($type_def.into())
    };
}

#[macro_export]
macro_rules! e_user {
    ($type_def:expr) => {
        crate::semantic::EType::User($type_def)
    };
}

#[macro_export]
macro_rules! p_num {
    ($num:ident) => {
        crate::semantic::EType::Static(crate::semantic::scope::static_types::StaticType::Primitive(
            crate::semantic::scope::static_types::PrimitiveType::Number(
                crate::semantic::scope::static_types::NumberType::$num,
            ),
        ))
    };
}

#[macro_export]
macro_rules! v_num {
    ($type_def:ident,$num:expr) => {
        crate::ast::expressions::data::Primitive::Number(
            crate::ast::expressions::data::Number::$type_def($num),
        )
    };
}

#[macro_export]
macro_rules! err_tuple {
    ($value:expr) => {
        e_static!(crate::semantic::scope::static_types::StaticType::Tuple(
            crate::semantic::scope::static_types::TupleType(vec![
                $value,
                e_static!(StaticType::Error)
            ])
        ))
    };
}

#[macro_export]
macro_rules! clear_stack {
    ($memory:ident) => {{
        use num_traits::Zero;
        let top = $memory.top();
        let data = $memory
            .pop(top)
            .expect("Read should have succeeded")
            .to_owned();
        assert!($memory.top().is_zero());
        data
    }};
}
