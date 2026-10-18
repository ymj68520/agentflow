//! Private construction machinery, not a source-admission or authority API.
//! PendingId is a builder handle, NEVER a DagRank or a published graph reference.
use af_core::Value as V;
use serde_json::Value as J;
use sha2::{Digest, Sha256};
use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use unicode_normalization::UnicodeNormalization;

type Result<T> = std::result::Result<T, Failure>;
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Failure {
    pub domain: &'static str,
    pub reason: &'static str,
    pub path: Vec<String>,
}
fn fail(domain: &'static str, reason: &'static str) -> Failure {
    Failure {
        domain,
        reason,
        path: vec![],
    }
}
fn invalid(reason: &'static str) -> Failure {
    fail("InvalidFacts", reason)
}
fn resource() -> Failure {
    fail("ResourceLimit", "resource_bound")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct PendingId(usize);
#[derive(Clone)]
enum Body {
    Scalar(V),
    Record(Vec<(String, PendingId)>),
    Variant(String, PendingId),
    Sequence(Vec<PendingId>),
    Set(Vec<PendingId>),
    Map(Vec<(PendingId, PendingId)>),
    None,
    Some(PendingId),
}
impl Body {
    fn children(&self) -> Vec<PendingId> {
        match self {
            Self::Scalar(_) | Self::None => vec![],
            Self::Record(f) => f.iter().map(|f| f.1).collect(),
            Self::Variant(_, n) | Self::Some(n) => vec![*n],
            Self::Sequence(v) | Self::Set(v) => v.clone(),
            Self::Map(v) => v.iter().flat_map(|(k, v)| [*k, *v]).collect(),
        }
    }
}
struct Node {
    key: String,
    height: u64,
    body: Body,
}
pub(crate) struct Graph {
    pub value: V,
    pub sites: BTreeSet<String>,
    pub nodes: usize,
}
struct Engine<'a> {
    model: &'a J,
    sites: BTreeMap<(String, String), String>,
    site_aliases: BTreeSet<String>,
    bindings: BTreeMap<String, &'a J>,
    eligible: BTreeSet<String>,
    nodes: Vec<Node>,
    candidates: BTreeMap<(String, [u8; 32]), Vec<PendingId>>,
    comparisons: RefCell<BTreeMap<(PendingId, PendingId), Ordering>>,
    visited_sites: BTreeSet<String>,
    work: Cell<u64>,
    edges: u64,
    scalar_bytes: u64,
}
impl Engine<'_> {
    fn source_equal(&self, a: &V, b: &V) -> Result<bool> {
        self.spend(1)?;
        Ok(match (a, b) {
            (V::List(a), V::List(b)) => {
                if a.len() != b.len() {
                    return Ok(false);
                }
                for (a, b) in a.iter().zip(b) {
                    if !self.source_equal(a, b)? {
                        return Ok(false);
                    }
                }
                true
            }
            (V::Record(a), V::Record(b)) => {
                if a.len() != b.len() {
                    return Ok(false);
                }
                for ((an, a), (bn, b)) in a.iter().zip(b) {
                    if an != bn || !self.source_equal(a, b)? {
                        return Ok(false);
                    }
                }
                true
            }
            (V::Enum(a), V::Enum(b)) => a == b,
            (V::Variant { case: a, value: x }, V::Variant { case: b, value: y }) => {
                a == b && self.source_equal(x, y)?
            }
            (V::Enum(a), V::Variant { case: b, value })
            | (V::Variant { case: b, value }, V::Enum(a)) => {
                a == b && matches!(value.as_ref(), V::Unit)
            }
            (V::Option(None), V::Option(None)) => true,
            (V::Option(Some(a)), V::Option(Some(b))) => self.source_equal(a, b)?,
            (V::Option(_), V::Option(_)) => false,
            _ => self.scalar_order(a, b)? == Ordering::Equal,
        })
    }
    fn source_fingerprint(&self, value: &V) -> Result<[u8; 32]> {
        fn feed(h: &mut Sha256, bytes: &[u8]) {
            h.update((bytes.len() as u64).to_be_bytes());
            h.update(bytes);
        }
        fn walk(e: &Engine<'_>, h: &mut Sha256, v: &V) -> Result<()> {
            e.spend(1)?;
            match v {
                V::Record(fields) => {
                    feed(h, b"record");
                    for (n, v) in fields {
                        feed(h, n.as_bytes());
                        walk(e, h, v)?;
                    }
                }
                V::List(values) => {
                    feed(h, b"sequence");
                    feed(h, &(values.len() as u64).to_be_bytes());
                    for v in values {
                        walk(e, h, v)?;
                    }
                }
                V::Enum(case) => {
                    feed(h, b"variant");
                    feed(h, case.as_bytes());
                    walk(e, h, &V::Unit)?;
                }
                V::Variant { case, value } => {
                    feed(h, b"variant");
                    feed(h, case.as_bytes());
                    walk(e, h, value)?;
                }
                V::Option(None) => feed(h, b"none"),
                V::Option(Some(v)) => {
                    feed(h, b"some");
                    walk(e, h, v)?;
                }
                V::Bytes(v) => {
                    e.spend((v.len() as u64).div_ceil(64))?;
                    feed(h, v);
                }
                V::String(v) => {
                    e.spend((v.len() as u64).div_ceil(64))?;
                    feed(h, v.as_bytes());
                }
                V::Decimal(v) => {
                    e.spend((v.as_str().len() as u64).div_ceil(64))?;
                    feed(h, v.as_str().as_bytes());
                }
                V::Unit => feed(h, b"unit"),
                V::Null => feed(h, b"null"),
                V::Bool(v) => feed(h, &[u8::from(*v)]),
                V::UInt32(v) => feed(h, &v.to_be_bytes()),
                V::UInt64(v) => feed(h, &v.to_be_bytes()),
                V::Int32(v) => feed(h, &v.to_be_bytes()),
                V::Int64(v) => feed(h, &v.to_be_bytes()),
                V::Timestamp(v) => feed(h, &v.as_micros().to_be_bytes()),
                V::Duration(v) => feed(h, &v.as_micros().to_be_bytes()),
                _ => return Err(invalid("invalid_shape")),
            }
            Ok(())
        }
        let mut h = Sha256::new();
        walk(self, &mut h, value)?;
        Ok(h.finalize().into())
    }
    fn spend(&self, n: u64) -> Result<()> {
        let work = self.work.get().checked_add(n).ok_or_else(resource)?;
        if work > 4_194_304 {
            return Err(resource());
        }
        self.work.set(work);
        Ok(())
    }
    fn descriptor(&self, key: &str) -> Result<(String, Vec<J>, J)> {
        if !self.eligible.contains(key) {
            return Err(fail("UnsupportedModel", "ineligible_value_type"));
        }
        let b = self
            .bindings
            .get(key)
            .ok_or_else(|| fail("UnsupportedModel", "unknown_nominal_type"))?;
        let mut parent = b["declaration"].as_str().unwrap().to_owned();
        let mut path = b["path"].as_array().unwrap().clone();
        let mut d = self.model["types"][key].clone();
        while let Some(n) = d["ref"].as_str() {
            n.clone_into(&mut parent);
            path.clear();
            d = self.model["types"][&parent].clone();
        }
        Ok((parent, path, d))
    }
    fn child_key(&mut self, parent: &str, path: &[J], part: J, d: &J) -> Result<String> {
        if let Some(n) = d["ref"].as_str() {
            return Ok(n.to_owned());
        }
        if d["kind"] == "dag_ref" {
            let policies = self.model["structural_binding_registration"]["reference_policies"]
                .as_array()
                .unwrap();
            let mut selectors = path.to_vec();
            selectors.push(part);
            let formal: Vec<_> = selectors
                .iter()
                .map(|s| {
                    let a = s.as_array().unwrap();
                    serde_json::json!({a[0].as_str().unwrap():a[1]})
                })
                .collect();
            let p = policies
                .iter()
                .find(|p| {
                    p["key"]["parent"]
                        == super::structural_staging::nominal(parent, super::STRUCTURAL_MODEL_ID)
                        && p["key"]["path"] == J::Array(formal.clone())
                        && p["environment"] == "g_value"
                })
                .ok_or_else(|| invalid("input_output_backedge"))?;
            if p["domain"] != "g_local" {
                return Err(invalid("input_output_backedge"));
            }
            return d["target"]["ref"]
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("invalid_child"));
        }
        let mut selectors = path.to_vec();
        selectors.push(part);
        let path_key = serde_json::to_string(&selectors).unwrap();
        let key = self
            .sites
            .get(&(parent.to_owned(), path_key))
            .cloned()
            .ok_or_else(|| fail("IncompleteContent", "missing_site"))?;
        Ok(key)
    }
    fn sub(
        &mut self,
        parent: &str,
        path: &[J],
        part: J,
        d: &J,
        v: &V,
        depth: u64,
    ) -> Result<PendingId> {
        let k = self.child_key(parent, path, part, d)?;
        self.build(&k, v, depth + 1)
    }
    fn build(&mut self, key: &str, v: &V, depth: u64) -> Result<PendingId> {
        self.spend(1)?;
        if depth > 96 {
            return Err(resource());
        }
        if self.site_aliases.contains(key) {
            self.visited_sites.insert(key.to_owned());
        }
        let (parent, path, d) = self.descriptor(key)?;
        let kind = d["kind"]
            .as_str()
            .ok_or_else(|| fail("UnsupportedModel", "invalid_shape"))?;
        let part = |tag: &str, v: J| serde_json::json!([tag, v]);
        let body = match kind {
            "record" => {
                let V::Record(values) = v else {
                    return Err(invalid("invalid_shape"));
                };
                let fields = d["fields"].as_array().unwrap();
                if values.len() != fields.len() {
                    return Err(fail("IncompleteFacts", "invalid_shape"));
                }
                let mut out = Vec::new();
                for (r, (name, value)) in fields.iter().zip(values) {
                    if r[0] != name.as_str() {
                        return Err(invalid("invalid_shape"));
                    }
                    let child = self.sub(
                        &parent,
                        &path,
                        part("field", J::String(name.clone())),
                        &r[1],
                        value,
                        depth,
                    )?;
                    out.push((name.clone(), child));
                }
                Body::Record(out)
            }
            "enum" => {
                let (tag, value) = match v {
                    V::Enum(t) => (t, &V::Unit),
                    V::Variant { case, value } => (case, value.as_ref()),
                    _ => return Err(invalid("invalid_shape")),
                };
                let row = d["variants"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r[0] == tag.as_str())
                    .ok_or_else(|| fail("UnsupportedModel", "invalid_shape"))?;
                Body::Variant(
                    tag.clone(),
                    self.sub(
                        &parent,
                        &path,
                        part("case", J::String(tag.clone())),
                        &row[1],
                        value,
                        depth,
                    )?,
                )
            }
            "sequence" | "set" => {
                let V::List(items) = v else {
                    return Err(invalid("invalid_shape"));
                };
                if items.len()
                    < usize::try_from(d["minimum"].as_u64().unwrap_or(0)).map_err(|_| resource())?
                {
                    return Err(fail("IncompleteFacts", "invalid_shape"));
                }
                let mut out = Vec::new();
                for item in items {
                    out.push(self.sub(
                        &parent,
                        &path,
                        part(if kind == "set" { "member" } else { "item" }, J::Null),
                        &d["item"],
                        item,
                        depth,
                    )?);
                }
                if kind == "set" {
                    self.sort_values(&mut out)?;
                    for pair in out.windows(2) {
                        if self.value_cmp(pair[0], pair[1])? == Ordering::Equal {
                            return Err(invalid("invalid_set_member"));
                        }
                    }
                    Body::Set(out)
                } else {
                    Body::Sequence(out)
                }
            }
            "option" => {
                let V::Option(item) = v else {
                    return Err(invalid("invalid_shape"));
                };
                match item {
                    None => Body::None,
                    Some(item) => Body::Some(self.sub(
                        &parent,
                        &path,
                        part("some", J::Null),
                        &d["item"],
                        item,
                        depth,
                    )?),
                }
            }
            "map" => {
                let V::List(items) = v else {
                    return Err(invalid("invalid_map_entry"));
                };
                // The complete immutable source already passed the typed
                // duplicate-key preflight before any value node was interned.
                let mut pairs = Vec::new();
                for p in items {
                    let V::List(p) = p else { unreachable!() };
                    pairs.push((
                        self.sub(
                            &parent,
                            &path,
                            part("key", J::Null),
                            &d["key"],
                            &p[0],
                            depth,
                        )?,
                        self.sub(
                            &parent,
                            &path,
                            part("value", J::Null),
                            &d["value"],
                            &p[1],
                            depth,
                        )?,
                    ));
                }
                for i in 1..pairs.len() {
                    let mut j = i;
                    while j > 0 && self.value_cmp(pairs[j - 1].0, pairs[j].0)? == Ordering::Greater
                    {
                        pairs.swap(j - 1, j);
                        j -= 1;
                    }
                }
                for p in pairs.windows(2) {
                    if self.value_cmp(p[0].0, p[1].0)? == Ordering::Equal {
                        return Err(invalid("duplicate_map_key"));
                    }
                }
                Body::Map(pairs)
            }
            _ => {
                let scalar = checked_scalar(kind, &d, v)?;
                let bytes = match &scalar {
                    V::Bytes(b) => b.len(),
                    V::String(s) => s.len(),
                    _ => 0,
                };
                self.scalar_bytes = self
                    .scalar_bytes
                    .checked_add(bytes as u64)
                    .ok_or_else(resource)?;
                if self.scalar_bytes > 134_217_728 {
                    return Err(resource());
                }
                // Byte scans are additionally bounded by scalar_bytes. Work
                // counts visited values/pairs and fixed 64-byte scan blocks.
                self.spend((bytes as u64).div_ceil(64))?;
                Body::Scalar(scalar)
            }
        };
        let children = body.children();
        let height = children
            .iter()
            .map(|n| self.nodes[n.0].height)
            .max()
            .map_or(0, |h| h + 1);
        let fingerprint = self.fingerprint(&body)?;
        for id in self
            .candidates
            .get(&(key.to_owned(), fingerprint))
            .cloned()
            .unwrap_or_default()
        {
            self.spend(1)?;
            if self.body_equal(&self.nodes[id.0].body, &body)? {
                return Ok(id);
            }
        }
        self.edges = self
            .edges
            .checked_add(children.len() as u64)
            .ok_or_else(resource)?;
        if self.edges > 262_144 {
            return Err(resource());
        }
        if self.nodes.len() >= 1_048_576 {
            return Err(resource());
        }
        let id = PendingId(self.nodes.len());
        self.nodes.push(Node {
            key: key.to_owned(),
            height,
            body,
        });
        self.candidates
            .entry((key.to_owned(), fingerprint))
            .or_default()
            .push(id);
        Ok(id)
    }
    fn body_equal(&self, a: &Body, b: &Body) -> Result<bool> {
        Ok(match (a, b) {
            (Body::Scalar(a), Body::Scalar(b)) => self.scalar_order(a, b)? == Ordering::Equal,
            (Body::Record(a), Body::Record(b)) => a == b,
            (Body::Variant(a, x), Body::Variant(b, y)) => a == b && x == y,
            (Body::Sequence(a), Body::Sequence(b)) | (Body::Set(a), Body::Set(b)) => a == b,
            (Body::Map(a), Body::Map(b)) => a == b,
            (Body::None, Body::None) => true,
            (Body::Some(a), Body::Some(b)) => a == b,
            _ => false,
        })
    }
    fn sort_values(&mut self, values: &mut [PendingId]) -> Result<()> {
        for i in 1..values.len() {
            let mut j = i;
            while j > 0 && self.value_cmp(values[j - 1], values[j])? == Ordering::Greater {
                values.swap(j - 1, j);
                j -= 1;
            }
        }
        Ok(())
    }
    #[allow(clippy::many_single_char_names)]
    fn value_cmp(&self, a: PendingId, b: PendingId) -> Result<Ordering> {
        self.spend(1)?;
        if a == b {
            return Ok(Ordering::Equal);
        }
        if let Some(order) = self.comparisons.borrow().get(&(a, b)).copied() {
            return Ok(order);
        }
        let a_node = &self.nodes[a.0];
        let b_node = &self.nodes[b.0];
        if a_node.key != b_node.key {
            return Err(invalid("invalid_child"));
        }
        let a_body = &a_node.body;
        let b_body = &b_node.body;
        let result = match (a_body, b_body) {
            (Body::Scalar(a), Body::Scalar(b)) => self.scalar_order(a, b),
            (Body::Variant(a, x), Body::Variant(b, y)) => {
                let (_, _, d) = self.descriptor(&a_node.key)?;
                let rows = d["variants"].as_array().unwrap();
                let c = rows
                    .iter()
                    .position(|r| r[0] == a.as_str())
                    .cmp(&rows.iter().position(|r| r[0] == b.as_str()));
                if c == Ordering::Equal {
                    self.value_cmp(*x, *y)
                } else {
                    Ok(c)
                }
            }
            (Body::None, Body::Some(_)) => Ok(Ordering::Less),
            (Body::Some(_), Body::None) => Ok(Ordering::Greater),
            _ => {
                let x = a_body.children();
                let y = b_body.children();
                let mut difference = Ordering::Equal;
                for (a, b) in x.iter().zip(&y) {
                    let c = self.value_cmp(*a, *b)?;
                    if c != Ordering::Equal {
                        difference = c;
                        break;
                    }
                }
                Ok(if difference == Ordering::Equal {
                    x.len().cmp(&y.len())
                } else {
                    difference
                })
            }
        }?;
        self.comparisons.borrow_mut().insert((a, b), result);
        self.comparisons
            .borrow_mut()
            .insert((b, a), result.reverse());
        Ok(result)
    }
}

fn checked_scalar(kind: &str, d: &J, v: &V) -> Result<V> {
    let valid = match (kind, v) {
        ("unit", V::Unit)
        | ("null", V::Null)
        | ("bool", V::Bool(_))
        | ("u32", V::UInt32(_))
        | ("u64", V::UInt64(_))
        | ("i32", V::Int32(_))
        | ("i64", V::Int64(_))
        | ("decimal", V::Decimal(_))
        | ("timestamp_i64", V::Timestamp(_))
        | ("duration_i64", V::Duration(_)) => true,
        ("u8", V::UInt32(n)) => *n <= 255,
        ("nfc_text", V::String(s)) => s.nfc().eq(s.chars()),
        ("bytes", V::Bytes(b)) => {
            b.len()
                >= usize::try_from(d["minimum_bytes"].as_u64().unwrap_or(0))
                    .map_err(|_| resource())?
                && b.len()
                    <= usize::try_from(d["maximum_bytes"].as_u64().unwrap_or(134_217_728))
                        .map_err(|_| resource())?
                && (d["utf8"] != true || std::str::from_utf8(b).is_ok())
        }
        _ => false,
    };
    if valid {
        Ok(v.clone())
    } else {
        Err(fail("UnsupportedModel", "invalid_shape"))
    }
}
fn scalar_cmp(a: &V, b: &V) -> Result<Ordering> {
    Ok(match (a, b) {
        (V::Unit, V::Unit) | (V::Null, V::Null) => Ordering::Equal,
        (V::Bool(a), V::Bool(b)) => a.cmp(b),
        (V::UInt32(a), V::UInt32(b)) => a.cmp(b),
        (V::UInt64(a), V::UInt64(b)) => a.cmp(b),
        (V::Int32(a), V::Int32(b)) => a.cmp(b),
        (V::Int64(a), V::Int64(b)) => a.cmp(b),
        (V::Timestamp(a), V::Timestamp(b)) => a.cmp(b),
        (V::Duration(a), V::Duration(b)) => a.cmp(b),
        (V::Bytes(a), V::Bytes(b)) => a.cmp(b),
        (V::String(a), V::String(b)) => a.as_bytes().cmp(b.as_bytes()),
        (V::Decimal(a), V::Decimal(b)) => {
            let a = a.as_str();
            let b = b.as_str();
            let negative = a.starts_with('-');
            if negative != b.starts_with('-') {
                return Ok(if negative {
                    Ordering::Less
                } else {
                    Ordering::Greater
                });
            }
            let (ai, af) = a
                .trim_start_matches('-')
                .split_once('.')
                .unwrap_or((a.trim_start_matches('-'), ""));
            let (bi, bf) = b
                .trim_start_matches('-')
                .split_once('.')
                .unwrap_or((b.trim_start_matches('-'), ""));
            let n = af.len().max(bf.len());
            let c = ai
                .len()
                .cmp(&bi.len())
                .then_with(|| ai.cmp(bi))
                .then_with(|| format!("{af:0<n$}").cmp(&format!("{bf:0<n$}")));
            if negative { c.reverse() } else { c }
        }
        _ => return Err(invalid("invalid_shape")),
    })
}
fn variant(case: &str, value: V) -> V {
    V::Variant {
        case: case.to_owned(),
        value: Box::new(value),
    }
}
fn record(fields: Vec<(&str, V)>) -> V {
    V::Record(fields.into_iter().map(|(n, v)| (n.to_owned(), v)).collect())
}
fn nominal(key: &str) -> V {
    let n = super::structural_staging::nominal(key, super::STRUCTURAL_MODEL_ID);
    record(vec![
        ("owner", V::String(n["owner"].as_str().unwrap().to_owned())),
        ("name", V::String(n["name"].as_str().unwrap().to_owned())),
    ])
}
impl Engine<'_> {
    fn scalar_order(&self, a: &V, b: &V) -> Result<Ordering> {
        self.spend(1)?;
        let bytes = match (a, b) {
            (V::Bytes(a), V::Bytes(b)) => Some((a.as_slice(), b.as_slice())),
            (V::String(a), V::String(b)) => Some((a.as_bytes(), b.as_bytes())),
            _ => None,
        };
        if let Some((a, b)) = bytes {
            for (x, y) in a.chunks(64).zip(b.chunks(64)) {
                self.spend(1)?;
                let order = x.cmp(y);
                if order != Ordering::Equal {
                    return Ok(order);
                }
            }
            return Ok(a.len().cmp(&b.len()));
        }
        if let (V::Decimal(a), V::Decimal(b)) = (a, b) {
            self.spend(
                ((a.as_str().len() + b.as_str().len()) as u64)
                    .saturating_mul(4)
                    .div_ceil(64),
            )?;
        }
        scalar_cmp(a, b)
    }
    fn fingerprint(&self, b: &Body) -> Result<[u8; 32]> {
        // Candidate index only. Every matching bucket still compares the full
        // exact typed body. This is not a graph identity or HashV1 domain.
        let mut h = Sha256::new();
        let mut bytes = |v: &[u8]| {
            h.update((v.len() as u64).to_be_bytes());
            h.update(v);
        };
        match b {
            Body::Scalar(v) => match v {
                V::Unit => bytes(b"unit"),
                V::Null => bytes(b"null"),
                V::Bool(v) => bytes(&[u8::from(*v)]),
                V::Int32(v) => bytes(&v.to_be_bytes()),
                V::UInt32(v) => bytes(&v.to_be_bytes()),
                V::Int64(v) => bytes(&v.to_be_bytes()),
                V::UInt64(v) => bytes(&v.to_be_bytes()),
                V::Decimal(v) => {
                    self.spend((v.as_str().len() as u64).div_ceil(64))?;
                    bytes(v.as_str().as_bytes());
                }
                V::String(v) => {
                    self.spend((v.len() as u64).div_ceil(64))?;
                    bytes(v.as_bytes());
                }
                V::Bytes(v) => {
                    self.spend((v.len() as u64).div_ceil(64))?;
                    bytes(v);
                }
                V::Timestamp(v) => bytes(&v.as_micros().to_be_bytes()),
                V::Duration(v) => bytes(&v.as_micros().to_be_bytes()),
                _ => return Err(invalid("invalid_shape")),
            },
            Body::Record(v) => {
                for (n, _) in v {
                    bytes(n.as_bytes());
                }
            }
            Body::Variant(case, _) => bytes(case.as_bytes()),
            Body::Sequence(v) | Body::Set(v) => bytes(&(v.len() as u64).to_be_bytes()),
            Body::Map(v) => bytes(&(v.len() as u64).to_be_bytes()),
            Body::None => bytes(b"none"),
            Body::Some(_) => bytes(b"some"),
        }
        for id in b.children() {
            bytes(&(id.0 as u64).to_be_bytes());
        }
        Ok(h.finalize().into())
    }
    fn preflight(&mut self, key: &str, v: &V, depth: u64) -> Result<()> {
        self.spend(1)?;
        if depth > 96 {
            return Err(resource());
        }
        let (parent, path, d) = self.descriptor(key)?;
        let kind = d["kind"].as_str().ok_or_else(|| invalid("invalid_shape"))?;
        let mut children = Vec::new();
        match (kind, v) {
            ("record", V::Record(values)) => {
                let fields = d["fields"].as_array().unwrap();
                if fields.len() != values.len() {
                    return Err(invalid("invalid_shape"));
                }
                for (row, (name, value)) in fields.iter().zip(values) {
                    if row[0] != name.as_str() {
                        return Err(invalid("invalid_shape"));
                    }
                    children.push((
                        self.child_key(
                            &parent,
                            &path,
                            serde_json::json!(["field", name]),
                            &row[1],
                        )?,
                        value,
                    ));
                }
            }
            ("enum", V::Enum(_) | V::Variant { .. }) => {
                let (tag, value) = match v {
                    V::Enum(tag) => (tag, &V::Unit),
                    V::Variant { case, value } => (case, value.as_ref()),
                    _ => unreachable!(),
                };
                let row = d["variants"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| r[0] == tag.as_str())
                    .ok_or_else(|| invalid("invalid_shape"))?;
                children.push((
                    self.child_key(&parent, &path, serde_json::json!(["case", tag]), &row[1])?,
                    value,
                ));
            }
            ("option", V::Option(Some(value))) => children.push((
                self.child_key(
                    &parent,
                    &path,
                    serde_json::json!(["some", null]),
                    &d["item"],
                )?,
                value.as_ref(),
            )),
            ("option", V::Option(None)) => {}
            ("sequence" | "set", V::List(items)) => {
                if items.len()
                    < usize::try_from(d["minimum"].as_u64().unwrap_or(0)).map_err(|_| resource())?
                {
                    return Err(invalid("invalid_shape"));
                }
                let key = self.child_key(
                    &parent,
                    &path,
                    serde_json::json!([if kind == "set" { "member" } else { "item" }, null]),
                    &d["item"],
                )?;
                children.extend(items.iter().map(|v| (key.clone(), v)));
            }
            ("map", V::List(items)) => {
                let k =
                    self.child_key(&parent, &path, serde_json::json!(["key", null]), &d["key"])?;
                let val = self.child_key(
                    &parent,
                    &path,
                    serde_json::json!(["value", null]),
                    &d["value"],
                )?;
                for p in items {
                    let V::List(p) = p else {
                        return Err(invalid("invalid_map_entry"));
                    };
                    if p.len() != 2 {
                        return Err(invalid("invalid_map_entry"));
                    }
                    children.push((k.clone(), &p[0]));
                    children.push((val.clone(), &p[1]));
                }
            }
            _ => {
                checked_scalar(kind, &d, v)?;
                let bytes = match v {
                    V::String(s) => s.len(),
                    V::Bytes(b) => b.len(),
                    V::Decimal(d) => d.as_str().len(),
                    _ => 0,
                };
                self.spend((bytes as u64).div_ceil(64))?;
            }
        }
        for (k, value) in children {
            self.preflight(&k, value, depth + 1)?;
        }
        if matches!(kind, "map" | "set") {
            let V::List(items) = v else { unreachable!() };
            let mut candidates = BTreeMap::<[u8; 32], Vec<&V>>::new();
            for item in items {
                let value = if kind == "map" {
                    let V::List(pair) = item else { unreachable!() };
                    &pair[0]
                } else {
                    item
                };
                let fingerprint = self.source_fingerprint(value)?;
                for prior in candidates.get(&fingerprint).into_iter().flatten() {
                    if self.source_equal(prior, value)? {
                        return Err(invalid(if kind == "map" {
                            "duplicate_map_key"
                        } else {
                            "invalid_set_member"
                        }));
                    }
                }
                candidates.entry(fingerprint).or_default().push(value);
            }
        }
        Ok(())
    }
    fn rank_order(&self, a: PendingId, b: PendingId, ranks: &[Option<u64>]) -> Result<Ordering> {
        self.spend(1)?;
        let an = &self.nodes[a.0];
        let bn = &self.nodes[b.0];
        let at = super::structural_staging::nominal(&an.key, super::STRUCTURAL_MODEL_ID);
        let bt = super::structural_staging::nominal(&bn.key, super::STRUCTURAL_MODEL_ID);
        let prefix = an
            .height
            .cmp(&bn.height)
            .then_with(|| at["owner"].as_str().cmp(&bt["owner"].as_str()))
            .then_with(|| {
                self.model["type_ranks"][&an.key]
                    .as_u64()
                    .cmp(&self.model["type_ranks"][&bn.key].as_u64())
            })
            .then_with(|| at["name"].as_str().cmp(&bt["name"].as_str()));
        if prefix != Ordering::Equal {
            return Ok(prefix);
        }
        let base = match (&an.body, &bn.body) {
            (Body::Scalar(a), Body::Scalar(b)) => self.scalar_order(a, b)?,
            (Body::Variant(a, _), Body::Variant(b, _)) => {
                let (_, _, d) = self.descriptor(&an.key)?;
                let rows = d["variants"].as_array().unwrap();
                rows.iter()
                    .position(|r| r[0] == a.as_str())
                    .cmp(&rows.iter().position(|r| r[0] == b.as_str()))
            }
            (Body::None, Body::Some(_)) => Ordering::Less,
            (Body::Some(_), Body::None) => Ordering::Greater,
            (Body::Record(a), Body::Record(b)) => {
                a.iter().map(|x| &x.0).cmp(b.iter().map(|x| &x.0))
            }
            (Body::Sequence(a), Body::Sequence(b)) | (Body::Set(a), Body::Set(b)) => {
                a.len().cmp(&b.len())
            }
            (Body::Map(a), Body::Map(b)) => a.len().cmp(&b.len()),
            _ => Ordering::Equal,
        };
        if base != Ordering::Equal {
            return Ok(base);
        }
        let children = |b: &Body| {
            b.children()
                .iter()
                .map(|n| ranks[n.0].ok_or_else(|| invalid("rank_dependency")))
                .collect::<Result<Vec<_>>>()
        };
        Ok(children(&an.body)?.cmp(&children(&bn.body)?))
    }
    fn finish(self, root: PendingId) -> Result<Graph> {
        // Every builder node is a descendant. Prove this independently rather
        // than retaining unused construction records as invented graph roots.
        let mut seen = BTreeSet::new();
        let mut todo = vec![root];
        while let Some(n) = todo.pop() {
            self.spend(1)?;
            if seen.insert(n) {
                todo.extend(self.nodes[n.0].body.children());
            }
        }
        if seen.len() != self.nodes.len() {
            return Err(invalid("invalid_interning"));
        }
        let mut heights = BTreeMap::<u64, Vec<PendingId>>::new();
        for (i, n) in self.nodes.iter().enumerate() {
            heights.entry(n.height).or_default().push(PendingId(i));
        }
        let mut ranks = vec![None; self.nodes.len()];
        let mut ordered = Vec::new();
        for values in heights.values_mut() {
            fallible_sort(values, &|a, b| self.rank_order(a, b, &ranks))?;
            for id in values {
                let rank = u64::try_from(ordered.len()).map_err(|_| resource())?;
                ranks[id.0] = Some(rank);
                ordered.push(*id);
            }
        }
        let rank = |n: PendingId| V::UInt64(ranks[n.0].unwrap());
        let mut output = Vec::new();
        for id in ordered {
            let node = &self.nodes[id.0];
            let body = match &node.body {
                Body::Scalar(v) => {
                    let tag = match v {
                        V::Unit => "unit",
                        V::Null => "null",
                        V::Bool(_) => "bool",
                        V::Int32(_) => "int32",
                        V::UInt32(_) => "uint32",
                        V::Int64(_) => "int64",
                        V::UInt64(_) => "uint64",
                        V::Decimal(_) => "decimal",
                        V::String(_) => "string",
                        V::Bytes(_) => "bytes",
                        V::Timestamp(_) => "timestamp",
                        V::Duration(_) => "duration",
                        _ => return Err(invalid("invalid_shape")),
                    };
                    let payload = match v {
                        V::Timestamp(t) => V::Int64(t.as_micros()),
                        V::Duration(d) => V::Int64(d.as_micros()),
                        _ => v.clone(),
                    };
                    variant("scalar", variant(tag, payload))
                }
                Body::Record(fields) => variant(
                    "record",
                    V::List(
                        fields
                            .iter()
                            .map(|(name, n)| {
                                record(vec![
                                    ("field", V::String(name.clone())),
                                    ("value", rank(*n)),
                                ])
                            })
                            .collect(),
                    ),
                ),
                Body::Variant(case, n) => variant(
                    "variant",
                    record(vec![
                        ("case", V::String(case.clone())),
                        ("payload", rank(*n)),
                    ]),
                ),
                Body::Sequence(v) => {
                    variant("sequence", V::List(v.iter().map(|n| rank(*n)).collect()))
                }
                Body::Set(v) => variant("set", V::List(v.iter().map(|n| rank(*n)).collect())),
                Body::Map(v) => variant(
                    "map",
                    V::List(
                        v.iter()
                            .map(|(k, v)| record(vec![("key", rank(*k)), ("value", rank(*v))]))
                            .collect(),
                    ),
                ),
                Body::None => variant("option_none", V::Unit),
                Body::Some(n) => variant("option_some", rank(*n)),
            };
            output.push(record(vec![("type", nominal(&node.key)), ("body", body)]));
        }
        Ok(Graph {
            value: record(vec![
                ("nodes", V::List(output)),
                ("roots", V::List(vec![rank(root)])),
            ]),
            sites: self.visited_sites,
            nodes: self.nodes.len(),
        })
    }
}
fn fallible_sort(
    values: &mut [PendingId],
    compare: &impl Fn(PendingId, PendingId) -> Result<Ordering>,
) -> Result<()> {
    if values.len() < 2 {
        return Ok(());
    }
    let middle = values.len() / 2;
    fallible_sort(&mut values[..middle], compare)?;
    fallible_sort(&mut values[middle..], compare)?;
    let mut merged = Vec::with_capacity(values.len());
    let (mut left, mut right) = (0, middle);
    while left < middle && right < values.len() {
        if compare(values[left], values[right])? == Ordering::Greater {
            merged.push(values[right]);
            right += 1;
        } else {
            merged.push(values[left]);
            left += 1;
        }
    }
    merged.extend_from_slice(&values[left..middle]);
    merged.extend_from_slice(&values[right..]);
    values.copy_from_slice(&merged);
    Ok(())
}
/// Only the private native source-admission adapter may supply this expanded
/// value. Logical dag_ref inputs here are expanded source targets, not ranks.
pub(crate) fn construct_admitted_subject(
    source: &V,
) -> std::result::Result<Graph, (Failure, &'static str)> {
    let model = super::structural_model_catalogue().map_err(|_| {
        (
            fail("IncompleteContent", "missing_site"),
            "source_admission",
        )
    })?;
    let r = &model["structural_binding_registration"];
    let eligible = r["value_bindings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| {
            b["eligibility"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e == "g_input")
        })
        .map(|b| b["catalogue_key"].as_str().unwrap().to_owned())
        .collect();
    let sites: BTreeMap<_, _> = r["sites"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| {
            (
                (
                    s["source_catalogue_key"].as_str().unwrap().to_owned(),
                    serde_json::to_string(&s["source_path"]).unwrap(),
                ),
                s["alias"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    let site_aliases = sites.values().cloned().collect();
    let bindings = r["value_bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| (b["catalogue_key"].as_str().unwrap().to_owned(), b))
        .collect();
    let mut engine = Engine {
        model,
        sites,
        site_aliases,
        bindings,
        eligible,
        nodes: vec![],
        candidates: BTreeMap::new(),
        comparisons: RefCell::new(BTreeMap::new()),
        visited_sites: BTreeSet::new(),
        work: Cell::new(0),
        edges: 0,
        scalar_bytes: 0,
    };
    engine
        .preflight("RawValidationSubject", source, 0)
        .map_err(|f| (f, "source_admission"))?;
    let root = engine
        .build("RawValidationSubject", source, 0)
        .map_err(|f| (f, "pre_capture_binding"))?;
    engine.finish(root).map_err(|f| (f, "pre_capture_binding"))
}
