use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{Chunk, ChunkAssetArgs, ChunkUkey, Compilation};

use crate::{JsCompilation, JsModule, ToJsModule};

#[napi(object)]
pub struct JsChunk {
  #[napi(js_name = "__inner_ukey")]
  pub inner_ukey: u32, // ChunkUkey
  #[napi(js_name = "__inner_groups")]
  pub inner_groups: Vec<u32>,
  pub name: Option<String>,
  pub id: Option<String>,
  pub ids: Vec<String>,
  pub id_name_hints: Vec<String>,
  pub filename_template: Option<String>,
  pub css_filename_template: Option<String>,
  pub files: Vec<String>,
  pub runtime: Vec<String>,
  pub hash: Option<String>,
  pub content_hash: HashMap<String, String>,
  pub rendered_hash: Option<String>,
  pub chunk_reasons: Vec<String>,
  pub auxiliary_files: Vec<String>,
}

impl JsChunk {
  pub fn from(chunk: &rspack_core::Chunk) -> Self {
    let Chunk {
      // not implement yet
      ukey: _ukey,
      prevent_integration: _prevent_integration,
      groups: _groups,
      kind: _kind,

      // used in js chunk
      name,
      filename_template,
      css_filename_template,
      id,
      ids,
      id_name_hints,
      files,
      auxiliary_files,
      runtime,
      hash,
      rendered_hash,
      content_hash,
      chunk_reasons,
    } = chunk;
    let mut files = Vec::from_iter(files.iter().cloned());
    files.sort_unstable();
    let mut auxiliary_files = auxiliary_files.iter().cloned().collect::<Vec<_>>();
    auxiliary_files.sort_unstable();
    let mut runtime = Vec::<String>::from_iter(runtime.clone().into_iter().map(|r| r.to_string()));
    runtime.sort_unstable();

    Self {
      inner_ukey: usize::from(chunk.ukey) as u32,
      inner_groups: chunk
        .groups
        .iter()
        .map(|ukey| ukey.as_usize() as u32)
        .collect(),
      name: name.clone(),
      id: id.clone(),
      ids: ids.clone(),
      id_name_hints: Vec::from_iter(id_name_hints.clone()),
      filename_template: filename_template
        .as_ref()
        .map(|tpl| tpl.template().to_string()),
      css_filename_template: css_filename_template
        .as_ref()
        .map(|tpl| tpl.template().to_string()),
      files,
      runtime,
      hash: hash.as_ref().map(|d| d.encoded().to_string()),
      content_hash: content_hash
        .iter()
        .map(|(key, v)| (key.to_string(), v.encoded().to_string()))
        .collect::<std::collections::HashMap<String, String>>(),
      rendered_hash: rendered_hash.as_ref().map(|hash| hash.to_string()),
      chunk_reasons: chunk_reasons.clone(),
      auxiliary_files,
    }
  }
}

fn chunk(ukey: u32, compilation: &Compilation) -> &Chunk {
  let ukey = ChunkUkey::from(ukey as usize);
  compilation
    .chunk_by_ukey
    .get(&ukey)
    .expect("Chunk must exist")
}

#[napi(js_name = "__chunk_inner_is_only_initial")]
pub fn is_only_initial(js_chunk_ukey: u32, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.inner;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.is_only_initial(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_can_be_initial")]
pub fn can_be_initial(js_chunk_ukey: u32, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.inner;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.can_be_initial(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_has_runtime")]
pub fn has_runtime(js_chunk_ukey: u32, compilation: &JsCompilation) -> bool {
  let compilation = &compilation.inner;
  let chunk = chunk(js_chunk_ukey, compilation);
  chunk.has_runtime(&compilation.chunk_group_by_ukey)
}

#[napi(js_name = "__chunk_inner_get_chunk_modules")]
pub fn get_chunk_modules(js_chunk_ukey: u32, compilation: &JsCompilation) -> Vec<JsModule> {
  let compilation = &compilation.inner;
  let modules = compilation.chunk_graph.get_chunk_modules(
    &ChunkUkey::from(js_chunk_ukey as usize),
    &compilation.module_graph,
  );

  return modules
    .iter()
    .filter_map(|module| module.to_js_module().ok())
    .collect::<Vec<_>>();
}

#[napi(js_name = "__chunk_inner_get_chunk_entry_modules")]
pub fn get_chunk_entry_modules(js_chunk_ukey: u32, compilation: &JsCompilation) -> Vec<JsModule> {
  let compilation = &compilation.inner;
  let modules = compilation
    .chunk_graph
    .get_chunk_entry_modules(&ChunkUkey::from(js_chunk_ukey as usize));

  return modules
    .iter()
    .filter_map(|module| compilation.module_graph.module_by_identifier(module))
    .filter_map(|module| module.to_js_module().ok())
    .collect::<Vec<_>>();
}

#[napi(object)]
pub struct JsChunkAssetArgs {
  pub chunk: JsChunk,
  pub filename: String,
}

impl From<&ChunkAssetArgs<'_>> for JsChunkAssetArgs {
  fn from(value: &ChunkAssetArgs) -> Self {
    Self {
      chunk: JsChunk::from(value.chunk),
      filename: value.filename.to_string(),
    }
  }
}
