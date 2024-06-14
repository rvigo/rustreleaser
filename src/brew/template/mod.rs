use anyhow::Result;
use handlebars::{handlebars_helper, Handlebars};

pub const FORMULA_FILE_TEMPLATE: &str = "default_template";

pub fn handlebars<'hb>() -> Result<Handlebars<'hb>> {
    let mut hb = Handlebars::new();

    let single_target = include_str!("./default_template.hbs");

    hb.register_template_string(FORMULA_FILE_TEMPLATE, single_target)?;

    handlebars_helper!(eq: |this: str, other: str| this.eq(other));

    hb.register_helper("eq", Box::new(eq));

    Ok(hb)
}
