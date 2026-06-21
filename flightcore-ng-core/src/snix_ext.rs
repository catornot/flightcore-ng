use std::{
    hash::Hash,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use nix_compat::flakeref::FlakeRef;
use snix_eval::{NixAttrs, NixList, Value};

pub trait ValueExt {
    fn get_attr(&self) -> Option<&NixAttrs>;
    fn get_list(&self) -> Option<&NixList>;
}

impl ValueExt for Value {
    fn get_attr(&self) -> Option<&NixAttrs> {
        match self {
            Value::Attrs(nix_attrs) => Some(nix_attrs),
            _ => None,
        }
    }

    fn get_list(&self) -> Option<&NixList> {
        match self {
            Value::List(nix_list) => Some(nix_list),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct CloneableFlakeRef(pub FlakeRef);

impl AsRef<FlakeRef> for CloneableFlakeRef {
    fn as_ref(&self) -> &FlakeRef {
        self.deref()
    }
}

impl From<FlakeRef> for CloneableFlakeRef {
    fn from(value: FlakeRef) -> Self {
        Self(value)
    }
}

impl Deref for CloneableFlakeRef {
    type Target = FlakeRef;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CloneableFlakeRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Clone for CloneableFlakeRef {
    fn clone(&self) -> Self {
        Self(
            FlakeRef::from_str(self.0.to_uri().as_str()).expect("an injection function failed lol"),
        )
    }
}

impl Hash for CloneableFlakeRef {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.0 {
            FlakeRef::File {
                last_modified,
                nar_hash,
                rev,
                rev_count,
                url,
            } => {
                last_modified.hash(state);
                nar_hash.hash(state);
                rev.hash(state);
                rev_count.hash(state);
                url.hash(state);
            }
            FlakeRef::Git {
                all_refs,
                export_ignore,
                keytype,
                public_key,
                public_keys,
                r#ref,
                rev,
                shallow,
                submodules,
                url,
                verify_commit,
            } => {
                all_refs.hash(state);
                export_ignore.hash(state);
                keytype.hash(state);
                public_key.hash(state);
                public_keys.hash(state);
                r#ref.hash(state);
                rev.hash(state);
                shallow.hash(state);
                submodules.hash(state);
                url.hash(state);
                verify_commit.hash(state);
            }
            FlakeRef::GitHub {
                owner,
                repo,
                host,
                keytype,
                public_key,
                public_keys,
                r#ref,
                rev,
            } => {
                owner.hash(state);
                repo.hash(state);
                host.hash(state);
                keytype.hash(state);
                public_key.hash(state);
                public_keys.hash(state);
                r#ref.hash(state);
                rev.hash(state);
            }
            FlakeRef::GitLab {
                owner,
                repo,
                host,
                keytype,
                public_key,
                public_keys,
                r#ref,
                rev,
            } => {
                owner.hash(state);
                repo.hash(state);
                host.hash(state);
                keytype.hash(state);
                public_key.hash(state);
                public_keys.hash(state);
                r#ref.hash(state);
                rev.hash(state);
            }
            FlakeRef::Indirect { id, r#ref, rev } => {
                id.hash(state);
                r#ref.hash(state);
                rev.hash(state);
            }
            FlakeRef::Mercurial { r#ref, rev } => {
                r#ref.hash(state);
                rev.hash(state);
            }
            FlakeRef::Path {
                last_modified,
                nar_hash,
                path,
                rev,
                rev_count,
            } => {
                last_modified.hash(state);
                nar_hash.hash(state);
                path.hash(state);
                rev.hash(state);
                rev_count.hash(state);
            }
            FlakeRef::SourceHut {
                owner,
                repo,
                host,
                keytype,
                public_key,
                public_keys,
                r#ref,
                rev,
            } => {
                owner.hash(state);
                repo.hash(state);
                host.hash(state);
                keytype.hash(state);
                public_key.hash(state);
                public_keys.hash(state);
                r#ref.hash(state);
                rev.hash(state);
            }
            FlakeRef::Tarball {
                last_modified,
                nar_hash,
                rev,
                rev_count,
                url,
            } => {
                last_modified.hash(state);
                nar_hash.hash(state);
                rev.hash(state);
                rev_count.hash(state);
                url.hash(state);
            }
            _ => {}
        }
    }
}
