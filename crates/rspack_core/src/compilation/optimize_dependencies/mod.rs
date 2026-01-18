use super::*;
use crate::logger::Logger;

pub async fn optimize_dependencies_pass(
  compilation: &mut Compilation,
  plugin_driver: SharedPluginDriver,
) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compilation");
  let start = logger.time("optimize dependencies");
  // https://github.com/webpack/webpack/blob/d15c73469fd71cf98734685225250148b68ddc79/lib/Compilation.js#L2812-L2814

  let mut diagnostics: Vec<Diagnostic> = vec![];
  // Take the artifacts and immediately replace with defaults to prevent panics
  // when JS plugins access compilation.get_module_graph() during the hook.
  let mut side_effects_optimize_artifact = compilation.side_effects_optimize_artifact.take();
  compilation
    .side_effects_optimize_artifact
    .replace(Default::default());
  let mut build_module_graph_artifact = compilation.build_module_graph_artifact.take();
  compilation
    .build_module_graph_artifact
    .replace(Default::default());

  // Store the result instead of using ? to ensure artifacts are always restored
  let result: Result<()> = async {
    while matches!(
      plugin_driver
        .compilation_hooks
        .optimize_dependencies
        .call(
          compilation,
          &mut side_effects_optimize_artifact,
          &mut build_module_graph_artifact,
          &mut diagnostics
        )
        .await
        .map_err(|e| e.wrap_err("caused by plugins in Compilation.hooks.optimizeDependencies"))?,
      Some(true)
    ) {}
    Ok(())
  }
  .await;

  // Restore the real artifacts (replacing the defaults)
  compilation
    .side_effects_optimize_artifact
    .replace(side_effects_optimize_artifact);
  compilation
    .build_module_graph_artifact
    .replace(build_module_graph_artifact);
  compilation.extend_diagnostics(diagnostics);

  // Propagate the error after restoring artifacts
  result?;

  logger.time_end(start);
  Ok(())
}
