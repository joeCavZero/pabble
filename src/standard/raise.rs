use penguin::prelude::*;

pub fn setup(peng: &mut PengEnv, unit: &mut PengUnit) -> Result<(), PengError> {
    match unit.register_immutable_native_function(peng, "raise", |ctx| {
        let mut values = Vec::new();

        let mut index = 0;

        loop {
            let value = match ctx.get_arg_value(index) {
                Some(value) => (*value.value()).clone(),

                None => {
                    break;
                }
            };

            values.push(value);

            index += 1;
        }

        Err(PengError::Raised(Box::new(PengError::UserError(values))))
    }) {
        Ok(_) => Ok(()),

        Err(e) => Err(e),
    }
}
