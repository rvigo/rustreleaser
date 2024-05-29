use crate::brew::target::{Target, Targets};
use anyhow::Result;
use handlebars::{handlebars_helper, Handlebars};
use serde::{Deserialize, Serialize};

pub fn handlebars<'hb>() -> Result<Handlebars<'hb>> {
    let mut hb = Handlebars::new();

    let multi_target = include_str!("./multi_target.hbs");
    let single_target = include_str!("./single_target.hbs");

    hb.register_template_string("multi_target", multi_target)?;
    hb.register_template_string("single_target", single_target)?;

    handlebars_helper!(eq: |this: str, other: str| this.eq(other));

    hb.register_helper("eq", Box::new(eq));

    Ok(hb)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Template {
    MultiTarget,
    SingleTarget,
}

impl ToString for Template {
    fn to_string(&self) -> String {
        match self {
            Template::MultiTarget => "multi_target".to_string(),
            Template::SingleTarget => "single_target".to_string(),
        }
    }
}

impl From<&Targets> for Template {
    fn from(target: &Targets) -> Self {
        match target.inner_type() {
            Target::Single(_) => Template::SingleTarget,
            Target::Multi(_) => Template::MultiTarget,
        }
    }
}
