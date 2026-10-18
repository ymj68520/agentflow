//! Validator for immutable handwritten registration fixtures only.
//! It NEVER constructs a value DAG, assigns a rank or accepts real Raw input.
use crate::structural_staging::{
    MetaResult, StructuralMetadataFailure, bad, catalogue, missing, nominal, resource, type_key,
    unsupported, validate_structural_shape_metadata,
};
use serde_json::{Value, json};
use std::cell::Cell;
use std::cmp::Ordering;
use std::collections::BTreeSet;

fn measured(v: &Value, depth: usize) -> MetaResult<usize> {
    if depth > 64 {
        return Err(resource());
    }
    let mut count = 1usize;
    match v {
        Value::String(s) => count = count.checked_add(s.len()).ok_or_else(resource)?,
        Value::Array(a) => {
            for x in a {
                count = count
                    .checked_add(measured(x, depth + 1)?)
                    .ok_or_else(resource)?;
                if count > 16384 {
                    return Err(resource());
                }
            }
        }
        Value::Object(o) => {
            for (k, x) in o {
                count = count
                    .checked_add(k.len())
                    .and_then(|n| n.checked_add(1))
                    .ok_or_else(resource)?
                    .checked_add(measured(x, depth + 1)?)
                    .ok_or_else(resource)?;
                if count > 16384 {
                    return Err(resource());
                }
            }
        }
        _ => {}
    }
    if count > 16384 {
        return Err(resource());
    }
    Ok(count)
}

#[derive(Clone)]
struct Context {
    key: String,
    parent: String,
    path: Vec<Value>,
    desc: Value,
    eligible: Vec<String>,
}
fn context(mt: &Value) -> MetaResult<Context> {
    let m = catalogue()?;
    let b = m["structural_binding_registration"]["value_bindings"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|b| b["value_type"] == *mt)
        .ok_or_else(|| unsupported("ineligible_value_type"))?;
    let key = b["catalogue_key"]
        .as_str()
        .ok_or_else(|| unsupported("ineligible_value_type"))?
        .to_owned();
    let mut c = Context {
        desc: m["types"][&key].clone(),
        key,
        parent: b["declaration"].as_str().unwrap_or("").to_owned(),
        path: b["path"].as_array().cloned().unwrap_or_default(),
        eligible: b["eligibility"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
    };
    while let Some(n) = c.desc["ref"].as_str() {
        c.parent = n.to_owned();
        c.path.clear();
        c.desc = m["types"][n].clone();
    }
    Ok(c)
}
fn child_type(c: &Context, part: Value, d: &Value) -> MetaResult<Value> {
    if let Some(n) = d["ref"].as_str() {
        return Ok(nominal(n, super::structural_staging::STRUCTURAL_MODEL_ID));
    }
    if d["kind"] == "dag_ref" {
        return Ok(nominal(
            d["target"]["ref"]
                .as_str()
                .ok_or_else(|| bad("invalid_child"))?,
            super::structural_staging::STRUCTURAL_MODEL_ID,
        ));
    }
    let mut path = c.path.clone();
    path.push(part);
    let m = catalogue()?;
    m["structural_binding_registration"]["sites"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|s| {
            s["source_catalogue_key"] == c.parent && s["source_path"] == Value::Array(path.clone())
        })
        .map(|s| s["definition"]["value_type"].clone())
        .ok_or_else(|| {
            StructuralMetadataFailure::new(
                crate::RegisteredModelFailure::IncompleteContent,
                "missing_site",
            )
        })
}
#[allow(clippy::many_single_char_names)]
fn decimal_cmp(a: &str, b: &str) -> Ordering {
    let sa = a.starts_with('-');
    let sb = b.starts_with('-');
    if sa != sb {
        return sb.cmp(&sa);
    }
    let parts = |s: &str| {
        let x = s.trim_start_matches('-');
        let (i, f) = x.split_once('.').unwrap_or((x, ""));
        (i.to_owned(), f.to_owned())
    };
    let (ai, af) = parts(a);
    let (bi, bf) = parts(b);
    let n = af.len().max(bf.len());
    let ap = format!("{af:0<n$}");
    let bp = format!("{bf:0<n$}");
    let order = ai
        .len()
        .cmp(&bi.len())
        .then_with(|| ai.cmp(&bi))
        .then_with(|| ap.cmp(&bp));
    if sa { order.reverse() } else { order }
}
fn sequence_cmp(d: &Value, a: &[Value], b: &[Value], depth: usize) -> MetaResult<Ordering> {
    for (x, y) in a.iter().zip(b) {
        let c = compare(d, x, y, depth + 1)?;
        if c != Ordering::Equal {
            return Ok(c);
        }
    }
    Ok(a.len().cmp(&b.len()))
}
#[allow(clippy::too_many_lines, clippy::many_single_char_names)]
fn compare(d: &Value, a: &Value, b: &Value, depth: usize) -> MetaResult<Ordering> {
    if depth > 96 {
        return Err(resource());
    }
    let m = catalogue()?;
    if let Some(n) = d["ref"].as_str() {
        return compare(&m["types"][n], a, b, depth + 1);
    }
    let kind = d["kind"]
        .as_str()
        .ok_or_else(|| unsupported("invalid_shape"))?;
    Ok(match kind {
        "dag_ref" => return compare(&d["target"], a, b, depth + 1),
        "unit" | "null" => Ordering::Equal,
        "bool" => a.as_bool().cmp(&b.as_bool()),
        "u8" | "u32" | "u64" => a.as_u64().cmp(&b.as_u64()),
        "i32" | "i64" | "timestamp_i64" | "duration_i64" => a.as_i64().cmp(&b.as_i64()),
        "finite_f64" => a
            .as_f64()
            .and_then(|x| b.as_f64().and_then(|y| x.partial_cmp(&y)))
            .ok_or_else(|| unsupported("invalid_shape"))?,
        "decimal" => decimal_cmp(
            a.as_str().ok_or_else(|| unsupported("invalid_shape"))?,
            b.as_str().ok_or_else(|| unsupported("invalid_shape"))?,
        ),
        "nfc_text" => a.as_str().cmp(&b.as_str()),
        "bytes" => a
            .as_array()
            .ok_or_else(|| unsupported("invalid_shape"))?
            .iter()
            .map(Value::as_u64)
            .cmp(
                b.as_array()
                    .ok_or_else(|| unsupported("invalid_shape"))?
                    .iter()
                    .map(Value::as_u64),
            ),
        "record" => {
            let mut c = Ordering::Equal;
            for row in d["fields"].as_array().into_iter().flatten() {
                let n = row[0]
                    .as_str()
                    .ok_or_else(|| unsupported("invalid_shape"))?;
                c = compare(&row[1], &a[n], &b[n], depth + 1)?;
                if c != Ordering::Equal {
                    break;
                }
            }
            c
        }
        "enum" => {
            let variants = d["variants"]
                .as_array()
                .ok_or_else(|| unsupported("invalid_shape"))?;
            let unit = variants.iter().all(|v| v[1]["kind"] == "unit");
            let tag = |v: &Value| {
                if unit {
                    v.as_str().map(str::to_owned)
                } else {
                    v.as_object().and_then(|o| o.keys().next().cloned())
                }
            };
            let ta = tag(a).ok_or_else(|| unsupported("invalid_shape"))?;
            let tb = tag(b).ok_or_else(|| unsupported("invalid_shape"))?;
            let ia = variants
                .iter()
                .position(|r| r[0] == ta)
                .ok_or_else(|| unsupported("invalid_shape"))?;
            let ib = variants
                .iter()
                .position(|r| r[0] == tb)
                .ok_or_else(|| unsupported("invalid_shape"))?;
            ia.cmp(&ib).then(if unit || ia != ib {
                Ordering::Equal
            } else {
                compare(&variants[ia][1], &a[&ta], &b[&tb], depth + 1)?
            })
        }
        "option" => {
            let sa = a.get("some");
            let sb = b.get("some");
            match (sa, sb) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(x), Some(y)) => compare(&d["item"], x, y, depth + 1)?,
            }
        }
        "sequence" | "set" => sequence_cmp(
            &d["item"],
            a.as_array().ok_or_else(|| unsupported("invalid_shape"))?,
            b.as_array().ok_or_else(|| unsupported("invalid_shape"))?,
            depth + 1,
        )?,
        "map" => {
            let aa = a.as_array().ok_or_else(|| unsupported("invalid_shape"))?;
            let bb = b.as_array().ok_or_else(|| unsupported("invalid_shape"))?;
            let mut c = Ordering::Equal;
            for (x, y) in aa.iter().zip(bb) {
                c = compare(&d["key"], &x[0], &y[0], depth + 1)?.then(compare(
                    &d["value"],
                    &x[1],
                    &y[1],
                    depth + 1,
                )?);
                if c != Ordering::Equal {
                    break;
                }
            }
            c.then(aa.len().cmp(&bb.len()))
        }
        _ => return Err(unsupported("invalid_shape")),
    })
}
fn sorted(d: &Value, values: &mut [Value]) -> MetaResult<()> {
    // Bounded insertion sort propagates typed-comparison failures; no rank or
    // node is generated by sorting an already supplied logical Set/Map value.
    for i in 1..values.len() {
        let mut j = i;
        while j > 0 && compare(d, &values[j - 1], &values[j], 0)? == Ordering::Greater {
            values.swap(j - 1, j);
            j -= 1;
        }
    }
    Ok(())
}
#[allow(clippy::too_many_lines)]
fn normalize(mt: &Value, value: &Value, arena: &str, depth: usize) -> MetaResult<Value> {
    if depth > 32 {
        return Err(resource());
    }
    let c = context(mt)?;
    if !c
        .eligible
        .iter()
        .any(|s| s == if arena == "g" { "g_input" } else { "w_output" })
    {
        return Err(bad("input_output_backedge"));
    }
    let d = &c.desc;
    let kind = d["kind"]
        .as_str()
        .ok_or_else(|| unsupported("invalid_shape"))?;
    let sub = |desc: &Value, v: &Value, part: Value| {
        let ty = child_type(&c, part, desc)?;
        normalize(
            &ty,
            v,
            if desc["kind"] == "dag_ref" {
                "g"
            } else {
                arena
            },
            depth + 1,
        )
    };
    Ok(match kind {
        "unit" | "null" | "bool" | "nfc_text" | "bytes" | "u8" | "u32" | "u64" | "i32" | "i64"
        | "decimal" | "finite_f64" | "timestamp_i64" | "duration_i64" => {
            validate_structural_shape_metadata(&c.key, value)?;
            if kind == "decimal" {
                json!(
                    af_core::Decimal::parse(
                        value.as_str().ok_or_else(|| unsupported("invalid_shape"))?
                    )
                    .map_err(|_| unsupported("invalid_shape"))?
                    .as_str()
                )
            } else if kind == "finite_f64" {
                let n = value.as_f64().ok_or_else(|| unsupported("invalid_shape"))?;
                json!(if n == 0.0 { 0.0 } else { n })
            } else {
                value.clone()
            }
        }
        "record" => {
            let object = value.as_object().ok_or_else(|| missing("invalid_shape"))?;
            let fields = d["fields"]
                .as_array()
                .ok_or_else(|| unsupported("invalid_shape"))?;
            if object.len() != fields.len()
                || fields
                    .iter()
                    .any(|r| !object.contains_key(r[0].as_str().unwrap_or("")))
            {
                return Err(missing("invalid_shape"));
            }
            let mut out = serde_json::Map::new();
            for row in fields {
                let n = row[0].as_str().unwrap_or("");
                out.insert(n.to_owned(), sub(&row[1], &value[n], json!(["field", n]))?);
            }
            Value::Object(out)
        }
        "enum" => {
            let cases = d["variants"]
                .as_array()
                .ok_or_else(|| unsupported("invalid_shape"))?;
            let unit = cases.iter().all(|c| c[1]["kind"] == "unit");
            let tag = if unit {
                value.as_str().map(str::to_owned)
            } else {
                value.as_object().and_then(|o| o.keys().next().cloned())
            }
            .ok_or_else(|| unsupported("invalid_shape"))?;
            let case = cases
                .iter()
                .find(|c| c[0] == tag)
                .ok_or_else(|| unsupported("invalid_shape"))?;
            let v = sub(
                &case[1],
                if unit { &Value::Null } else { &value[&tag] },
                json!(["case", tag]),
            )?;
            if unit { json!(tag) } else { json!({tag:v}) }
        }
        "sequence" | "set" => {
            let items = value.as_array().ok_or_else(|| missing("invalid_shape"))?;
            if items.len()
                < usize::try_from(d["minimum"].as_u64().unwrap_or(0)).map_err(|_| resource())?
            {
                return Err(missing("invalid_shape"));
            }
            let mut out = items
                .iter()
                .map(|v| {
                    sub(
                        &d["item"],
                        v,
                        json!([if kind == "sequence" { "item" } else { "member" }, null]),
                    )
                })
                .collect::<MetaResult<Vec<_>>>()?;
            if kind == "set" {
                sorted(&d["item"], &mut out)?;
                for pair in out.windows(2) {
                    if compare(&d["item"], &pair[0], &pair[1], 0)? == Ordering::Equal {
                        return Err(bad("invalid_set_member"));
                    }
                }
            }
            Value::Array(out)
        }
        "map" => {
            let mut out = Vec::new();
            for row in value
                .as_array()
                .ok_or_else(|| unsupported("invalid_shape"))?
            {
                out.push(json!([
                    sub(&d["key"], &row[0], json!(["key", null]))?,
                    sub(&d["value"], &row[1], json!(["value", null]))?
                ]));
            }
            for i in 1..out.len() {
                let mut j = i;
                while j > 0
                    && compare(&d["key"], &out[j - 1][0], &out[j][0], 0)? == Ordering::Greater
                {
                    out.swap(j - 1, j);
                    j -= 1;
                }
            }
            for pair in out.windows(2) {
                if compare(&d["key"], &pair[0][0], &pair[1][0], 0)? == Ordering::Equal {
                    return Err(bad("duplicate_map_key"));
                }
            }
            Value::Array(out)
        }
        "option" => {
            if value.get("none").is_some() {
                json!({"none":null})
            } else {
                json!({"some":sub(&d["item"],&value["some"],json!(["some",null]))?})
            }
        }
        _ => return Err(unsupported("invalid_shape")),
    })
}

#[derive(Clone)]
struct Projected {
    ty: Value,
    value: Value,
    height: usize,
    locals: Vec<usize>,
    external: Vec<(Value, Value)>,
    base: Vec<u64>,
    desc: Value,
    scalar: bool,
    units: usize,
}
struct NodeEdges<'a> {
    arena: &'a str,
    index: usize,
    count: usize,
    prior: &'a [Projected],
    g: &'a [Projected],
    locals: Vec<usize>,
    external: Vec<(Value, Value)>,
    budget: &'a Cell<usize>,
}
impl NodeEdges<'_> {
    fn edge(&mut self, c: &Context, d: &Value, part: Value, provided: &Value) -> MetaResult<Value> {
        let expected = child_type(c, part, d)?;
        let external = self.arena == "w" && d["kind"] == "dag_ref";
        let rank = if self.arena == "w" {
            let o = provided.as_object().ok_or_else(|| bad("invalid_child"))?;
            if o.len() != 1 {
                return Err(bad("invalid_child"));
            }
            o.get(if external { "input" } else { "local" })
                .and_then(Value::as_u64)
                .ok_or_else(|| bad("invalid_child"))?
        } else {
            provided
                .as_u64()
                .ok_or_else(|| bad("input_output_backedge"))?
        };
        let r = usize::try_from(rank).map_err(|_| bad("invalid_child"))?;
        if r >= if external { self.g.len() } else { self.count } {
            return Err(bad("invalid_child"));
        }
        if !external && r >= self.index {
            return Err(bad("rank_dependency"));
        }
        let node = if external { &self.g[r] } else { &self.prior[r] };
        if node.ty != expected {
            return Err(bad("invalid_child"));
        }
        let spent = self
            .budget
            .get()
            .checked_add(node.units)
            .ok_or_else(resource)?;
        if spent > 16384 {
            return Err(resource());
        }
        self.budget.set(spent);
        if external {
            self.external.push((expected, node.value.clone()));
        } else {
            self.locals.push(r);
        }
        Ok(node.value.clone())
    }
}
fn node_order(a: &Projected, b: &Projected) -> MetaResult<Ordering> {
    let m = catalogue()?;
    let ak = type_key(&a.ty)?;
    let bk = type_key(&b.ty)?;
    let order = a
        .height
        .cmp(&b.height)
        .then_with(|| a.ty["owner"].as_str().cmp(&b.ty["owner"].as_str()))
        .then_with(|| {
            m["type_ranks"][ak]
                .as_u64()
                .cmp(&m["type_ranks"][bk].as_u64())
        })
        .then_with(|| a.ty["name"].as_str().cmp(&b.ty["name"].as_str()));
    if order != Ordering::Equal {
        return Ok(order);
    }
    let mut order = if a.scalar && b.scalar {
        compare(&b.desc, &a.value, &b.value, 0)?
    } else {
        a.base.cmp(&b.base)
    };
    if order == Ordering::Equal {
        for ((at, av), (bt, bv)) in a.external.iter().zip(&b.external) {
            order = at["owner"]
                .as_str()
                .cmp(&bt["owner"].as_str())
                .then_with(|| at["name"].as_str().cmp(&bt["name"].as_str()));
            if order == Ordering::Equal {
                order = compare(&json!({"ref":type_key(at)?}), av, bv, 0)?;
            }
            if order != Ordering::Equal {
                break;
            }
        }
    }
    Ok(order.then_with(|| a.locals.cmp(&b.locals)))
}

#[allow(clippy::too_many_lines, clippy::many_single_char_names)]
fn graph(
    arena: &str,
    provided: &Value,
    sources: &[Value],
    g: &[Projected],
) -> MetaResult<Vec<Projected>> {
    let o = provided
        .as_object()
        .ok_or_else(|| unsupported("invalid_shape"))?;
    if o.len() != 2 || !o.contains_key("nodes") || !o.contains_key("roots") {
        return Err(unsupported("invalid_shape"));
    }
    let nodes = provided["nodes"].as_array().ok_or_else(resource)?;
    if nodes.len() > 64 {
        return Err(resource());
    }
    let source_values = sources
        .iter()
        .map(|s| normalize(&s["type"], &s["value"], arena, 0))
        .collect::<MetaResult<Vec<_>>>()?;
    let m = catalogue()?;
    let mut projected: Vec<Projected> = Vec::new();
    let budget = Cell::new(0usize);
    for (i, node) in nodes.iter().enumerate() {
        let object = node
            .as_object()
            .ok_or_else(|| unsupported("invalid_shape"))?;
        if object.len() != 2 || !object.contains_key("type") || !object.contains_key("body") {
            return Err(unsupported("invalid_shape"));
        }
        let mt = &node["type"];
        type_key(mt)?;
        let c = context(mt)?;
        if !c
            .eligible
            .iter()
            .any(|s| s == if arena == "g" { "g_input" } else { "w_output" })
        {
            return Err(bad("input_output_backedge"));
        }
        let body = node["body"]
            .as_object()
            .ok_or_else(|| unsupported("invalid_shape"))?;
        if body.len() != 1 {
            return Err(unsupported("invalid_shape"));
        }
        let (tag, payload) = body
            .iter()
            .next()
            .ok_or_else(|| unsupported("invalid_shape"))?;
        let d = &c.desc;
        let kind = d["kind"]
            .as_str()
            .ok_or_else(|| unsupported("invalid_shape"))?;
        let mut edges = NodeEdges {
            arena,
            index: i,
            count: nodes.len(),
            prior: &projected,
            g,
            locals: Vec::new(),
            external: Vec::new(),
            budget: &budget,
        };
        let scalar = m["structural_binding_registration"]["scalar_bodies"][kind].as_str();
        let base;
        let value;
        if let Some(st) = scalar {
            if tag != "scalar"
                || payload
                    .as_object()
                    .is_none_or(|p| p.len() != 1 || !p.contains_key(st))
            {
                return Err(unsupported("invalid_shape"));
            }
            value = normalize(mt, &payload[st], arena, 0)?;
            base = vec![];
        } else {
            match kind {
                "record" => {
                    if tag != "record" {
                        return Err(unsupported("invalid_shape"));
                    }
                    let rows = payload
                        .as_array()
                        .ok_or_else(|| unsupported("invalid_shape"))?;
                    let fields = d["fields"]
                        .as_array()
                        .ok_or_else(|| unsupported("invalid_shape"))?;
                    if rows.len() != fields.len() {
                        return Err(missing("invalid_shape"));
                    }
                    let mut out = serde_json::Map::new();
                    for (row, field) in rows.iter().zip(fields) {
                        let name = field[0]
                            .as_str()
                            .ok_or_else(|| unsupported("invalid_shape"))?;
                        if row["field"] != name
                            || row
                                .as_object()
                                .is_none_or(|o| o.len() != 2 || !o.contains_key("value"))
                        {
                            return Err(unsupported("invalid_shape"));
                        }
                        out.insert(
                            name.to_owned(),
                            edges.edge(&c, &field[1], json!(["field", name]), &row["value"])?,
                        );
                    }
                    value = Value::Object(out);
                    base = vec![];
                }
                "enum" => {
                    if tag != "variant"
                        || payload.as_object().is_none_or(|o| {
                            o.len() != 2 || !o.contains_key("case") || !o.contains_key("payload")
                        })
                    {
                        return Err(unsupported("invalid_shape"));
                    }
                    let cases = d["variants"]
                        .as_array()
                        .ok_or_else(|| unsupported("invalid_shape"))?;
                    let index = cases
                        .iter()
                        .position(|r| r[0] == payload["case"])
                        .ok_or_else(|| unsupported("invalid_shape"))?;
                    let case = payload["case"]
                        .as_str()
                        .ok_or_else(|| unsupported("invalid_shape"))?;
                    let v = edges.edge(
                        &c,
                        &cases[index][1],
                        json!(["case", case]),
                        &payload["payload"],
                    )?;
                    value = if cases.iter().all(|r| r[1]["kind"] == "unit") {
                        json!(case)
                    } else {
                        json!({case:v})
                    };
                    base = vec![u64::try_from(index).map_err(|_| resource())?];
                }
                "sequence" | "set" => {
                    if tag != kind {
                        return Err(unsupported("invalid_shape"));
                    }
                    let rows = payload
                        .as_array()
                        .ok_or_else(|| unsupported("invalid_shape"))?;
                    if u64::try_from(rows.len()).map_err(|_| resource())?
                        < d["minimum"].as_u64().unwrap_or(0)
                    {
                        return Err(missing("invalid_shape"));
                    }
                    let mut out = Vec::new();
                    for row in rows {
                        out.push(edges.edge(
                            &c,
                            &d["item"],
                            json!([if kind == "sequence" { "item" } else { "member" }, null]),
                            row,
                        )?);
                    }
                    if kind == "set" {
                        for pair in out.windows(2) {
                            if compare(&d["item"], &pair[0], &pair[1], 0)? != Ordering::Less {
                                return Err(bad("invalid_set_member"));
                            }
                        }
                    }
                    base = vec![u64::try_from(out.len()).map_err(|_| resource())?];
                    value = Value::Array(out);
                }
                "map" => {
                    if tag != "map" {
                        return Err(bad("invalid_map_entry"));
                    }
                    let rows = payload.as_array().ok_or_else(|| bad("invalid_map_entry"))?;
                    let mut out = Vec::new();
                    for row in rows {
                        if row.as_object().is_none_or(|o| {
                            o.len() != 2 || !o.contains_key("key") || !o.contains_key("value")
                        }) {
                            return Err(bad("invalid_map_entry"));
                        }
                        out.push(json!([
                            edges.edge(&c, &d["key"], json!(["key", null]), &row["key"])?,
                            edges.edge(&c, &d["value"], json!(["value", null]), &row["value"])?
                        ]));
                    }
                    for pair in out.windows(2) {
                        let order = compare(&d["key"], &pair[0][0], &pair[1][0], 0)?;
                        if order == Ordering::Equal {
                            return Err(bad("duplicate_map_key"));
                        }
                        if order != Ordering::Less {
                            return Err(bad("invalid_map_entry"));
                        }
                    }
                    base = vec![u64::try_from(out.len()).map_err(|_| resource())?];
                    value = Value::Array(out);
                }
                "option" => {
                    if tag == "option_none" {
                        if !payload.is_null() {
                            return Err(unsupported("invalid_shape"));
                        }
                        value = json!({"none":null});
                        base = vec![0];
                    } else if tag == "option_some" {
                        value =
                            json!({"some":edges.edge(&c,&d["item"],json!(["some",null]),payload)?});
                        base = vec![1];
                    } else {
                        return Err(unsupported("invalid_shape"));
                    }
                }
                _ => return Err(unsupported("invalid_shape")),
            }
        }
        if projected.iter().any(|p| p.ty == *mt && p.value == value) {
            return Err(bad("invalid_interning"));
        }
        let height = edges
            .locals
            .iter()
            .map(|r| projected[*r].height)
            .max()
            .map_or(0, |h| h + 1);
        if height > 32 {
            return Err(resource());
        }
        let units = measured(&value, 0)?;
        let next = Projected {
            ty: mt.clone(),
            value,
            height,
            locals: edges.locals,
            external: edges.external,
            base,
            desc: d.clone(),
            scalar: scalar.is_some(),
            units,
        };
        if let Some(prior) = projected.last() {
            if node_order(prior, &next)? != Ordering::Less {
                return Err(bad("rank_dependency"));
            }
        }
        projected.push(next);
    }
    let roots = provided["roots"]
        .as_array()
        .ok_or_else(|| missing("missing_node"))?;
    if roots.len() != sources.len() {
        return Err(missing("missing_node"));
    }
    let mut todo = Vec::new();
    for ((root, source), value) in roots.iter().zip(sources).zip(source_values) {
        let r = root
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|n| *n < projected.len())
            .ok_or_else(|| bad("invalid_child"))?;
        let node = &projected[r];
        if node.ty != source["type"]
            || compare(
                &json!({"ref":type_key(&source["type"])?}),
                &node.value,
                &value,
                0,
            )? != Ordering::Equal
        {
            return Err(bad("invalid_child"));
        }
        todo.push(r);
    }
    let mut visited = BTreeSet::new();
    while let Some(i) = todo.pop() {
        if visited.insert(i) {
            todo.extend(&projected[i].locals);
        }
    }
    if visited.len() != projected.len() {
        return Err(bad("invalid_interning"));
    }
    Ok(projected)
}

pub(crate) fn check_subgraph(source: &Value, proposal: &Value) -> MetaResult<()> {
    measured(proposal, 0)?;
    let expected = if source.get("w").is_some() { 2 } else { 1 };
    if proposal.as_object().is_none_or(|o| {
        o.len() != expected || !o.contains_key("g") || (expected == 2 && !o.contains_key("w"))
    }) {
        return Err(unsupported("invalid_shape"));
    }
    let declared = source["g"]
        .as_array()
        .ok_or_else(|| unsupported("missing_site"))?;
    let g = graph("g", &proposal["g"], declared, &[])?;
    if let Some(w) = source.get("w") {
        graph(
            "w",
            &proposal["w"],
            w.as_array().ok_or_else(|| unsupported("missing_site"))?,
            &g,
        )?;
    }
    Ok(())
}
