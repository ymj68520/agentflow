use crate::error::{PortableProofError, ValidationStage};

/// Per-process resource guard. The numeric limits are implementation policy;
/// exceeding them does not assert anything about semantic validity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecoderLimits {
    pub max_input_bytes: usize,
    pub max_scalar_bytes: usize,
    pub max_container_items: usize,
    pub max_total_nodes: usize,
    pub max_nesting: usize,
    pub max_graph_work: usize,
}

impl Default for DecoderLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 32 * 1024 * 1024,
            max_scalar_bytes: 1024 * 1024,
            max_container_items: 100_000,
            max_total_nodes: 1_000_000,
            max_nesting: 96,
            max_graph_work: 2_000_000,
        }
    }
}

pub(crate) fn preflight_json(
    bytes: &[u8],
    limits: DecoderLimits,
) -> Result<(), PortableProofError> {
    if bytes.len() > limits.max_input_bytes {
        return Err(limit("input byte limit"));
    }
    if std::str::from_utf8(bytes).is_err() {
        return Err(PortableProofError::new(
            ValidationStage::D0,
            "InvalidUtf8",
            "$",
            "portable input is not valid UTF-8",
        ));
    }
    let mut scan = Scan {
        bytes,
        pos: 0,
        nodes: 0,
        limits,
    };
    scan.value(0)?;
    if scan.pos != bytes.len() {
        return Err(PortableProofError::new(
            ValidationStage::D1,
            "NoncanonicalEncoding",
            "$",
            "trailing bytes after the canonical value",
        ));
    }
    Ok(())
}

fn limit(detail: &'static str) -> PortableProofError {
    PortableProofError::new(ValidationStage::D0, "ResourceLimit", "$", detail)
}

struct Scan<'a> {
    bytes: &'a [u8],
    pos: usize,
    nodes: usize,
    limits: DecoderLimits,
}

impl Scan<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let value = self.peek()?;
        self.pos += 1;
        Some(value)
    }

    fn count_node(&mut self, depth: usize) -> Result<(), PortableProofError> {
        if depth > self.limits.max_nesting {
            return Err(limit("nesting limit"));
        }
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > self.limits.max_total_nodes {
            return Err(limit("total node limit"));
        }
        Ok(())
    }

    fn whitespace(&self) -> bool {
        matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t'))
    }

    fn string(&mut self) -> Result<String, PortableProofError> {
        let start = self.pos;
        if self.bump() != Some(b'"') {
            return Err(shape("$", "expected JSON string"));
        }
        let mut escaped = false;
        loop {
            let Some(byte) = self.bump() else {
                return Err(shape("$", "unterminated JSON string"));
            };
            if !escaped && byte == b'"' {
                break;
            }
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            }
        }
        let raw = &self.bytes[start..self.pos];
        if raw.len() > self.limits.max_scalar_bytes {
            return Err(limit("string length limit"));
        }
        serde_json::from_slice::<String>(raw).map_err(|error| {
            PortableProofError::new(
                ValidationStage::D0,
                "InvalidUnicode",
                "$",
                error.to_string(),
            )
        })
    }

    fn value(&mut self, depth: usize) -> Result<(), PortableProofError> {
        self.count_node(depth)?;
        while self.whitespace() {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => {
                self.string()?;
                Ok(())
            }
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            Some(b'-' | b'0'..=b'9') => {
                self.number();
                Ok(())
            }
            _ => Err(shape("$", "expected JSON value")),
        }
    }

    fn object(&mut self, depth: usize) -> Result<(), PortableProofError> {
        self.pos += 1;
        while self.whitespace() {
            self.pos += 1;
        }
        if self.bump() == Some(b'}') {
            return Ok(());
        }
        self.pos -= 1;
        let mut keys = std::collections::HashSet::new();
        let mut members = 0usize;
        loop {
            while self.whitespace() {
                self.pos += 1;
            }
            let key = self.string()?;
            if !keys.insert(key.clone()) {
                return Err(PortableProofError::new(
                    ValidationStage::D0,
                    "DuplicateField",
                    "$",
                    format!("duplicate object member {key:?}"),
                ));
            }
            members += 1;
            if members > self.limits.max_container_items {
                return Err(limit("object member limit"));
            }
            while self.whitespace() {
                self.pos += 1;
            }
            if self.bump() != Some(b':') {
                return Err(shape("$", "expected ':' after object member"));
            }
            self.value(depth + 1)?;
            while self.whitespace() {
                self.pos += 1;
            }
            match self.bump() {
                Some(b',') => {}
                Some(b'}') => return Ok(()),
                _ => return Err(shape("$", "expected ',' or '}'")),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<(), PortableProofError> {
        self.pos += 1;
        while self.whitespace() {
            self.pos += 1;
        }
        if self.bump() == Some(b']') {
            return Ok(());
        }
        self.pos -= 1;
        let mut items = 0usize;
        loop {
            self.value(depth + 1)?;
            items += 1;
            if items > self.limits.max_container_items {
                return Err(limit("array item limit"));
            }
            while self.whitespace() {
                self.pos += 1;
            }
            match self.bump() {
                Some(b',') => {}
                Some(b']') => return Ok(()),
                _ => return Err(shape("$", "expected ',' or ']'")),
            }
        }
    }

    fn literal(&mut self, literal: &[u8]) -> Result<(), PortableProofError> {
        if self.bytes.get(self.pos..self.pos + literal.len()) == Some(literal) {
            self.pos += literal.len();
            Ok(())
        } else {
            Err(shape("$", "invalid JSON literal"))
        }
    }

    fn number(&mut self) {
        while matches!(
            self.peek(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.pos += 1;
        }
    }
}

fn shape(path: &str, detail: &'static str) -> PortableProofError {
    PortableProofError::new(ValidationStage::D0, "InvalidContainerShape", path, detail)
}
