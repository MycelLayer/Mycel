//! Shared protocol-facing types for the first Rust workspace cut.
//!
//! This module intentionally starts with a narrow typed object model instead of
//! a full canonical object graph. The current goal is to keep object-kind,
//! signing, and derived-ID knowledge in one place so the verifier and later
//! protocol layers do not each re-encode the same rules.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub const CORE_PROTOCOL_VERSION: &str = "mycel/0.1";
pub const WIRE_PROTOCOL_VERSION: &str = "mycel-wire/0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProtocolVersion {
    pub core: &'static str,
    pub wire: &'static str,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self {
            core: CORE_PROTOCOL_VERSION,
            wire: WIRE_PROTOCOL_VERSION,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SignatureRule {
    Forbidden,
    Required,
}

impl SignatureRule {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Forbidden => "forbidden",
            Self::Required => "required",
        }
    }

    pub fn is_required(self) -> bool {
        matches!(self, Self::Required)
    }
}

impl fmt::Display for SignatureRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObjectKind {
    Document,
    Block,
    Patch,
    Revision,
    View,
    Snapshot,
}

impl ObjectKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Block => "block",
            Self::Patch => "patch",
            Self::Revision => "revision",
            Self::View => "view",
            Self::Snapshot => "snapshot",
        }
    }

    pub fn schema(self) -> ObjectSchema {
        match self {
            Self::Document => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Forbidden,
                signer_field: None,
                derived_id_field: None,
                derived_id_prefix: None,
            },
            Self::Block => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Forbidden,
                signer_field: None,
                derived_id_field: None,
                derived_id_prefix: None,
            },
            Self::Patch => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Required,
                signer_field: Some("author"),
                derived_id_field: Some("patch_id"),
                derived_id_prefix: Some("patch"),
            },
            Self::Revision => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Required,
                signer_field: Some("author"),
                derived_id_field: Some("revision_id"),
                derived_id_prefix: Some("rev"),
            },
            Self::View => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Required,
                signer_field: Some("maintainer"),
                derived_id_field: Some("view_id"),
                derived_id_prefix: Some("view"),
            },
            Self::Snapshot => ObjectSchema {
                kind: self,
                signature_rule: SignatureRule::Required,
                signer_field: Some("created_by"),
                derived_id_field: Some("snapshot_id"),
                derived_id_prefix: Some("snap"),
            },
        }
    }
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for ObjectKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "document" => Ok(Self::Document),
            "block" => Ok(Self::Block),
            "patch" => Ok(Self::Patch),
            "revision" => Ok(Self::Revision),
            "view" => Ok(Self::View),
            "snapshot" => Ok(Self::Snapshot),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ObjectSchema {
    pub kind: ObjectKind,
    pub signature_rule: SignatureRule,
    pub signer_field: Option<&'static str>,
    pub derived_id_field: Option<&'static str>,
    pub derived_id_prefix: Option<&'static str>,
}

impl ObjectSchema {
    pub fn derived_id(self) -> Option<(&'static str, &'static str)> {
        match (self.derived_id_field, self.derived_id_prefix) {
            (Some(field), Some(prefix)) => Some((field, prefix)),
            _ => None,
        }
    }
}

pub fn object_schema(object_type: &str) -> Option<ObjectSchema> {
    ObjectKind::from_str(object_type)
        .ok()
        .map(ObjectKind::schema)
}

#[cfg(test)]
mod tests {
    use super::{object_schema, ObjectKind, SignatureRule};

    #[test]
    fn object_kind_round_trips_from_strings() {
        let kind = "revision"
            .parse::<ObjectKind>()
            .expect("revision should parse");
        assert_eq!(kind, ObjectKind::Revision);
        assert_eq!(kind.to_string(), "revision");
    }

    #[test]
    fn patch_schema_requires_signature_and_derived_id() {
        let schema = object_schema("patch").expect("patch schema should exist");
        assert_eq!(schema.kind, ObjectKind::Patch);
        assert_eq!(schema.signature_rule, SignatureRule::Required);
        assert_eq!(schema.signer_field, Some("author"));
        assert_eq!(schema.derived_id(), Some(("patch_id", "patch")));
    }

    #[test]
    fn unknown_object_kind_has_no_schema() {
        assert!(object_schema("unknown-object").is_none());
    }
}
