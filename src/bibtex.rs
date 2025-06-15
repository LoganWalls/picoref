use rquickjs::{CatchResultExt, Context, Runtime};
use serde::Serialize;

use crate::entry::EntryData;

const JS_BUNDLE: &str = include_str!("../js/dist/bibtex-converter.bundle.js");

pub fn to_bibtex(
    entries: impl IntoIterator<Item = EntryData> + Serialize,
) -> anyhow::Result<String> {
    let csl_json = serde_json::to_string(&entries)?;
    let rt = Runtime::new().unwrap();
    let ctx = Context::full(&rt).unwrap();

    ctx.with(|ctx| {
        // Load the JS bundle
        ctx.eval::<(), _>(JS_BUNDLE)
            .catch(&ctx)
            .map_err(|e| anyhow::anyhow!("JavaScript error: {}", e))?;
        // Run the conversion
        ctx.eval::<(), _>(format!("globalThis.inputJson = {csl_json};"))
            .catch(&ctx)
            .map_err(|e| anyhow::anyhow!("JavaScript error: {}", e))?;
        ctx.eval::<String, _>(
            r#"
                    try {
                        BibtexConverter.cslToBibtex(globalThis.inputJson);
                    } catch (e) {
                        JSON.stringify({
                            success: false,
                            error: e.message,
                            stack: e.stack,
                            name: e.name
                        });
                    }
                "#,
        )
        .catch(&ctx)
        .map_err(|e| anyhow::anyhow!("JavaScript error: {}", e))
    })
}

// static BUNDLE: Bundle = embed! { "bibtexConverter": "js/dist/bibtex-converter.bundle.js" };
// pub fn convert_csl(csl_json: &str) -> anyhow::Result<String> {
//     let rt = Runtime::new().unwrap();
//     let ctx = Context::full(&rt).unwrap();
//     rt.set_loader(BUNDLE, BUNDLE);
//
//     ctx.with(|ctx| {
//         let module = Module::import(&ctx, "bibtexConverter")?;
//         let csl_to_bibtex: rquickjs::Function = module.get("cslToBibtex")?;
//         csl_to_bibtex
//             .call((csl_json,))
//             .catch(&ctx)
//             .map_err(|e| anyhow::anyhow!("JavaScript error: {}", e))
//     })
// }
