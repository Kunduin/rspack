use std::sync::Arc;

use futures::Future;
use rspack_identifier::Identifier;

use crate::{
  cache::snapshot::{Snapshot, SnapshotManager},
  cache::storage,
  ModuleIdentifier, ResolveArgs, ResolveError, ResolveResult,
};

type Storage = dyn storage::Storage<(Snapshot, ResolveResult)>;

#[derive(Debug)]
pub struct ResolveModuleOccasion {
  storage: Option<Box<Storage>>,
  snapshot_manager: Arc<SnapshotManager>,
}

impl ResolveModuleOccasion {
  pub fn new(storage: Option<Box<Storage>>, snapshot_manager: Arc<SnapshotManager>) -> Self {
    Self {
      storage,
      snapshot_manager,
    }
  }

  pub async fn use_cache<'a, G, F>(
    &self,
    args: ResolveArgs<'a>,
    generator: G,
  ) -> Result<(Result<ResolveResult, ResolveError>, bool), ResolveError>
  where
    G: Fn(ResolveArgs<'a>) -> F,
    F: Future<Output = Result<ResolveResult, ResolveError>>,
  {
    let storage = match &self.storage {
      Some(s) => s,
      // no cache return directly
      None => return Ok((generator(args).await, false)),
    };

    let id = ModuleIdentifier::from(format!(
      "{:?}|{}|{}|{:?}",
      args.context,
      args
        .importer
        .copied()
        .unwrap_or_else(|| Identifier::from("")),
      args.specifier,
      args.dependency_type
    ));
    {
      // read
      if let Some((snapshot, data)) = storage.get(&id) {
        let valid = self
          .snapshot_manager
          .check_snapshot_valid(&snapshot)
          .await
          .unwrap_or(false);

        if valid {
          return Ok((Ok(data), true));
        }
      };
    }

    // run generator and save to cache
    let data = generator(args).await?;
    let mut paths = Vec::new();
    if let ResolveResult::Resource(resource) = &data {
      paths.push(resource.path.as_path());
    }

    let snapshot = self
      .snapshot_manager
      .create_snapshot(&paths, |option| &option.resolve)
      .await
      .map_err(|err| ResolveError(err.to_string(), err))?;
    storage.set(id, (snapshot, data.clone()));
    Ok((Ok(data), false))
  }
}
