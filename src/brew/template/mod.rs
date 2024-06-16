use anyhow::Result;
use handlebars::{
    Context, Handlebars, Helper, Output, RenderContext, RenderError, RenderErrorReason,
};

pub const FORMULA_FILE_TEMPLATE: &str = "default_template";

pub fn handlebars<'hb>() -> Result<Handlebars<'hb>> {
    let mut hb = Handlebars::new();

    let single_target = include_str!("./default_template.hbs");

    hb.register_template_string(FORMULA_FILE_TEMPLATE, single_target)?;

    hb.register_helper("one_or_many", Box::new(one_or_many_helper));

    Ok(hb)
}

fn one_or_many_helper(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let param = h.param(0).and_then(|v| v.value().as_array()).ok_or(
        RenderErrorReason::ParamTypeMismatchForName(
            "symbol",
            "symbol".to_owned(),
            "str".to_owned(),
        ),
    )?;

    if param.len() == 1 {
        let single = &param[0];
        out.write(
            single
                .as_str()
                .ok_or(RenderErrorReason::ParamTypeMismatchForName(
                    "symbol",
                    "symbol".to_owned(),
                    "str".to_owned(),
                ))?,
        )?;
    } else {
        out.write(&format!(
            "[{}]",
            param
                .iter()
                .map(|v| v
                    .as_str()
                    .ok_or(RenderErrorReason::ParamTypeMismatchForName(
                        "symbol",
                        "symbol".to_owned(),
                        "str".to_owned(),
                    )))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ")
        ))?;
    }
    Ok(())
}
